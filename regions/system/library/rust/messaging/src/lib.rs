//! k1s0-messaging: Kafka メッセージングの抽象化ライブラリ。
//!
//! このライブラリは k1s0 プロジェクト全体で使用する
//! Kafka プロデューサー・コンシューマーの抽象化を提供する。

pub mod config;
pub mod consumer;
pub mod dlq;
pub mod error;
pub mod event;
pub mod producer;

#[cfg(feature = "kafka")]
pub mod kafka_consumer;
#[cfg(feature = "kafka")]
pub mod kafka_producer;

pub use config::MessagingConfig;
pub use consumer::{ConsumerConfig, EventConsumer};
pub use error::MessagingError;
pub use event::{EventEnvelope, EventMetadata};
pub use producer::{EventProducer, NoOpEventProducer};

#[cfg(feature = "mock")]
pub use producer::MockEventProducer;

// LOW-013 監査対応: テスト用インメモリプロデューサーを共通ライブラリからエクスポートする。
// サービス側のテストで重複定義していた InMemoryProducer の代替として使用可能。
// 使用条件: テスト時（#[cfg(test)]）または "testing" フィーチャー有効時のみ。
#[cfg(any(test, feature = "testing"))]
pub use producer::InMemoryEventProducer;

#[cfg(feature = "kafka")]
pub use kafka_consumer::KafkaEventConsumer;
#[cfg(feature = "kafka")]
pub use kafka_producer::KafkaEventProducer;
