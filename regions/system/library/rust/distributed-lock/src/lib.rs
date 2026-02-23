pub mod error;
pub mod lock;
pub mod memory;

pub use error::LockError;
pub use lock::{DistributedLock, LockGuard};
pub use memory::InMemoryDistributedLock;

#[cfg(feature = "mock")]
pub use lock::MockDistributedLock;
