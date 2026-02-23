pub mod client;
pub mod error;
pub mod memory;

pub use client::{CacheClient, CacheEntry, LockGuard};
pub use error::CacheError;
pub use memory::InMemoryCacheClient;

#[cfg(feature = "mock")]
pub use client::MockCacheClient;
