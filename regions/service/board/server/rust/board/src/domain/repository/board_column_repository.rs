// ボードカラムリポジトリ trait。
// テナント分離のため全メソッドに tenant_id パラメータを追加している。
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::board_column::{
    BoardColumn, BoardColumnFilter, DecrementColumnRequest, IncrementColumnRequest, UpdateWipLimitRequest,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait BoardColumnRepository: Send + Sync {
    /// カラムを ID で取得する
    async fn find_by_id(&self, tenant_id: &str, id: Uuid) -> anyhow::Result<Option<BoardColumn>>;
    /// プロジェクト+ステータスコードでカラムを取得する（upsert 用）
    async fn find_by_project_and_status(
        &self,
        tenant_id: &str,
        project_id: Uuid,
        status_code: &str,
    ) -> anyhow::Result<Option<BoardColumn>>;
    /// カラム一覧取得
    async fn find_all(&self, tenant_id: &str, filter: &BoardColumnFilter) -> anyhow::Result<Vec<BoardColumn>>;
    /// 件数取得
    async fn count(&self, tenant_id: &str, filter: &BoardColumnFilter) -> anyhow::Result<i64>;
    /// カラムのタスク数を増加する（WIP チェック付き）
    async fn increment(&self, tenant_id: &str, req: &IncrementColumnRequest) -> anyhow::Result<BoardColumn>;
    /// カラムのタスク数を減少する
    async fn decrement(&self, tenant_id: &str, req: &DecrementColumnRequest) -> anyhow::Result<BoardColumn>;
    /// WIP 制限を更新する（楽観的ロック）
    async fn update_wip_limit(&self, tenant_id: &str, req: &UpdateWipLimitRequest) -> anyhow::Result<BoardColumn>;
}
