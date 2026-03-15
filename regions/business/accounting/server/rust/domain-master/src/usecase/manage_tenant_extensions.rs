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
        let extension = self
            .tenant_ext_repo
            .upsert(tenant_id, item_id, input)
            .await?;
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

        let items = self
            .item_repo
            .find_by_category(category_id, active_only)
            .await?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::master_category::MasterCategory;
    use crate::domain::repository::category_repository::MockCategoryRepository;
    use crate::domain::repository::item_repository::MockItemRepository;
    use crate::domain::repository::tenant_extension_repository::MockTenantExtensionRepository;
    use crate::usecase::event_publisher::MockDomainMasterEventPublisher;
    use chrono::Utc;
    use mockall::predicate::*;

    fn sample_item(category_id: Uuid) -> MasterItem {
        MasterItem {
            id: Uuid::new_v4(),
            category_id,
            code: "JPY".to_string(),
            display_name: "Japanese Yen".to_string(),
            description: None,
            attributes: Some(serde_json::json!({"symbol": "¥"})),
            parent_item_id: None,
            effective_from: None,
            effective_until: None,
            is_active: true,
            sort_order: 1,
            created_by: "admin".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_extension(item_id: Uuid) -> TenantMasterExtension {
        TenantMasterExtension {
            id: Uuid::new_v4(),
            tenant_id: "tenant-1".to_string(),
            item_id,
            display_name_override: Some("Custom Yen".to_string()),
            attributes_override: None,
            is_enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_category() -> MasterCategory {
        MasterCategory {
            id: Uuid::new_v4(),
            code: "CURRENCY".to_string(),
            display_name: "Currency".to_string(),
            description: None,
            validation_schema: None,
            is_active: true,
            sort_order: 1,
            created_by: "admin".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn build_usecase(
        category_repo: MockCategoryRepository,
        item_repo: MockItemRepository,
        tenant_ext_repo: MockTenantExtensionRepository,
        event_publisher: MockDomainMasterEventPublisher,
    ) -> ManageTenantExtensionsUseCase {
        ManageTenantExtensionsUseCase::new(
            Arc::new(category_repo),
            Arc::new(item_repo),
            Arc::new(tenant_ext_repo),
            Arc::new(event_publisher),
        )
    }

    #[tokio::test]
    async fn test_get_extension() {
        let category_repo = MockCategoryRepository::new();
        let item_repo = MockItemRepository::new();
        let mut tenant_ext_repo = MockTenantExtensionRepository::new();
        let event_publisher = MockDomainMasterEventPublisher::new();

        let item_id = Uuid::new_v4();
        let ext = sample_extension(item_id);
        let ext_clone = ext.clone();

        tenant_ext_repo
            .expect_find_by_tenant_and_item()
            .with(eq("tenant-1"), eq(item_id))
            .times(1)
            .returning(move |_, _| Ok(Some(ext_clone.clone())));

        let uc = build_usecase(category_repo, item_repo, tenant_ext_repo, event_publisher);
        let result = uc.get_extension("tenant-1", item_id).await;
        assert!(result.is_ok());
        let ext = result.unwrap();
        assert!(ext.is_some());
        assert_eq!(ext.unwrap().tenant_id, "tenant-1");
    }

    #[tokio::test]
    async fn test_upsert_create() {
        let category_repo = MockCategoryRepository::new();
        let mut item_repo = MockItemRepository::new();
        let mut tenant_ext_repo = MockTenantExtensionRepository::new();
        let mut event_publisher = MockDomainMasterEventPublisher::new();

        let item_id = Uuid::new_v4();
        let category_id = Uuid::new_v4();
        let item = sample_item(category_id);
        let item_clone = item.clone();
        let ext = sample_extension(item_id);
        let ext_clone = ext.clone();

        tenant_ext_repo
            .expect_find_by_tenant_and_item()
            .with(eq("tenant-1"), eq(item_id))
            .times(1)
            .returning(|_, _| Ok(None));

        item_repo
            .expect_find_by_id()
            .with(eq(item_id))
            .times(1)
            .returning(move |_| Ok(Some(item_clone.clone())));

        tenant_ext_repo
            .expect_upsert()
            .times(1)
            .returning(move |_, _, _| Ok(ext_clone.clone()));

        event_publisher
            .expect_publish_tenant_extension_changed()
            .times(1)
            .returning(|_| Ok(()));

        let uc = build_usecase(category_repo, item_repo, tenant_ext_repo, event_publisher);

        let input = UpsertTenantMasterExtension {
            display_name_override: Some("Custom Yen".to_string()),
            attributes_override: None,
            is_enabled: Some(true),
        };

        let result = uc
            .upsert_extension("tenant-1", item_id, &input, "admin")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upsert_update() {
        let category_repo = MockCategoryRepository::new();
        let mut item_repo = MockItemRepository::new();
        let mut tenant_ext_repo = MockTenantExtensionRepository::new();
        let mut event_publisher = MockDomainMasterEventPublisher::new();

        let item_id = Uuid::new_v4();
        let category_id = Uuid::new_v4();
        let item = sample_item(category_id);
        let item_clone = item.clone();
        let existing_ext = sample_extension(item_id);
        let existing_ext_clone = existing_ext.clone();
        let updated_ext = sample_extension(item_id);
        let updated_ext_clone = updated_ext.clone();

        tenant_ext_repo
            .expect_find_by_tenant_and_item()
            .with(eq("tenant-1"), eq(item_id))
            .times(1)
            .returning(move |_, _| Ok(Some(existing_ext_clone.clone())));

        item_repo
            .expect_find_by_id()
            .with(eq(item_id))
            .times(1)
            .returning(move |_| Ok(Some(item_clone.clone())));

        tenant_ext_repo
            .expect_upsert()
            .times(1)
            .returning(move |_, _, _| Ok(updated_ext_clone.clone()));

        event_publisher
            .expect_publish_tenant_extension_changed()
            .times(1)
            .returning(|_| Ok(()));

        let uc = build_usecase(category_repo, item_repo, tenant_ext_repo, event_publisher);

        let input = UpsertTenantMasterExtension {
            display_name_override: Some("Updated Name".to_string()),
            attributes_override: None,
            is_enabled: Some(true),
        };

        let result = uc
            .upsert_extension("tenant-1", item_id, &input, "admin")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_success() {
        let category_repo = MockCategoryRepository::new();
        let item_repo = MockItemRepository::new();
        let mut tenant_ext_repo = MockTenantExtensionRepository::new();
        let mut event_publisher = MockDomainMasterEventPublisher::new();

        let item_id = Uuid::new_v4();
        let ext = sample_extension(item_id);
        let ext_clone = ext.clone();

        tenant_ext_repo
            .expect_find_by_tenant_and_item()
            .with(eq("tenant-1"), eq(item_id))
            .times(1)
            .returning(move |_, _| Ok(Some(ext_clone.clone())));

        tenant_ext_repo
            .expect_delete()
            .with(eq("tenant-1"), eq(item_id))
            .times(1)
            .returning(|_, _| Ok(()));

        event_publisher
            .expect_publish_tenant_extension_changed()
            .times(1)
            .returning(|_| Ok(()));

        let uc = build_usecase(category_repo, item_repo, tenant_ext_repo, event_publisher);
        let result = uc.delete_extension("tenant-1", item_id, "admin").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let category_repo = MockCategoryRepository::new();
        let item_repo = MockItemRepository::new();
        let mut tenant_ext_repo = MockTenantExtensionRepository::new();
        let event_publisher = MockDomainMasterEventPublisher::new();

        let item_id = Uuid::new_v4();

        tenant_ext_repo
            .expect_find_by_tenant_and_item()
            .with(eq("tenant-1"), eq(item_id))
            .times(1)
            .returning(|_, _| Ok(None));

        let uc = build_usecase(category_repo, item_repo, tenant_ext_repo, event_publisher);
        let result = uc.delete_extension("tenant-1", item_id, "admin").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_list_tenant_items() {
        let mut category_repo = MockCategoryRepository::new();
        let mut item_repo = MockItemRepository::new();
        let mut tenant_ext_repo = MockTenantExtensionRepository::new();
        let event_publisher = MockDomainMasterEventPublisher::new();

        let category = sample_category();
        let category_id = category.id;
        let category_clone = category.clone();

        let item = sample_item(category_id);
        let items = vec![item.clone()];

        category_repo
            .expect_find_by_code()
            .with(eq("CURRENCY"))
            .times(1)
            .returning(move |_| Ok(Some(category_clone.clone())));

        category_repo
            .expect_find_by_id()
            .with(eq(category_id))
            .times(1)
            .returning(move |_| {
                Ok(Some(MasterCategory {
                    id: category_id,
                    code: "CURRENCY".to_string(),
                    display_name: "Currency".to_string(),
                    description: None,
                    validation_schema: None,
                    is_active: true,
                    sort_order: 1,
                    created_by: "admin".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }))
            });

        item_repo
            .expect_find_by_category()
            .with(eq(category_id), eq(true))
            .times(1)
            .returning(move |_, _| Ok(items.clone()));

        tenant_ext_repo
            .expect_find_by_tenant_and_category()
            .with(eq("tenant-1"), eq(category_id))
            .times(1)
            .returning(|_, _| Ok(vec![]));

        let uc = build_usecase(category_repo, item_repo, tenant_ext_repo, event_publisher);
        let result = uc.list_tenant_items("tenant-1", "CURRENCY").await;
        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].effective_display_name, "Japanese Yen");
    }

    #[tokio::test]
    async fn test_list_merged_with_disabled_filter() {
        let mut category_repo = MockCategoryRepository::new();
        let mut item_repo = MockItemRepository::new();
        let mut tenant_ext_repo = MockTenantExtensionRepository::new();
        let event_publisher = MockDomainMasterEventPublisher::new();

        let category_id = Uuid::new_v4();

        let item1 = sample_item(category_id);
        let _item1_id = item1.id;
        let mut item2 = sample_item(category_id);
        item2.id = Uuid::new_v4();
        item2.code = "USD".to_string();
        item2.display_name = "US Dollar".to_string();
        let item2_id = item2.id;
        let items = vec![item1.clone(), item2.clone()];

        category_repo
            .expect_find_by_id()
            .with(eq(category_id))
            .times(1)
            .returning(move |_| {
                Ok(Some(MasterCategory {
                    id: category_id,
                    code: "CURRENCY".to_string(),
                    display_name: "Currency".to_string(),
                    description: None,
                    validation_schema: None,
                    is_active: true,
                    sort_order: 1,
                    created_by: "admin".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }))
            });

        item_repo
            .expect_find_by_category()
            .with(eq(category_id), eq(true))
            .times(1)
            .returning(move |_, _| Ok(items.clone()));

        let disabled_ext = TenantMasterExtension {
            id: Uuid::new_v4(),
            tenant_id: "tenant-1".to_string(),
            item_id: item2_id,
            display_name_override: None,
            attributes_override: None,
            is_enabled: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        tenant_ext_repo
            .expect_find_by_tenant_and_category()
            .with(eq("tenant-1"), eq(category_id))
            .times(1)
            .returning(move |_, _| Ok(vec![disabled_ext.clone()]));

        let uc = build_usecase(category_repo, item_repo, tenant_ext_repo, event_publisher);
        let result = uc
            .list_tenant_items_by_category_id("tenant-1", category_id, true)
            .await;
        assert!(result.is_ok());
        let merged = result.unwrap();
        // item2 (USD) should be filtered out because its extension is disabled
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].base_item.code, "JPY");
    }
}
