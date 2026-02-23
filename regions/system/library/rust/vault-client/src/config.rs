use std::time::Duration;

#[derive(Debug, Clone)]
pub struct VaultClientConfig {
    pub server_url: String,
    pub cache_ttl: Duration,
    pub cache_max_capacity: usize,
}

impl VaultClientConfig {
    pub fn new(server_url: impl Into<String>) -> Self {
        Self {
            server_url: server_url.into(),
            cache_ttl: Duration::from_secs(600),
            cache_max_capacity: 500,
        }
    }

    pub fn cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    pub fn cache_max_capacity(mut self, capacity: usize) -> Self {
        self.cache_max_capacity = capacity;
        self
    }
}
