#[derive(Clone, Copy, Debug)]
pub struct AppConfig {
    pub block_size: usize,
    pub quick_probe_enabled: bool,
    pub quick_probe_steps: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            block_size: 4 * 1024 * 1024,
            quick_probe_enabled: true,
            quick_probe_steps: 100,
        }
    }
}
