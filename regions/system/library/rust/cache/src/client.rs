use async_trait::async_trait;
use std::time::Duration;

use crate::CacheError;

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub value: String,
    pub expires_at: Option<std::time::Instant>,
}

impl CacheEntry {
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map_or(false, |exp| exp <= std::time::Instant::now())
    }
}

pub struct LockGuard {
    pub key: String,
    pub lock_value: String,
}

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait CacheClient: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError>;
    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<(), CacheError>;
    async fn delete(&self, key: &str) -> Result<bool, CacheError>;
    async fn exists(&self, key: &str) -> Result<bool, CacheError>;
    async fn set_nx(&self, key: &str, value: &str, ttl: Duration) -> Result<bool, CacheError>;
    async fn expire(&self, key: &str, ttl: Duration) -> Result<bool, CacheError>;
}
