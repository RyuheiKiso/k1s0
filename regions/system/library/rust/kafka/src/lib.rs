//! k1s0-kafka: Kafka クライアント設定・管理ライブラリ。
//!
//! Kafka 接続の設定・管理・ヘルスチェックを提供する。

pub mod config;
pub mod error;
pub mod health;
pub mod topic;

pub use config::KafkaConfig;
pub use error::KafkaError;
pub use health::KafkaHealthChecker;
pub use topic::{TopicConfig, TopicPartitionInfo};
