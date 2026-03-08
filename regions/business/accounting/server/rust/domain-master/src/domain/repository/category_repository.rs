use crate::domain::entity::master_category::{
    CreateMasterCategory, MasterCategory, UpdateMasterCategory,
};
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn find_all(&self, active_only: bool) -> anyhow::Result<Vec<MasterCategory>>;
    async fn find_by_code(&self, code: &str) -> anyhow::Result<Option<MasterCategory>>;
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<MasterCategory>>;
    async fn create(
        &self,
        input: &CreateMasterCategory,
        created_by: &str,
    ) -> anyhow::Result<MasterCategory>;
    async fn update(
        &self,
        code: &str,
        input: &UpdateMasterCategory,
    ) -> anyhow::Result<MasterCategory>;
    async fn delete(&self, code: &str) -> anyhow::Result<()>;
}
