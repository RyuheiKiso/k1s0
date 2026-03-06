use crate::domain::entity::master_item::MasterItem;
use crate::domain::entity::tenant_master_extension::{
    TenantMasterExtension, TenantMergedItem, UpsertTenantMasterExtension,
};
use crate::domain::repository::category_repository::CategoryRepository;
use crate::domain::repository::item_repository::ItemRepository;
use crate::domain::repository::tenant_extension_repository::TenantExtensionRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct ManageTenantExtensionsUseCase {
    category_repo: Arc<dyn CategoryRepository>,
    item_repo: Arc<dyn ItemRepository>,
    tenant_ext_repo: Arc<dyn TenantExtensionRepository>,
}

impl ManageTenantExtensionsUseCase {
    pub fn new(
        category_repo: Arc<dyn CategoryRepository>,
        item_repo: Arc<dyn ItemRepository>,
        tenant_ext_repo: Arc<dyn TenantExtensionRepository>,
    ) -> Self {
        Self {
            category_repo,
            item_repo,
            tenant_ext_repo,
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
    ) -> anyhow::Result<TenantMasterExtension> {
        self.item_repo
            .find_by_id(item_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Item not found: {}", item_id))?;
        self.tenant_ext_repo
            .upsert(tenant_id, item_id, input)
            .await
    }

    pub async fn delete_extension(
        &self,
        tenant_id: &str,
        item_id: Uuid,
    ) -> anyhow::Result<()> {
        self.tenant_ext_repo
            .delete(tenant_id, item_id)
            .await
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

        let items = self
            .item_repo
            .find_by_category(category.id, true)
            .await?;

        let extensions = self
            .tenant_ext_repo
            .find_by_tenant_and_category(tenant_id, category.id)
            .await?;

        let merged = items
            .into_iter()
            .filter_map(|item| {
                let ext = extensions
                    .iter()
                    .find(|e| e.item_id == item.id)
                    .cloned();

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
}

fn merge_item(item: MasterItem, extension: Option<TenantMasterExtension>) -> TenantMergedItem {
    let effective_display_name = extension
        .as_ref()
        .and_then(|e| e.display_name_override.clone())
        .unwrap_or_else(|| item.display_name.clone());

    let effective_attributes = match (&item.attributes, extension.as_ref().and_then(|e| e.attributes_override.as_ref())) {
        (Some(base), Some(override_attrs)) => {
            let mut merged = base.clone();
            if let (Some(base_obj), Some(override_obj)) = (merged.as_object_mut(), override_attrs.as_object()) {
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
