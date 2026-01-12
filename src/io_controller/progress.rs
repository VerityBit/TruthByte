use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use serde::Serialize;

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

pub(super) fn should_cancel(cancel_flag: &Option<Arc<AtomicBool>>) -> bool {
    cancel_flag
        .as_ref()
        .map(|flag| flag.load(Ordering::Relaxed))
        .unwrap_or(false)
}

pub(super) fn speed_mbps(bytes: u64, start_time: Instant) -> f64 {
    let elapsed = start_time.elapsed().as_secs_f64();
    if elapsed <= 0.0 {
        return 0.0;
    }
    (bytes as f64 / (1024.0 * 1024.0)) / elapsed
}

pub(super) fn percent_of(done: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (done as f64 / total as f64) * 100.0
    }
}

pub(super) fn emit_progress(sink: Option<&dyn EventSink>, update: ProgressUpdate) {
    if let Some(sink) = sink {
        sink.progress(update);
    }
}

pub(super) fn emit_error(sink: Option<&dyn EventSink>, message: String) {
    if let Some(sink) = sink {
        sink.error(message);
    }
}
