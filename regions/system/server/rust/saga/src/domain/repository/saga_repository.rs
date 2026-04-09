use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::saga_state::{SagaState, SagaStatus};
use crate::domain::entity::saga_step_log::SagaStepLog;

/// `SagaListParams` はSaga一覧取得のパラメータ。
/// cursor が指定された場合は keyset ページネーションを使用し、
/// 指定されない場合は後方互換のため OFFSET ページネーションを使用する。
#[derive(Debug, Clone, Default)]
pub struct SagaListParams {
    pub workflow_name: Option<String>,
    pub status: Option<SagaStatus>,
    pub correlation_id: Option<String>,
    pub page: i32,
    pub page_size: i32,
    /// keyset ページネーション用カーソル。形式: "{`created_at_unix_ms`}_{id}"
    /// 指定された場合はそのカーソル位置より前のレコードを取得する。
    pub cursor: Option<String>,
    /// テナント ID: RLS によるテナント分離のために使用する（CRIT-005 対応）
    pub tenant_id: String,
}

/// `SagaRepository` はSaga永続化のためのリポジトリトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SagaRepository: Send + Sync {
    /// 新しいSagaを作成する。SagaState に `tenant_id` が含まれるためRLS設定はリポジトリ実装が行う。
    async fn create(&self, state: &SagaState) -> anyhow::Result<()>;

    /// `SagaStateとStepLogを原子的に更新する。SagaState.tenant_id` でRLS設定を行う。
    async fn update_with_step_log(
        &self,
        state: &SagaState,
        log: &SagaStepLog,
    ) -> anyhow::Result<()>;

    /// `Sagaのステータスを更新する。tenant_id` を引数で受け取りRLS設定を行う。
    async fn update_status(
        &self,
        saga_id: Uuid,
        status: &SagaStatus,
        error_message: Option<String>,
        tenant_id: &str,
    ) -> anyhow::Result<()>;

    /// `IDでSagaを検索する。tenant_id` を引数で受け取りRLS設定を行う。
    async fn find_by_id(&self, saga_id: Uuid, tenant_id: &str)
        -> anyhow::Result<Option<SagaState>>;

    /// `SagaのステップログをSteps取得する。tenant_id` を引数で受け取りRLS設定を行う。
    async fn find_step_logs(
        &self,
        saga_id: Uuid,
        tenant_id: &str,
    ) -> anyhow::Result<Vec<SagaStepLog>>;

    /// Saga一覧を取得する。SagaListParams に `tenant_id` が含まれる。
    async fn list(&self, params: &SagaListParams) -> anyhow::Result<(Vec<SagaState>, i32)>;

    /// 未完了のSagaを検索する（起動時リカバリ用）。
    /// CRIT-005 注意: このメソッドは全テナント横断で実行するため `tenant_id` フィルタを適用しない。
    /// 将来的には DB 接続ロールを管理者ロールに切り替えて RLS をバイパスする実装に移行すること。
    async fn find_incomplete(&self) -> anyhow::Result<Vec<SagaState>>;
}
