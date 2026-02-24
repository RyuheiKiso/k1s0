pub mod error;
pub mod lock;
pub mod memory;
#[cfg(feature = "redis")]
pub mod redis;

pub use error::LockError;
pub use lock::{DistributedLock, LockGuard};
pub use memory::InMemoryDistributedLock;
#[cfg(feature = "redis")]
pub use self::redis::RedisDistributedLock;

#[cfg(feature = "mock")]
pub use lock::MockDistributedLock;
