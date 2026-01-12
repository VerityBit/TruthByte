use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::core_logic::{self, DiagnosisReport, DriveHealthStatus};

use super::DIRECT_IO_ALIGNMENT;
use super::direct_io::{AlignedBuffer, align_down_u64, open_direct_read, open_direct_write, resolve_block_size};

impl super::DriveInspector {
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
