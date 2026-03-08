use crate::domain::entity::master_category::{
    CreateMasterCategory, MasterCategory, UpdateMasterCategory,
};
use crate::domain::repository::category_repository::CategoryRepository;
use crate::usecase::event_publisher::DomainMasterEventPublisher;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct ManageCategoriesUseCase {
    category_repo: Arc<dyn CategoryRepository>,
    event_publisher: Arc<dyn DomainMasterEventPublisher>,
}

impl ManageCategoriesUseCase {
    pub fn new(
        category_repo: Arc<dyn CategoryRepository>,
        event_publisher: Arc<dyn DomainMasterEventPublisher>,
    ) -> Self {
        Self {
            category_repo,
            event_publisher,
        }
    }

    pub async fn list_categories(&self, active_only: bool) -> anyhow::Result<Vec<MasterCategory>> {
        self.category_repo.find_all(active_only).await
    }

    pub async fn get_category(&self, code: &str) -> anyhow::Result<Option<MasterCategory>> {
        self.category_repo.find_by_code(code).await
    }

    pub async fn get_category_by_id(&self, id: Uuid) -> anyhow::Result<Option<MasterCategory>> {
        self.category_repo.find_by_id(id).await
    }

    pub async fn create_category(
        &self,
        input: &CreateMasterCategory,
        created_by: &str,
    ) -> anyhow::Result<MasterCategory> {
        if let Some(_existing) = self.category_repo.find_by_code(&input.code).await? {
            anyhow::bail!("Duplicate code: category '{}' already exists", input.code);
        }
        let category = self.category_repo.create(input, created_by).await?;
        self.publish_category_event("created", created_by, None, Some(&category))
            .await;
        Ok(category)
    }

    pub async fn update_category(
        &self,
        code: &str,
        updated_by: &str,
        input: &UpdateMasterCategory,
    ) -> anyhow::Result<MasterCategory> {
        let existing = self
            .category_repo
            .find_by_code(code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", code))?;
        let updated = self.category_repo.update(code, input).await?;
        self.publish_category_event("updated", updated_by, Some(&existing), Some(&updated))
            .await;
        Ok(updated)
    }

    pub async fn update_category_by_id(
        &self,
        id: Uuid,
        updated_by: &str,
        input: &UpdateMasterCategory,
    ) -> anyhow::Result<MasterCategory> {
        let category = self
            .category_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", id))?;
        self.update_category(&category.code, updated_by, input).await
    }

    pub async fn delete_category(&self, code: &str, deleted_by: &str) -> anyhow::Result<()> {
        let existing = self
            .category_repo
            .find_by_code(code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", code))?;
        self.category_repo.delete(code).await?;
        self.publish_category_event("deleted", deleted_by, Some(&existing), None)
            .await;
        Ok(())
    }

    pub async fn delete_category_by_id(
        &self,
        id: Uuid,
        deleted_by: &str,
    ) -> anyhow::Result<()> {
        let category = self
            .category_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", id))?;
        self.delete_category(&category.code, deleted_by).await
    }

    async fn publish_category_event(
        &self,
        action: &str,
        actor: &str,
        before: Option<&MasterCategory>,
        after: Option<&MasterCategory>,
    ) {
        let category = after.or(before);
        let Some(category) = category else {
            return;
        };

        let event = serde_json::json!({
            "event_type": "DOMAIN_MASTER_CATEGORY_CHANGED",
            "resource_type": "master_category",
            "resource_id": category.id,
            "resource_code": category.code,
            "action": action,
            "actor": actor,
            "before": before,
            "after": after,
            "timestamp": Utc::now().to_rfc3339(),
        });

        if let Err(err) = self.event_publisher.publish_category_changed(&event).await {
            tracing::warn!(
                error = %err,
                action,
                resource_code = %category.code,
                "failed to publish category changed event"
            );
        }
    }
}
