// アクティビティリポジトリ trait。
// RLS テナント分離のため、全 DB 操作メソッドに tenant_id パラメータを持つ。
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::activity::{Activity, ActivityFilter, CreateActivity};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ActivityRepository: Send + Sync {
    async fn find_by_id(&self, tenant_id: &str, id: Uuid) -> anyhow::Result<Option<Activity>>;
    async fn find_by_idempotency_key(&self, tenant_id: &str, key: &str) -> anyhow::Result<Option<Activity>>;
    async fn find_all(&self, tenant_id: &str, filter: &ActivityFilter) -> anyhow::Result<Vec<Activity>>;
    async fn count(&self, tenant_id: &str, filter: &ActivityFilter) -> anyhow::Result<i64>;
    async fn create(&self, tenant_id: &str, input: &CreateActivity, actor_id: &str) -> anyhow::Result<Activity>;
    // updated_by を Option<String> に変更して mockall の lifetime 制約エラーを回避する
    async fn update_status(
        &self,
        tenant_id: &str,
        id: Uuid,
        status: &str,
        updated_by: Option<String>,
    ) -> anyhow::Result<Activity>;
}
