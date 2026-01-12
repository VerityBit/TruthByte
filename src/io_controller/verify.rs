use std::io::{self, ErrorKind, Read, Seek, SeekFrom};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use crate::core_logic::{self, DiagnosisReport, DriveHealthStatus};

use super::DIRECT_IO_ALIGNMENT;
use super::direct_io::{AlignedBuffer, open_direct_read, resolve_block_size};
use super::progress::{emit_error, emit_progress, percent_of, should_cancel, speed_mbps};

impl super::DriveInspector {
    pub fn run_verify_phase(&self, total_bytes: u64) -> io::Result<DiagnosisReport> {
        self.run_verify_phase_with_events(total_bytes, None, None)
    }

    pub fn run_verify_phase_with_events(
        &self,
        total_bytes: u64,
        cancel_flag: Option<Arc<AtomicBool>>,
        sink: Option<&dyn super::EventSink>,
    ) -> io::Result<DiagnosisReport> {
        let block_size = resolve_block_size(self.block_size)?;
        if total_bytes % DIRECT_IO_ALIGNMENT as u64 != 0 {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "Total bytes must be aligned for direct I/O.",
            ));
        }
        let mut file = open_direct_read(std::path::Path::new(&self.file_path)).map_err(|e| {
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
                    super::ProgressUpdate {
                        phase: super::ProgressPhase::Verify,
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
            super::ProgressUpdate {
                phase: super::ProgressPhase::Verify,
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
