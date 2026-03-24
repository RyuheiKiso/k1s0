// アクティビティリポジトリ trait。
// クリーンアーキテクチャに従い、戻り値型に anyhow::Result ではなく ActivityError を使用する。
// RLS テナント分離のため、全 DB 操作メソッドに tenant_id パラメータを持つ。
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::activity::{Activity, ActivityFilter, CreateActivity};
use crate::domain::error::ActivityError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ActivityRepository: Send + Sync {
    /// アクティビティを ID で取得する
    async fn find_by_id(&self, tenant_id: &str, id: Uuid) -> Result<Option<Activity>, ActivityError>;
    /// 冪等性キーでアクティビティを取得する（重複チェック用）
    async fn find_by_idempotency_key(&self, tenant_id: &str, key: &str) -> Result<Option<Activity>, ActivityError>;
    /// アクティビティ一覧取得
    async fn find_all(&self, tenant_id: &str, filter: &ActivityFilter) -> Result<Vec<Activity>, ActivityError>;
    /// 件数取得
    async fn count(&self, tenant_id: &str, filter: &ActivityFilter) -> Result<i64, ActivityError>;
    /// アクティビティ作成（outbox を同一トランザクションで保存）
    async fn create(&self, tenant_id: &str, input: &CreateActivity, actor_id: &str) -> Result<Activity, ActivityError>;
    // updated_by を Option<String> に変更して mockall の lifetime 制約エラーを回避する
    async fn update_status(
        &self,
        tenant_id: &str,
        id: Uuid,
        status: &str,
        updated_by: Option<String>,
    ) -> Result<Activity, ActivityError>;
}
