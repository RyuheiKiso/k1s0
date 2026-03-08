use crate::domain::entity::master_item::{CreateMasterItem, MasterItem, UpdateMasterItem};
use crate::domain::repository::category_repository::CategoryRepository;
use crate::domain::repository::item_repository::ItemRepository;
use crate::domain::repository::version_repository::VersionRepository;
use crate::domain::service::validation_service::ValidationService;
use crate::usecase::event_publisher::DomainMasterEventPublisher;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct ManageItemsUseCase {
    category_repo: Arc<dyn CategoryRepository>,
    item_repo: Arc<dyn ItemRepository>,
    version_repo: Arc<dyn VersionRepository>,
    event_publisher: Arc<dyn DomainMasterEventPublisher>,
}

impl ManageItemsUseCase {
    pub fn new(
        category_repo: Arc<dyn CategoryRepository>,
        item_repo: Arc<dyn ItemRepository>,
        version_repo: Arc<dyn VersionRepository>,
        event_publisher: Arc<dyn DomainMasterEventPublisher>,
    ) -> Self {
        Self {
            category_repo,
            item_repo,
            version_repo,
            event_publisher,
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

    pub async fn list_items_by_category_id(
        &self,
        category_id: Uuid,
        active_only: bool,
    ) -> anyhow::Result<Vec<MasterItem>> {
        self.category_repo
            .find_by_id(category_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_id))?;
        self.item_repo.find_by_category(category_id, active_only).await
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

    pub async fn get_item_by_id(&self, item_id: Uuid) -> anyhow::Result<Option<MasterItem>> {
        self.item_repo.find_by_id(item_id).await
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

        let item = self.item_repo.create(category.id, input, created_by).await?;

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

        self.publish_item_event("created", created_by, None, Some(&item))
            .await;
        Ok(item)
    }

    pub async fn create_item_by_category_id(
        &self,
        category_id: Uuid,
        input: &CreateMasterItem,
        created_by: &str,
    ) -> anyhow::Result<MasterItem> {
        let category = self
            .category_repo
            .find_by_id(category_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_id))?;
        self.create_item(&category.code, input, created_by).await
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

        self.publish_item_event("updated", actor, Some(&existing), Some(&updated))
            .await;
        Ok(updated)
    }

    pub async fn update_item_by_id(
        &self,
        item_id: Uuid,
        input: &UpdateMasterItem,
        actor: &str,
    ) -> anyhow::Result<MasterItem> {
        let existing = self
            .item_repo
            .find_by_id(item_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Item '{}' not found", item_id))?;
        let category = self
            .category_repo
            .find_by_id(existing.category_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", existing.category_id))?;
        self.update_item(&category.code, &existing.code, input, actor)
            .await
    }

    pub async fn delete_item(
        &self,
        category_code: &str,
        item_code: &str,
        actor: &str,
    ) -> anyhow::Result<()> {
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
        self.item_repo.delete(item.id).await?;
        self.publish_item_event("deleted", actor, Some(&item), None)
            .await;
        Ok(())
    }

    pub async fn delete_item_by_id(&self, item_id: Uuid, actor: &str) -> anyhow::Result<()> {
        let item = self
            .item_repo
            .find_by_id(item_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Item '{}' not found", item_id))?;
        self.item_repo.delete(item.id).await?;
        self.publish_item_event("deleted", actor, Some(&item), None)
            .await;
        Ok(())
    }

    async fn publish_item_event(
        &self,
        action: &str,
        actor: &str,
        before: Option<&MasterItem>,
        after: Option<&MasterItem>,
    ) {
        let item = after.or(before);
        let Some(item) = item else {
            return;
        };

        let event = serde_json::json!({
            "event_type": "DOMAIN_MASTER_ITEM_CHANGED",
            "resource_type": "master_item",
            "resource_id": item.id,
            "resource_code": item.code,
            "action": action,
            "actor": actor,
            "before": before,
            "after": after,
            "timestamp": Utc::now().to_rfc3339(),
        });

        if let Err(err) = self.event_publisher.publish_item_changed(&event).await {
            tracing::warn!(
                error = %err,
                action,
                resource_code = %item.code,
                "failed to publish item changed event"
            );
        }
    }
}
