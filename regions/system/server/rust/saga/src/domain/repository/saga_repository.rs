use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::saga_state::{SagaState, SagaStatus};
use crate::domain::entity::saga_step_log::SagaStepLog;

/// SagaListParams はSaga一覧取得のパラメータ。
#[derive(Debug, Clone, Default)]
pub struct SagaListParams {
    pub workflow_name: Option<String>,
    pub status: Option<SagaStatus>,
    pub correlation_id: Option<String>,
    pub page: i32,
    pub page_size: i32,
}

/// SagaRepository はSaga永続化のためのリポジトリトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SagaRepository: Send + Sync {
    /// 新しいSagaを作成する。
    async fn create(&self, state: &SagaState) -> anyhow::Result<()>;

    /// SagaStateとStepLogを原子的に更新する。
    async fn update_with_step_log(
        &self,
        state: &SagaState,
        log: &SagaStepLog,
    ) -> anyhow::Result<()>;

    /// Sagaのステータスを更新する。
    async fn update_status(
        &self,
        saga_id: Uuid,
        status: &SagaStatus,
        error_message: Option<String>,
    ) -> anyhow::Result<()>;

    /// IDでSagaを検索する。
    async fn find_by_id(&self, saga_id: Uuid) -> anyhow::Result<Option<SagaState>>;

    /// SagaのステップログをSteps取得する。
    async fn find_step_logs(&self, saga_id: Uuid) -> anyhow::Result<Vec<SagaStepLog>>;

    /// Saga一覧を取得する。
    async fn list(&self, params: &SagaListParams) -> anyhow::Result<(Vec<SagaState>, i64)>;

    /// 未完了のSagaを検索する（起動時リカバリ用）。
    async fn find_incomplete(&self) -> anyhow::Result<Vec<SagaState>>;
}
