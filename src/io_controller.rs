use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;

use crate::config::AppConfig;
use crate::core_logic::{self, DiagnosisReport, DriveHealthStatus};

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
}

impl DriveInspector {
    pub fn new(path: &str) -> Self {
        Self::with_config(path, AppConfig::default())
    }

    pub fn with_config(path: &str, config: AppConfig) -> Self {
        Self {
            file_path: path.to_string(),
            block_size: config.block_size,
        }
    }

    pub fn run_write_phase(&self, limit_mb: u64) -> io::Result<u64> {
        self.run_write_phase_with_events(limit_mb, None, None)
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

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|e| {
                emit_error(
                    sink,
                    format!("Unable to open target for writing: {}", e),
                );
                e
            })?;

        let mut buffer = vec![0u8; self.block_size];
        let mut current_offset: u64 = 0;
        let limit_bytes = if limit_mb == 0 {
            u64::MAX
        } else {
            limit_mb * 1024 * 1024
        };
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

            let write_len = std::cmp::min(remaining, self.block_size as u64) as usize;
            let target_buf = &mut buffer[0..write_len];
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
        let mut file = File::open(&self.file_path).map_err(|e| {
            emit_error(sink, format!("Unable to open target for reading: {}", e));
            e
        })?;
        let mut buffer = vec![0u8; self.block_size];
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
            let read_len = std::cmp::min(remaining, self.block_size as u64) as usize;
            let target_buf = &mut buffer[0..read_len];

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
                    if let Some(status) =
                        core_logic::analyze_failure_sample(&expected, target_buf)
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
