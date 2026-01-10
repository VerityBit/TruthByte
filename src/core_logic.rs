use std::num::Wrapping;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DriveHealthStatus {
    Healthy,
    FakeCapacity,
    PhysicalCorruption,
    DataLoss,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosisReport {
    pub total_capacity: u64,
    pub tested_bytes: u64,
    pub valid_bytes: u64,
    pub error_count: u64,
    pub health_score: f64,
    pub status: DriveHealthStatus,
    pub conclusion: String,
}

fn generate_seed(offset: u64) -> u64 {
    let mut z = Wrapping(offset);
    z += Wrapping(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)) * Wrapping(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)) * Wrapping(0x94D049BB133111EB);
    (z ^ (z >> 31)).0
}

struct SplitMix64 {
    state: u64,
    buffer: u64,
    available: u8,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self {
            state: seed,
            buffer: 0,
            available: 0,
        }
    }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    #[inline(always)]
    fn next_u8(&mut self) -> u8 {
        if self.available == 0 {
            self.buffer = self.next_u64();
            self.available = 8;
        }
        let byte = self.buffer as u8;
        self.buffer >>= 8;
        self.available -= 1;
        byte
    }
}

pub fn fill_block(offset: u64, buffer: &mut [u8]) {
    let seed = generate_seed(offset);
    let mut rng = SplitMix64::new(seed);

    for byte in buffer.iter_mut() {
        *byte = rng.next_u8();
    }
}

pub fn verify_block(offset: u64, buffer: &[u8]) -> Result<(), usize> {
    let seed = generate_seed(offset);
    let mut rng = SplitMix64::new(seed);

    for (index, &actual_byte) in buffer.iter().enumerate() {
        let expected_byte = rng.next_u8();
        if actual_byte != expected_byte {
            return Err(index);
        }
    }
    Ok(())
}

pub fn analyze_failure_sample(expected: &[u8], actual: &[u8]) -> Option<DriveHealthStatus> {
    if expected.is_empty() || expected.len() != actual.len() {
        return None;
    }
    if expected == actual {
        return None;
    }
    if actual.iter().all(|&byte| byte == 0) && expected.iter().any(|&byte| byte != 0) {
        return Some(DriveHealthStatus::FakeCapacity);
    }
    Some(DriveHealthStatus::PhysicalCorruption)
}

fn status_severity(status: DriveHealthStatus) -> u8 {
    match status {
        DriveHealthStatus::Healthy => 0,
        DriveHealthStatus::PhysicalCorruption => 1,
        DriveHealthStatus::DataLoss => 2,
        DriveHealthStatus::FakeCapacity => 3,
    }
}

pub fn generate_report(
    total_capacity: u64,
    tested_bytes: u64,
    valid_bytes: u64,
    mismatch_blocks: u64,
    read_error_blocks: u64,
    sample_status: Option<DriveHealthStatus>,
) -> DiagnosisReport {
    let error_count = mismatch_blocks + read_error_blocks;
    let mut status = if error_count == 0 {
        DriveHealthStatus::Healthy
    } else {
        DriveHealthStatus::PhysicalCorruption
    };

    if read_error_blocks > 0 {
        status = DriveHealthStatus::DataLoss;
    }

    if let Some(sample) = sample_status {
        if status_severity(sample) > status_severity(status) {
            status = sample;
        }
    }

    let health_score = if total_capacity == 0 {
        0.0
    } else {
        let ratio = valid_bytes as f64 / total_capacity as f64;
        (ratio * 100.0).clamp(0.0, 100.0)
    };

    let base = match status {
        DriveHealthStatus::Healthy => "No inconsistencies detected.",
        DriveHealthStatus::FakeCapacity => "Warning: detected zero-filled data; possible fake capacity.",
        DriveHealthStatus::PhysicalCorruption => {
            "Warning: detected random corruption or inconsistent data."
        }
        DriveHealthStatus::DataLoss => "Warning: read errors or missing data detected.",
    };

    let mut conclusion = format!("{base} Errors: {error_count}.");
    if tested_bytes < total_capacity {
        conclusion.push_str(&format!(
            " Verification ended early after {tested_bytes} / {total_capacity} bytes."
        ));
    }

    DiagnosisReport {
        total_capacity,
        tested_bytes,
        valid_bytes,
        error_count,
        health_score,
        status,
        conclusion,
    }
}

#[cfg(test)]
mod tests {
    use super::{fill_block, generate_seed, verify_block};

    #[test]
    fn test_determinism() {
        let mut block1 = vec![0u8; 1024];
        let mut block2 = vec![0u8; 1024];
        fill_block(100, &mut block1);
        fill_block(100, &mut block2);
        assert_eq!(block1, block2);
    }

    #[test]
    fn test_offset_variance() {
        let mut block1 = vec![0u8; 1024];
        let mut block2 = vec![0u8; 1024];
        fill_block(100, &mut block1);
        fill_block(200, &mut block2);
        assert_ne!(block1, block2);
    }

    #[test]
    fn test_seed_separation() {
        let seed_a = generate_seed(0);
        let seed_b = generate_seed(4 * 1024 * 1024 * 1024);
        assert_ne!(seed_a, seed_b);
    }

    #[test]
    fn test_verification_logic() {
        let offset = 500;
        let mut data = vec![0u8; 256];

        fill_block(offset, &mut data);
        assert!(verify_block(offset, &data).is_ok());

        data[10] = data[10].wrapping_add(1);
        assert_eq!(verify_block(offset, &data), Err(10));
    }
}
