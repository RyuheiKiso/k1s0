use std::time::Duration;

#[derive(Debug, Clone)]
pub struct BulkheadConfig {
    pub max_concurrent_calls: usize,
    pub max_wait_duration: Duration,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self {
            max_concurrent_calls: 20,
            max_wait_duration: Duration::from_millis(500),
        }
    }
}
