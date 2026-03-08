use crate::domain::entity::master_item::{CreateMasterItem, MasterItem, UpdateMasterItem};
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ItemRepository: Send + Sync {
    async fn find_by_category(
        &self,
        category_id: Uuid,
        active_only: bool,
    ) -> anyhow::Result<Vec<MasterItem>>;
    async fn find_by_category_and_code(
        &self,
        category_id: Uuid,
        code: &str,
    ) -> anyhow::Result<Option<MasterItem>>;
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<MasterItem>>;
    async fn create(
        &self,
        category_id: Uuid,
        input: &CreateMasterItem,
        created_by: &str,
    ) -> anyhow::Result<MasterItem>;
    async fn update(
        &self,
        id: Uuid,
        input: &UpdateMasterItem,
    ) -> anyhow::Result<MasterItem>;
    async fn delete(&self, id: Uuid) -> anyhow::Result<()>;
}
