use crate::domain::entity::master_item::MasterItem;
use crate::domain::entity::tenant_master_extension::{
    TenantMasterExtension, TenantMergedItem, UpsertTenantMasterExtension,
};
use crate::domain::repository::category_repository::CategoryRepository;
use crate::domain::repository::item_repository::ItemRepository;
use crate::domain::repository::tenant_extension_repository::TenantExtensionRepository;
use crate::usecase::event_publisher::DomainMasterEventPublisher;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct ManageTenantExtensionsUseCase {
    category_repo: Arc<dyn CategoryRepository>,
    item_repo: Arc<dyn ItemRepository>,
    tenant_ext_repo: Arc<dyn TenantExtensionRepository>,
    event_publisher: Arc<dyn DomainMasterEventPublisher>,
}

impl ManageTenantExtensionsUseCase {
    pub fn new(
        category_repo: Arc<dyn CategoryRepository>,
        item_repo: Arc<dyn ItemRepository>,
        tenant_ext_repo: Arc<dyn TenantExtensionRepository>,
        event_publisher: Arc<dyn DomainMasterEventPublisher>,
    ) -> Self {
        Self {
            category_repo,
            item_repo,
            tenant_ext_repo,
            event_publisher,
        }
    }

    pub async fn get_extension(
        &self,
        tenant_id: &str,
        item_id: Uuid,
    ) -> anyhow::Result<Option<TenantMasterExtension>> {
        self.tenant_ext_repo
            .find_by_tenant_and_item(tenant_id, item_id)
            .await
    }

    pub async fn upsert_extension(
        &self,
        tenant_id: &str,
        item_id: Uuid,
        input: &UpsertTenantMasterExtension,
        actor: &str,
    ) -> anyhow::Result<TenantMasterExtension> {
        let existing = self
            .tenant_ext_repo
            .find_by_tenant_and_item(tenant_id, item_id)
            .await?;
        self.item_repo
            .find_by_id(item_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Item not found: {}", item_id))?;
        let extension = self.tenant_ext_repo.upsert(tenant_id, item_id, input).await?;
        let action = if existing.is_some() {
            "updated"
        } else {
            "upserted"
        };
        self.publish_tenant_extension_event(
            action,
            actor,
            tenant_id,
            item_id,
            existing.as_ref(),
            Some(&extension),
        )
        .await;
        Ok(extension)
    }

    pub async fn delete_extension(
        &self,
        tenant_id: &str,
        item_id: Uuid,
        actor: &str,
    ) -> anyhow::Result<()> {
        let existing = self
            .tenant_ext_repo
            .find_by_tenant_and_item(tenant_id, item_id)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Tenant extension not found for tenant '{}' and item '{}'",
                    tenant_id,
                    item_id
                )
            })?;
        self.tenant_ext_repo.delete(tenant_id, item_id).await?;
        self.publish_tenant_extension_event(
            "deleted",
            actor,
            tenant_id,
            item_id,
            Some(&existing),
            None,
        )
        .await;
        Ok(())
    }

    pub async fn list_tenant_items(
        &self,
        tenant_id: &str,
        category_code: &str,
    ) -> anyhow::Result<Vec<TenantMergedItem>> {
        let category = self
            .category_repo
            .find_by_code(category_code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_code))?;
        self.list_tenant_items_by_category_id(tenant_id, category.id, true)
            .await
    }

    pub async fn list_tenant_items_by_category_id(
        &self,
        tenant_id: &str,
        category_id: Uuid,
        active_only: bool,
    ) -> anyhow::Result<Vec<TenantMergedItem>> {
        self.category_repo
            .find_by_id(category_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_id))?;

        let items = self.item_repo.find_by_category(category_id, active_only).await?;

        let extensions = self
            .tenant_ext_repo
            .find_by_tenant_and_category(tenant_id, category_id)
            .await?;

        let merged = items
            .into_iter()
            .filter_map(|item| {
                let ext = extensions.iter().find(|e| e.item_id == item.id).cloned();

                if let Some(ref extension) = ext {
                    if !extension.is_enabled {
                        return None;
                    }
                }

                Some(merge_item(item, ext))
            })
            .collect();

        Ok(merged)
    }

    async fn publish_tenant_extension_event(
        &self,
        operation: &str,
        changed_by: &str,
        tenant_id: &str,
        item_id: Uuid,
        before: Option<&TenantMasterExtension>,
        after: Option<&TenantMasterExtension>,
    ) {
        let event_type = format!("tenant_extension.{}", operation);
        let event = serde_json::json!({
            "event_id": Uuid::new_v4().to_string(),
            "event_type": event_type,
            "tenant_id": tenant_id,
            "item_id": item_id,
            "operation": operation.to_uppercase(),
            "before": before,
            "after": after,
            "changed_by": changed_by,
            "trace_id": Uuid::new_v4().to_string(),
            "timestamp": Utc::now().to_rfc3339(),
        });

        if let Err(err) = self
            .event_publisher
            .publish_tenant_extension_changed(&event)
            .await
        {
            tracing::warn!(
                error = %err,
                operation,
                tenant_id,
                item_id = %item_id,
                "failed to publish tenant extension changed event"
            );
        }
    }
}

fn merge_item(item: MasterItem, extension: Option<TenantMasterExtension>) -> TenantMergedItem {
    let effective_display_name = extension
        .as_ref()
        .and_then(|e| e.display_name_override.clone())
        .unwrap_or_else(|| item.display_name.clone());

    let effective_attributes = match (
        &item.attributes,
        extension
            .as_ref()
            .and_then(|e| e.attributes_override.as_ref()),
    ) {
        (Some(base), Some(override_attrs)) => {
            let mut merged = base.clone();
            if let (Some(base_obj), Some(override_obj)) =
                (merged.as_object_mut(), override_attrs.as_object())
            {
                for (key, value) in override_obj {
                    base_obj.insert(key.clone(), value.clone());
                }
            }
            Some(merged)
        }
        (Some(base), None) => Some(base.clone()),
        (None, Some(override_attrs)) => Some(override_attrs.clone()),
        (None, None) => None,
    };

    TenantMergedItem {
        base_item: item,
        extension,
        effective_display_name,
        effective_attributes,
    }
}
