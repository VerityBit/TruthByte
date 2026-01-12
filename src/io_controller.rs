use std::alloc::{alloc, dealloc, Layout};
use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::ptr::NonNull;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use serde::Serialize;

use crate::config::AppConfig;
use crate::core_logic::{self, DiagnosisReport, DriveHealthStatus};

const DIRECT_IO_ALIGNMENT: usize = 4096;

#[cfg(windows)]
const FILE_FLAG_NO_BUFFERING: u32 = 0x20000000;
#[cfg(windows)]
const FILE_FLAG_WRITE_THROUGH: u32 = 0x80000000;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressPhase {
    Write,
    Verify,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressUpdate {
    pub phase: ProgressPhase,
    pub percent: f64,
    pub speed_mbps: f64,
    pub bytes_written: u64,
    pub bytes_verified: u64,
    pub total_bytes: u64,
}

pub trait EventSink: Send + Sync {
    fn progress(&self, update: ProgressUpdate);
    fn error(&self, message: String);
}

pub struct DriveInspector {
    file_path: String,
    block_size: usize,
    quick_probe_enabled: bool,
    quick_probe_steps: usize,
}

impl DriveInspector {
    pub fn new(path: &str) -> Self {
        Self::with_config(path, AppConfig::default())
    }

    pub fn with_config(path: &str, config: AppConfig) -> Self {
        Self {
            file_path: path.to_string(),
            block_size: config.block_size,
            quick_probe_enabled: config.quick_probe_enabled,
            quick_probe_steps: config.quick_probe_steps,
        }
    }

    pub fn run_write_phase(&self, limit_mb: u64) -> io::Result<u64> {
        self.run_write_phase_with_events(limit_mb, None, None)
    }

    pub fn run_quick_probe_phase(
        &self,
        limit_mb: u64,
        steps: usize,
    ) -> io::Result<Option<DiagnosisReport>> {
        let path = Path::new(&self.file_path);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err(io::Error::new(
                    ErrorKind::NotFound,
                    "Parent directory does not exist.",
                ));
            }
        }
        let block_size = resolve_block_size(self.block_size)?;
        if limit_mb == 0 {
            println!("[INFO] Quick probe skipped: no limit provided.");
            return Ok(None);
        }
        if steps < 2 {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "Quick probe steps must be at least 2.",
            ));
        }
        let limit_bytes_raw = limit_mb * 1024 * 1024;
        let limit_bytes = align_down_u64(limit_bytes_raw, DIRECT_IO_ALIGNMENT as u64);
        if limit_bytes < block_size as u64 {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "Limit too small for quick probe.",
            ));
        }

        let mut offsets = compute_probe_offsets(limit_bytes, block_size, steps);
        if offsets.is_empty() {
            return Ok(None);
        }
        offsets.sort_unstable();
        offsets.dedup();

        println!(
            "[INFO] Quick probe start. Anchors={}, Span={}MB",
            offsets.len(),
            limit_bytes / 1024 / 1024
        );

        let mut file = open_direct_write(path)?;
        let mut buffer = AlignedBuffer::new(block_size, DIRECT_IO_ALIGNMENT)?;

        for &offset in &offsets {
            file.seek(SeekFrom::Start(offset))?;
            core_logic::fill_block(offset, buffer.as_mut_slice());
            file.write_all(buffer.as_mut_slice())?;
        }
        file.sync_all()?;

        let mut verify = open_direct_read(path)?;
        let mut mismatch_blocks: u64 = 0;
        let mut read_error_blocks: u64 = 0;
        let mut valid_bytes: u64 = 0;
        let mut sample_status: Option<DriveHealthStatus> = None;

        for &offset in &offsets {
            verify.seek(SeekFrom::Start(offset))?;
            if let Err(e) = verify.read_exact(buffer.as_mut_slice()) {
                read_error_blocks += 1;
                println!("[ERROR] Quick probe read failure at offset {}: {}", offset, e);
                continue;
            }

            if core_logic::verify_block(offset, buffer.as_mut_slice()).is_ok() {
                valid_bytes += block_size as u64;
                continue;
            }

            mismatch_blocks += 1;
            if matches_other_anchor(&offsets, offset, buffer.as_mut_slice()) {
                sample_status = Some(DriveHealthStatus::FakeCapacity);
                break;
            }
            if sample_status.is_none() {
                sample_status = Some(DriveHealthStatus::PhysicalCorruption);
            }
        }

        let tested_bytes = offsets.len() as u64 * block_size as u64;
        let report = core_logic::generate_report(
            limit_bytes,
            tested_bytes,
            valid_bytes,
            mismatch_blocks,
            read_error_blocks,
            sample_status,
        );

        if report.error_count == 0 {
            println!("[INFO] Quick probe complete: no anomalies.");
            Ok(None)
        } else {
            println!(
                "[RESULT] Quick probe anomaly detected: status={:?}, errors={}.",
                report.status, report.error_count
            );
            Ok(Some(report))
        }
    }

    pub fn run_write_phase_with_events(
        &self,
        limit_mb: u64,
        cancel_flag: Option<Arc<AtomicBool>>,
        sink: Option<&dyn EventSink>,
    ) -> io::Result<u64> {
        let path = Path::new(&self.file_path);

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err(io::Error::new(
                    ErrorKind::NotFound,
                    "Parent directory does not exist.",
                ));
            }
        }

        let block_size = resolve_block_size(self.block_size)?;
        let mut file = open_direct_write(path).map_err(|e| {
            emit_error(sink, format!("Unable to open target for writing: {}", e));
            e
        })?;

        let mut buffer = AlignedBuffer::new(block_size, DIRECT_IO_ALIGNMENT)?;
        let mut current_offset: u64 = 0;
        let limit_bytes_raw = if limit_mb == 0 {
            u64::MAX
        } else {
            limit_mb * 1024 * 1024
        };
        let limit_bytes = align_down_u64(limit_bytes_raw, DIRECT_IO_ALIGNMENT as u64);
        if limit_mb > 0 && limit_bytes == 0 {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "Limit too small for direct I/O alignment.",
            ));
        }
        if limit_mb > 0 && limit_bytes != limit_bytes_raw {
            println!(
                "[INFO] Limit aligned from {} to {} bytes for direct I/O.",
                limit_bytes_raw, limit_bytes
            );
        }
        let total_bytes = if limit_mb == 0 { 0 } else { limit_bytes };
        let start_time = Instant::now();
        let mut last_log_time = Instant::now();
        let mut last_emit_time = Instant::now();

        println!(
            "[INFO] Write phase start. Target={}, Limit={}MB",
            self.file_path, limit_mb
        );

        let mut stop_due_to_full = false;
        while current_offset < limit_bytes && !should_cancel(&cancel_flag) {
            let remaining = limit_bytes.saturating_sub(current_offset);
            if remaining == 0 {
                break;
            }

            let write_len = std::cmp::min(remaining, block_size as u64) as usize;
            let target_buf = &mut buffer.as_mut_slice()[0..write_len];
            core_logic::fill_block(current_offset, target_buf);

            let mut written = 0;
            while written < write_len {
                match file.write(&target_buf[written..]) {
                    Ok(0) => {
                        println!("[INFO] Write stopped: storage full.");
                        stop_due_to_full = true;
                        break;
                    }
                    Ok(count) => {
                        written += count;
                        current_offset += count as u64;
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::WriteZero || e.kind() == ErrorKind::StorageFull {
                            println!("[INFO] Write stopped: storage full.");
                            stop_due_to_full = true;
                            break;
                        }
                        emit_error(
                            sink,
                            format!("Write failure at offset {}: {}", current_offset, e),
                        );
                        return Err(e);
                    }
                }
            }

            if stop_due_to_full {
                break;
            }

            if last_log_time.elapsed().as_secs() >= 2 {
                let mb_written = current_offset / 1024 / 1024;
                println!("[PROGRESS] Written {} MB", mb_written);
                last_log_time = Instant::now();
            }

            if last_emit_time.elapsed().as_millis() >= 500 {
                emit_progress(
                    sink,
                    ProgressUpdate {
                        phase: ProgressPhase::Write,
                        percent: percent_of(current_offset, total_bytes),
                        speed_mbps: speed_mbps(current_offset, start_time),
                        bytes_written: current_offset,
                        bytes_verified: 0,
                        total_bytes,
                    },
                );
                last_emit_time = Instant::now();
            }
        }

        println!("[INFO] Syncing data...");
        if let Err(e) = file.sync_all() {
            emit_error(sink, format!("Failed to sync data: {}", e));
            return Err(e);
        }

        let duration = start_time.elapsed();
        let mb_total = current_offset / 1024 / 1024;
        let speed = if duration.as_secs_f64() > 0.0 {
            mb_total as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        println!(
            "[RESULT] Write complete: {} MB, {:.2}s, {:.2} MB/s",
            mb_total,
            duration.as_secs_f64(),
            speed
        );

        emit_progress(
            sink,
            ProgressUpdate {
                phase: ProgressPhase::Write,
                percent: percent_of(current_offset, total_bytes),
                speed_mbps: speed_mbps(current_offset, start_time),
                bytes_written: current_offset,
                bytes_verified: 0,
                total_bytes,
            },
        );

        Ok(current_offset)
    }

    pub fn run_verify_phase(&self, total_bytes: u64) -> io::Result<DiagnosisReport> {
        self.run_verify_phase_with_events(total_bytes, None, None)
    }

    pub fn run_verify_phase_with_events(
        &self,
        total_bytes: u64,
        cancel_flag: Option<Arc<AtomicBool>>,
        sink: Option<&dyn EventSink>,
    ) -> io::Result<DiagnosisReport> {
        let block_size = resolve_block_size(self.block_size)?;
        if total_bytes % DIRECT_IO_ALIGNMENT as u64 != 0 {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "Total bytes must be aligned for direct I/O.",
            ));
        }
        let mut file = open_direct_read(Path::new(&self.file_path)).map_err(|e| {
            emit_error(sink, format!("Unable to open target for reading: {}", e));
            e
        })?;
        let mut buffer = AlignedBuffer::new(block_size, DIRECT_IO_ALIGNMENT)?;
        let mut current_offset: u64 = 0;
        let mut mismatch_blocks: u64 = 0;
        let mut read_error_blocks: u64 = 0;
        let mut valid_bytes: u64 = 0;
        let mut sample_status: Option<DriveHealthStatus> = None;
        let mut last_log_time = Instant::now();
        let mut last_emit_time = Instant::now();
        let start_time = Instant::now();

        println!("[INFO] Verify phase start. Total Bytes={}", total_bytes);

        while current_offset < total_bytes && !should_cancel(&cancel_flag) {
            let remaining = total_bytes - current_offset;
            let read_len = std::cmp::min(remaining, block_size as u64) as usize;
            let target_buf = &mut buffer.as_mut_slice()[0..read_len];

            match file.read_exact(target_buf) {
                Ok(_) => {}
                Err(e) => {
                    if read_error_blocks < 5 {
                        println!(
                            "[ERROR] Read failed at offset {}: {}. Skipping block.",
                            current_offset, e
                        );
                    }
                    read_error_blocks += 1;
                    if let Err(seek_err) =
                        file.seek(SeekFrom::Start(current_offset + read_len as u64))
                    {
                        emit_error(
                            sink,
                            format!(
                                "Unable to seek past read failure at offset {}: {}",
                                current_offset, seek_err
                            ),
                        );
                        break;
                    }
                    current_offset += read_len as u64;
                    continue;
                }
            }

            match core_logic::verify_block(current_offset, target_buf) {
                Ok(_) => {
                    valid_bytes += read_len as u64;
                }
                Err(bad_idx) => {
                    let global_pos = current_offset + bad_idx as u64;
                    if mismatch_blocks < 5 {
                        println!(
                            "[FAILURE] Mismatch at offset 0x{:X} ({}).",
                            global_pos, global_pos
                        );
                    }
                    mismatch_blocks += 1;

                    let mut expected = vec![0u8; read_len];
                    core_logic::fill_block(current_offset, &mut expected);
                    if let Some(status) = core_logic::analyze_failure_sample(&expected, target_buf)
                    {
                        let replace = match sample_status {
                            None => true,
                            Some(existing) => {
                                let existing_severity = match existing {
                                    DriveHealthStatus::Healthy => 0,
                                    DriveHealthStatus::PhysicalCorruption => 1,
                                    DriveHealthStatus::DataLoss => 2,
                                    DriveHealthStatus::FakeCapacity => 3,
                                };
                                let new_severity = match status {
                                    DriveHealthStatus::Healthy => 0,
                                    DriveHealthStatus::PhysicalCorruption => 1,
                                    DriveHealthStatus::DataLoss => 2,
                                    DriveHealthStatus::FakeCapacity => 3,
                                };
                                new_severity > existing_severity
                            }
                        };
                        if replace {
                            sample_status = Some(status);
                        }
                    }
                }
            }

            current_offset += read_len as u64;

            if last_log_time.elapsed().as_secs() >= 2 {
                let percent = (current_offset as f64 / total_bytes as f64) * 100.0;
                let total_errors = mismatch_blocks + read_error_blocks;
                println!("[PROGRESS] {:.1}% (errors: {})", percent, total_errors);
                last_log_time = Instant::now();
            }

            if last_emit_time.elapsed().as_millis() >= 500 {
                emit_progress(
                    sink,
                    ProgressUpdate {
                        phase: ProgressPhase::Verify,
                        percent: percent_of(current_offset, total_bytes),
                        speed_mbps: speed_mbps(current_offset, start_time),
                        bytes_written: total_bytes,
                        bytes_verified: current_offset,
                        total_bytes,
                    },
                );
                last_emit_time = Instant::now();
            }
        }

        let report = core_logic::generate_report(
            total_bytes,
            current_offset,
            valid_bytes,
            mismatch_blocks,
            read_error_blocks,
            sample_status,
        );

        println!(
            "[RESULT] Verify complete: status={:?}, errors={}.",
            report.status, report.error_count
        );

        emit_progress(
            sink,
            ProgressUpdate {
                phase: ProgressPhase::Verify,
                percent: percent_of(current_offset, total_bytes),
                speed_mbps: speed_mbps(current_offset, start_time),
                bytes_written: total_bytes,
                bytes_verified: current_offset,
                total_bytes,
            },
        );

        Ok(report)
    }
}

fn should_cancel(cancel_flag: &Option<Arc<AtomicBool>>) -> bool {
    cancel_flag
        .as_ref()
        .map(|flag| flag.load(Ordering::Relaxed))
        .unwrap_or(false)
}

fn speed_mbps(bytes: u64, start_time: Instant) -> f64 {
    let elapsed = start_time.elapsed().as_secs_f64();
    if elapsed <= 0.0 {
        return 0.0;
    }
    (bytes as f64 / (1024.0 * 1024.0)) / elapsed
}

fn percent_of(done: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (done as f64 / total as f64) * 100.0
    }
}

fn emit_progress(sink: Option<&dyn EventSink>, update: ProgressUpdate) {
    if let Some(sink) = sink {
        sink.progress(update);
    }
}

fn emit_error(sink: Option<&dyn EventSink>, message: String) {
    if let Some(sink) = sink {
        sink.error(message);
    }
}

fn compute_probe_offsets(total_bytes: u64, block_size: usize, steps: usize) -> Vec<u64> {
    let mut offsets = Vec::with_capacity(steps + 1);
    if total_bytes == 0 || block_size == 0 {
        return offsets;
    }
    let last_offset = total_bytes.saturating_sub(block_size as u64);
    for step in 0..=steps {
        let numerator = total_bytes.saturating_mul(step as u64);
        let raw = numerator / steps as u64;
        let aligned = align_down_u64(raw, DIRECT_IO_ALIGNMENT as u64);
        offsets.push(aligned.min(last_offset));
    }
    offsets
}

fn matches_other_anchor(offsets: &[u64], current: u64, buffer: &[u8]) -> bool {
    for &offset in offsets {
        if offset == current {
            continue;
        }
        if core_logic::verify_block(offset, buffer).is_ok() {
            return true;
        }
    }
    false
}

fn align_up(value: usize, alignment: usize) -> usize {
    if alignment == 0 {
        return value;
    }
    (value + alignment - 1) / alignment * alignment
}

fn align_down_u64(value: u64, alignment: u64) -> u64 {
    if alignment == 0 {
        return value;
    }
    value / alignment * alignment
}

fn resolve_block_size(block_size: usize) -> io::Result<usize> {
    if block_size == 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "Block size must be greater than zero.",
        ));
    }
    Ok(align_up(block_size, DIRECT_IO_ALIGNMENT))
}

fn open_direct_write(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(FILE_FLAG_NO_BUFFERING | FILE_FLAG_WRITE_THROUGH);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_DIRECT);
    }
    options.open(path)
}

fn open_direct_read(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(FILE_FLAG_NO_BUFFERING);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_DIRECT);
    }
    options.open(path)
}

struct AlignedBuffer {
    ptr: NonNull<u8>,
    len: usize,
    alignment: usize,
}

impl AlignedBuffer {
    fn new(len: usize, alignment: usize) -> io::Result<Self> {
        let layout = Layout::from_size_align(len, alignment).map_err(|_| {
            io::Error::new(ErrorKind::InvalidInput, "Invalid alignment for buffer.")
        })?;
        let ptr = unsafe { alloc(layout) };
        let ptr = NonNull::new(ptr).ok_or_else(|| {
            io::Error::new(ErrorKind::Other, "Failed to allocate aligned buffer.")
        })?;
        Ok(Self {
            ptr,
            len,
            alignment,
        })
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        let Ok(layout) = Layout::from_size_align(self.len, self.alignment) else {
            return;
        };
        unsafe {
            dealloc(self.ptr.as_ptr(), layout);
        }
    }
}
