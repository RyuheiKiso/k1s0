// ボードカラムリポジトリ trait。
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::board_column::{
    BoardColumn, BoardColumnFilter, DecrementColumnRequest, IncrementColumnRequest, UpdateWipLimitRequest,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait BoardColumnRepository: Send + Sync {
    /// カラムを ID で取得する
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<BoardColumn>>;
    /// プロジェクト+ステータスコードでカラムを取得する（upsert 用）
    async fn find_by_project_and_status(
        &self,
        project_id: Uuid,
        status_code: &str,
    ) -> anyhow::Result<Option<BoardColumn>>;
    /// カラム一覧取得
    async fn find_all(&self, filter: &BoardColumnFilter) -> anyhow::Result<Vec<BoardColumn>>;
    /// 件数取得
    async fn count(&self, filter: &BoardColumnFilter) -> anyhow::Result<i64>;
    /// カラムのタスク数を増加する（WIP チェック付き）
    async fn increment(&self, req: &IncrementColumnRequest) -> anyhow::Result<BoardColumn>;
    /// カラムのタスク数を減少する
    async fn decrement(&self, req: &DecrementColumnRequest) -> anyhow::Result<BoardColumn>;
    /// WIP 制限を更新する（楽観的ロック）
    async fn update_wip_limit(&self, req: &UpdateWipLimitRequest) -> anyhow::Result<BoardColumn>;
}
