use std::io::{self, ErrorKind, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use crate::core_logic;

use super::DIRECT_IO_ALIGNMENT;
use super::direct_io::{AlignedBuffer, align_down_u64, open_direct_write, resolve_block_size};
use super::progress::{emit_error, emit_progress, percent_of, should_cancel, speed_mbps};

impl super::DriveInspector {
    pub fn run_write_phase(&self, limit_mb: u64) -> io::Result<u64> {
        self.run_write_phase_with_events(limit_mb, None, None)
    }

    pub fn run_write_phase_with_events(
        &self,
        limit_mb: u64,
        cancel_flag: Option<Arc<AtomicBool>>,
        sink: Option<&dyn super::EventSink>,
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
                    super::ProgressUpdate {
                        phase: super::ProgressPhase::Write,
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
            super::ProgressUpdate {
                phase: super::ProgressPhase::Write,
                percent: percent_of(current_offset, total_bytes),
                speed_mbps: speed_mbps(current_offset, start_time),
                bytes_written: current_offset,
                bytes_verified: 0,
                total_bytes,
            },
        );

        Ok(current_offset)
    }
}
