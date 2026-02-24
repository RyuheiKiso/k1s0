pub mod client;
pub mod error;
pub mod memory;
#[cfg(feature = "redis")]
pub mod redis;

pub use client::{CacheClient, CacheEntry, LockGuard};
pub use error::CacheError;
pub use memory::InMemoryCacheClient;
#[cfg(feature = "redis")]
pub use self::redis::RedisCacheClient;

#[cfg(feature = "mock")]
pub use client::MockCacheClient;
