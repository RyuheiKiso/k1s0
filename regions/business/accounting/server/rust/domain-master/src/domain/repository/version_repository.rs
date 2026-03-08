use crate::domain::entity::master_item_version::MasterItemVersion;
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait VersionRepository: Send + Sync {
    async fn find_by_item(&self, item_id: Uuid) -> anyhow::Result<Vec<MasterItemVersion>>;
    async fn get_latest_version_number(&self, item_id: Uuid) -> anyhow::Result<i32>;
    async fn create<'a>(
        &self,
        item_id: Uuid,
        version_number: i32,
        before_data: Option<serde_json::Value>,
        after_data: Option<serde_json::Value>,
        changed_by: &'a str,
        change_reason: Option<&'a str>,
    ) -> anyhow::Result<MasterItemVersion>;
}
