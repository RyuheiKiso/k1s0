//! Saga オーケストレーションモジュール。
//!
//! 分散トランザクションを Saga パターンで管理する。
//! ステップの順次実行、失敗時の補償処理、デッドレターキューを提供する。

#[cfg(feature = "domain-event")]
pub mod choreography;
pub mod compensator;
pub mod dead_letter;
pub mod definition;
pub mod metrics;
pub mod orchestrator;
#[cfg(feature = "postgres")]
pub mod persistence;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use definition::{BackoffStrategy, RetryPolicy, SagaBuilder, SagaDefinition};

/// Saga の個別ステップ。
#[async_trait]
pub trait SagaStep: Send + Sync {
    /// ステップ名を返す。
    fn name(&self) -> &str;

    /// ステップを実行する。
    ///
    /// `context` は前のステップからの出力を含む JSON 値。
    async fn execute(
        &self,
        context: &serde_json::Value,
    ) -> Result<serde_json::Value, SagaStepError>;

    /// 補償処理を実行する。
    ///
    /// `context` はこのステップの `execute` の出力。
    async fn compensate(
        &self,
        context: &serde_json::Value,
    ) -> Result<(), SagaStepError>;
}

/// Saga ステップのエラー。
#[derive(Debug, thiserror::Error)]
pub enum SagaStepError {
    /// リトライ可能なエラー。
    #[error("retryable error: {0}")]
    Retryable(String),
    /// リトライ不可能なエラー。
    #[error("non-retryable error: {0}")]
    NonRetryable(String),
}

/// Saga オーケストレータの抽象 trait。
#[async_trait]
pub trait SagaOrchestrator: Send + Sync {
    /// Saga を実行する。
    async fn execute(
        &self,
        definition: &SagaDefinition,
        initial_context: serde_json::Value,
    ) -> Result<SagaResult, crate::error::ConsensusError>;

    /// 中断された Saga を再開する。
    async fn resume(
        &self,
        saga_id: &str,
    ) -> Result<SagaResult, crate::error::ConsensusError>;

    /// デッドレターキューの Saga 一覧を取得する。
    async fn dead_letters(
        &self,
        limit: u32,
    ) -> Result<Vec<SagaInstance>, crate::error::ConsensusError>;
}

/// Saga の実行結果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaResult {
    /// Saga インスタンス ID。
    pub saga_id: String,
    /// 最終ステータス。
    pub status: SagaStatus,
    /// 最終出力（成功時）。
    pub output: Option<serde_json::Value>,
    /// エラーメッセージ（失敗時）。
    pub error: Option<String>,
}

/// Saga のステータス。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SagaStatus {
    /// 実行中。
    Running,
    /// 成功完了。
    Completed,
    /// 補償処理中。
    Compensating,
    /// 補償完了（ロールバック済み）。
    Compensated,
    /// デッドレター（補償失敗）。
    DeadLetter,
    /// タイムアウト。
    TimedOut,
}

impl std::fmt::Display for SagaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "RUNNING"),
            Self::Completed => write!(f, "COMPLETED"),
            Self::Compensating => write!(f, "COMPENSATING"),
            Self::Compensated => write!(f, "COMPENSATED"),
            Self::DeadLetter => write!(f, "DEAD_LETTER"),
            Self::TimedOut => write!(f, "TIMED_OUT"),
        }
    }
}

/// Saga インスタンスの永続化情報。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaInstance {
    /// Saga インスタンス ID。
    pub saga_id: String,
    /// Saga 定義名。
    pub saga_name: String,
    /// 現在のステータス。
    pub status: SagaStatus,
    /// 現在のステップインデックス。
    pub current_step: i32,
    /// コンテキスト（JSON）。
    pub context: serde_json::Value,
    /// エラーメッセージ。
    pub error_message: Option<String>,
    /// 作成日時。
    pub created_at: DateTime<Utc>,
    /// 更新日時。
    pub updated_at: DateTime<Utc>,
}

/// Saga ステップの実行記録。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStepRecord {
    /// Saga インスタンス ID。
    pub saga_id: String,
    /// ステップ名。
    pub step_name: String,
    /// ステップインデックス。
    pub step_index: i32,
    /// ステータス。
    pub status: StepStatus,
    /// 入力コンテキスト。
    pub input: serde_json::Value,
    /// 出力コンテキスト。
    pub output: Option<serde_json::Value>,
    /// エラーメッセージ。
    pub error_message: Option<String>,
    /// 実行日時。
    pub executed_at: DateTime<Utc>,
}

/// ステップの実行ステータス。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StepStatus {
    /// 実行成功。
    Success,
    /// 実行失敗。
    Failed,
    /// 補償成功。
    Compensated,
    /// 補償失敗。
    CompensationFailed,
    /// スキップ。
    Skipped,
}
