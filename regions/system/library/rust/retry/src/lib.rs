//! k1s0-retry: リトライ・サーキットブレーカーライブラリ。
//!
//! 指数バックオフ付きリトライとサーキットブレーカーパターンを提供する。

pub mod circuit_breaker;
pub mod error;
pub mod policy;
pub mod retry;

pub use circuit_breaker::{CircuitBreaker, CircuitBreakerState};
pub use error::RetryError;
pub use policy::RetryConfig;
pub use retry::with_retry;
