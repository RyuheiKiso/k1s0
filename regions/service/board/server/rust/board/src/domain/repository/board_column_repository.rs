// ボードカラムリポジトリ trait。
// クリーンアーキテクチャに従い、戻り値型に anyhow::Result ではなく BoardError を使用する。
// テナント分離のため全メソッドに tenant_id パラメータを追加している。
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::board_column::{
    BoardColumn, BoardColumnFilter, DecrementColumnRequest, IncrementColumnRequest, UpdateWipLimitRequest,
};
use crate::domain::error::BoardError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait BoardColumnRepository: Send + Sync {
    /// カラムを ID で取得する
    async fn find_by_id(&self, tenant_id: &str, id: Uuid) -> Result<Option<BoardColumn>, BoardError>;
    /// プロジェクト+ステータスコードでカラムを取得する（upsert 用）
    async fn find_by_project_and_status(
        &self,
        tenant_id: &str,
        project_id: Uuid,
        status_code: &str,
    ) -> Result<Option<BoardColumn>, BoardError>;
    /// カラム一覧取得
    async fn find_all(&self, tenant_id: &str, filter: &BoardColumnFilter) -> Result<Vec<BoardColumn>, BoardError>;
    /// 件数取得
    async fn count(&self, tenant_id: &str, filter: &BoardColumnFilter) -> Result<i64, BoardError>;
    /// カラムのタスク数を増加する（WIP チェック付き）
    async fn increment(&self, tenant_id: &str, req: &IncrementColumnRequest) -> Result<BoardColumn, BoardError>;
    /// カラムのタスク数を減少する
    async fn decrement(&self, tenant_id: &str, req: &DecrementColumnRequest) -> Result<BoardColumn, BoardError>;
    /// WIP 制限を更新する（楽観的ロック）
    async fn update_wip_limit(&self, tenant_id: &str, req: &UpdateWipLimitRequest) -> Result<BoardColumn, BoardError>;
}
