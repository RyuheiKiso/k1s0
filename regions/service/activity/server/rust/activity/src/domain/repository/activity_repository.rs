// アクティビティリポジトリ trait。
use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::activity::{Activity, ActivityFilter, CreateActivity};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ActivityRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Activity>>;
    async fn find_by_idempotency_key(&self, key: &str) -> anyhow::Result<Option<Activity>>;
    async fn find_all(&self, filter: &ActivityFilter) -> anyhow::Result<Vec<Activity>>;
    async fn count(&self, filter: &ActivityFilter) -> anyhow::Result<i64>;
    async fn create(&self, input: &CreateActivity, actor_id: &str) -> anyhow::Result<Activity>;
    async fn update_status(
        &self,
        id: Uuid,
        status: &str,
        updated_by: Option<&str>,
    ) -> anyhow::Result<Activity>;
}
