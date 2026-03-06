use crate::domain::entity::master_item_version::MasterItemVersion;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait VersionRepository: Send + Sync {
    async fn find_by_item(&self, item_id: Uuid) -> anyhow::Result<Vec<MasterItemVersion>>;
    async fn get_latest_version_number(&self, item_id: Uuid) -> anyhow::Result<i32>;
    async fn create(
        &self,
        item_id: Uuid,
        version_number: i32,
        before_data: Option<serde_json::Value>,
        after_data: Option<serde_json::Value>,
        changed_by: &str,
        change_reason: Option<&str>,
    ) -> anyhow::Result<MasterItemVersion>;
}
