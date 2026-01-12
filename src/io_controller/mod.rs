mod direct_io;
mod progress;
mod probe;
mod verify;
mod write;

use crate::config::AppConfig;

pub use progress::{EventSink, ProgressPhase, ProgressUpdate};

const DIRECT_IO_ALIGNMENT: usize = 4096;

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
}
