#[derive(Clone, Copy, Debug)]
pub struct AppConfig {
    pub block_size: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            block_size: 4 * 1024 * 1024,
        }
    }
}
