//! k1s0-outbox: トランザクショナルアウトボックスパターンの実装。
//!
//! データベーストランザクションと Kafka メッセージ発行の
//! 原子性を保証するアウトボックスパターンを提供する。
//!
//! - `OutboxProcessor` / `OutboxPublisher`: メッセージレベルの段階的ステータス遷移
//! - `OutboxEventPoller` / `OutboxEventHandler`: サービス層の fetch+mark 一括パターン

pub mod error;
pub mod event;
pub mod message;
pub mod poller;
pub mod processor;
pub mod store;
pub mod util;

#[cfg(feature = "postgres")]
pub mod postgres_store;

pub use error::OutboxError;
pub use event::OutboxEvent;
pub use message::{OutboxMessage, OutboxStatus};
pub use poller::{
    OutboxEventFetcher, OutboxEventHandler, OutboxEventPoller, OutboxEventSource,
    RepositoryOutboxSource,
};
pub use processor::{OutboxProcessor, OutboxPublisher};
pub use store::OutboxStore;

#[cfg(feature = "postgres")]
pub use postgres_store::PostgresOutboxStore;
