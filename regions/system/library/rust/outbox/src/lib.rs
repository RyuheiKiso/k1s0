//! k1s0-outbox: トランザクショナルアウトボックスパターンの実装。
//!
//! データベーストランザクションと Kafka メッセージ発行の
//! 原子性を保証するアウトボックスパターンを提供する。

pub mod message;
pub mod processor;
pub mod store;
pub mod error;

#[cfg(feature = "postgres")]
pub mod postgres_store;

pub use error::OutboxError;
pub use message::{OutboxMessage, OutboxStatus};
pub use processor::OutboxProcessor;
pub use store::OutboxStore;

#[cfg(feature = "postgres")]
pub use postgres_store::PostgresOutboxStore;
