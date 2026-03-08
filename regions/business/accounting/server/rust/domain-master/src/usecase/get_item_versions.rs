use crate::domain::entity::master_item_version::MasterItemVersion;
use crate::domain::repository::category_repository::CategoryRepository;
use crate::domain::repository::item_repository::ItemRepository;
use crate::domain::repository::version_repository::VersionRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetItemVersionsUseCase {
    category_repo: Arc<dyn CategoryRepository>,
    item_repo: Arc<dyn ItemRepository>,
    version_repo: Arc<dyn VersionRepository>,
}

impl GetItemVersionsUseCase {
    pub fn new(
        category_repo: Arc<dyn CategoryRepository>,
        item_repo: Arc<dyn ItemRepository>,
        version_repo: Arc<dyn VersionRepository>,
    ) -> Self {
        Self {
            category_repo,
            item_repo,
            version_repo,
        }
    }

    pub async fn list_versions(
        &self,
        category_code: &str,
        item_code: &str,
    ) -> anyhow::Result<Vec<MasterItemVersion>> {
        let category = self
            .category_repo
            .find_by_code(category_code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_code))?;

        let item = self
            .item_repo
            .find_by_category_and_code(category.id, item_code)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Item '{}' not found in category '{}'",
                    item_code,
                    category_code
                )
            })?;

        self.version_repo.find_by_item(item.id).await
    }

    pub async fn list_versions_by_item_id(
        &self,
        item_id: Uuid,
    ) -> anyhow::Result<Vec<MasterItemVersion>> {
        self.item_repo
            .find_by_id(item_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Item '{}' not found", item_id))?;
        self.version_repo.find_by_item(item_id).await
    }
}
