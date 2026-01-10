use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind, Read, Write};
use std::path::Path;
use std::time::Instant;

use crate::config::AppConfig;
use crate::core_logic;

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
            .open(path)?;

        let mut buffer = vec![0u8; self.block_size];
        let mut current_offset: u64 = 0;
        let limit_bytes = if limit_mb == 0 {
            u64::MAX
        } else {
            limit_mb * 1024 * 1024
        };
        let start_time = Instant::now();
        let mut last_log_time = Instant::now();

        println!(
            "[INFO] Write phase start. Target={}, Limit={}MB",
            self.file_path, limit_mb
        );

        let mut stop_due_to_full = false;
        loop {
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
        }

        println!("[INFO] Syncing data...");
        file.sync_all()?;

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

        Ok(current_offset)
    }

    pub fn run_verify_phase(&self, total_bytes: u64) -> io::Result<bool> {
        let mut file = File::open(&self.file_path)?;
        let mut buffer = vec![0u8; self.block_size];
        let mut current_offset: u64 = 0;
        let mut error_count: u64 = 0;
        let mut last_log_time = Instant::now();

        println!("[INFO] Verify phase start. Total Bytes={}", total_bytes);

        while current_offset < total_bytes {
            let remaining = total_bytes - current_offset;
            let read_len = std::cmp::min(remaining, self.block_size as u64) as usize;
            let target_buf = &mut buffer[0..read_len];

            match file.read_exact(target_buf) {
                Ok(_) => {}
                Err(e) => {
                    println!("[ERROR] Read failed at offset {}: {}", current_offset, e);
                    return Err(e);
                }
            }

            match core_logic::verify_block(current_offset, target_buf) {
                Ok(_) => {}
                Err(bad_idx) => {
                    let global_pos = current_offset + bad_idx as u64;
                    if error_count < 10 {
                        println!(
                            "[FAILURE] Mismatch at offset 0x{:X} ({}).",
                            global_pos, global_pos
                        );
                    }
                    error_count += 1;
                }
            }

            current_offset += read_len as u64;

            if last_log_time.elapsed().as_secs() >= 2 {
                let percent = (current_offset as f64 / total_bytes as f64) * 100.0;
                println!("[PROGRESS] {:.1}% (errors: {})", percent, error_count);
                last_log_time = Instant::now();
            }
        }

        if error_count > 0 {
            println!("[RESULT] Verify failed: {} errors.", error_count);
            Ok(false)
        } else {
            println!("[RESULT] Verify ok: data matches.");
            Ok(true)
        }
    }
}
