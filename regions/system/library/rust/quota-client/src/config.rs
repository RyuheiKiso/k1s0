use std::time::Duration;

#[derive(Debug, Clone)]
pub struct QuotaClientConfig {
    pub server_url: String,
    pub timeout: Duration,
    pub policy_cache_ttl: Duration,
}

impl QuotaClientConfig {
    pub fn new(server_url: impl Into<String>) -> Self {
        Self {
            server_url: server_url.into(),
            timeout: Duration::from_secs(5),
            policy_cache_ttl: Duration::from_secs(60),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_policy_cache_ttl(mut self, ttl: Duration) -> Self {
        self.policy_cache_ttl = ttl;
        self
    }
}
