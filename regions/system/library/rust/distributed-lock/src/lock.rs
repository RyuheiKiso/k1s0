use async_trait::async_trait;
use std::time::Duration;

use crate::LockError;

pub struct LockGuard {
    pub key: String,
    pub token: String,
}

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait DistributedLock: Send + Sync {
    async fn acquire(&self, key: &str, ttl: Duration) -> Result<LockGuard, LockError>;
    async fn release(&self, guard: LockGuard) -> Result<(), LockError>;
    async fn extend(&self, guard: &LockGuard, ttl: Duration) -> Result<(), LockError>;
    async fn is_locked(&self, key: &str) -> Result<bool, LockError>;
}
