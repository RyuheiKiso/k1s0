//! k1s0-consensus
//!
//! 分散合意のためのライブラリ。リーダー選出、分散ロック、Saga オーケストレーションを提供する。
//!
//! # 概要
//!
//! - `leader`: リーダー選出（リース方式）とハートビート
//! - `lock`: 分散ロック（DB / Redis）とフェンシングトークン
//! - `saga`: Saga オーケストレーション、補償、デッドレター
//!
//! # Feature フラグ
//!
//! - `postgres`: `PostgreSQL` バックエンドを有効化
//! - `redis-backend`: Redis バックエンドを有効化
//! - `domain-event`: コレオグラフィ Saga を有効化（`k1s0-domain-event` 連携）
//! - `full`: すべての feature を有効化
//!
//! # 使用例
//!
//! ```ignore
//! use k1s0_consensus::leader::{LeaderElector, LeaderLease};
//! use k1s0_consensus::lock::DistributedLock;
//! use k1s0_consensus::saga::{SagaBuilder, SagaStep};
//! ```

pub mod config;
pub mod error;
pub mod leader;
pub mod lock;
pub mod saga;

pub use config::ConsensusConfig;
pub use error::ConsensusError;
pub use leader::{LeaderElector, LeaderEvent, LeaderLease, LeaderWatcher};
pub use lock::{DistributedLock, LockGuard};
pub use saga::{
    BackoffStrategy, RetryPolicy, SagaBuilder, SagaDefinition, SagaInstance, SagaResult,
    SagaStatus, SagaStep,
};
