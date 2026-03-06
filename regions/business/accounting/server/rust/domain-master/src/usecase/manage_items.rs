use crate::domain::entity::master_item::{CreateMasterItem, MasterItem, UpdateMasterItem};
use crate::domain::repository::category_repository::CategoryRepository;
use crate::domain::repository::item_repository::ItemRepository;
use crate::domain::repository::version_repository::VersionRepository;
use crate::domain::service::validation_service::ValidationService;
use std::sync::Arc;

pub struct ManageItemsUseCase {
    category_repo: Arc<dyn CategoryRepository>,
    item_repo: Arc<dyn ItemRepository>,
    version_repo: Arc<dyn VersionRepository>,
}

impl ManageItemsUseCase {
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

    pub async fn list_items(
        &self,
        category_code: &str,
        active_only: bool,
    ) -> anyhow::Result<Vec<MasterItem>> {
        let category = self
            .category_repo
            .find_by_code(category_code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_code))?;
        self.item_repo
            .find_by_category(category.id, active_only)
            .await
    }

    pub async fn get_item(
        &self,
        category_code: &str,
        item_code: &str,
    ) -> anyhow::Result<Option<MasterItem>> {
        let category = self
            .category_repo
            .find_by_code(category_code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_code))?;
        self.item_repo
            .find_by_category_and_code(category.id, item_code)
            .await
    }

    pub async fn create_item(
        &self,
        category_code: &str,
        input: &CreateMasterItem,
        created_by: &str,
    ) -> anyhow::Result<MasterItem> {
        let category = self
            .category_repo
            .find_by_code(category_code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_code))?;

        if let Some(_existing) = self
            .item_repo
            .find_by_category_and_code(category.id, &input.code)
            .await?
        {
            anyhow::bail!(
                "Duplicate code: item '{}' already exists in category '{}'",
                input.code,
                category_code
            );
        }

        ValidationService::validate_item_attributes(&category, &input.attributes)?;

        let item = self
            .item_repo
            .create(category.id, input, created_by)
            .await?;

        self.version_repo
            .create(
                item.id,
                1,
                None,
                Some(serde_json::to_value(&item)?),
                created_by,
                Some("Initial creation"),
            )
            .await?;

        Ok(item)
    }

    pub async fn update_item(
        &self,
        category_code: &str,
        item_code: &str,
        input: &UpdateMasterItem,
        actor: &str,
    ) -> anyhow::Result<MasterItem> {
        let category = self
            .category_repo
            .find_by_code(category_code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_code))?;

        let existing = self
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

        if let Some(ref attrs) = input.attributes {
            ValidationService::validate_item_attributes(&category, &Some(attrs.clone()))?;
        }

        let before_data = serde_json::to_value(&existing)?;
        let updated = self.item_repo.update(existing.id, input).await?;
        let after_data = serde_json::to_value(&updated)?;

        let version_number = self
            .version_repo
            .get_latest_version_number(existing.id)
            .await?
            + 1;
        self.version_repo
            .create(
                existing.id,
                version_number,
                Some(before_data),
                Some(after_data),
                actor,
                None,
            )
            .await?;

        Ok(updated)
    }

    pub async fn delete_item(
        &self,
        category_code: &str,
        item_code: &str,
    ) -> anyhow::Result<()> {
        let category = self
            .category_repo
            .find_by_code(category_code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_code))?;

        self.item_repo
            .find_by_category_and_code(category.id, item_code)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Item '{}' not found in category '{}'",
                    item_code,
                    category_code
                )
            })?;

        let item = self
            .item_repo
            .find_by_category_and_code(category.id, item_code)
            .await?
            .unwrap();
        self.item_repo.delete(item.id).await
    }
}
