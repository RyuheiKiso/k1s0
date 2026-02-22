//! k1s0-messaging: Kafka メッセージングの抽象化ライブラリ。
//!
//! このライブラリは k1s0 プロジェクト全体で使用する
//! Kafka プロデューサー・コンシューマーの抽象化を提供する。

pub mod config;
pub mod consumer;
pub mod error;
pub mod event;
pub mod producer;

pub use config::MessagingConfig;
pub use consumer::{ConsumerConfig, EventConsumer};
pub use error::MessagingError;
pub use event::{EventEnvelope, EventMetadata};
pub use producer::{EventProducer, NoOpEventProducer};

#[cfg(feature = "mock")]
pub use producer::MockEventProducer;
