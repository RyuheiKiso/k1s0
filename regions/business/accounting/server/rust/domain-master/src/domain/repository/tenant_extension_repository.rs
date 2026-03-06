use crate::domain::entity::tenant_master_extension::{
    TenantMasterExtension, UpsertTenantMasterExtension,
};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait TenantExtensionRepository: Send + Sync {
    async fn find_by_tenant_and_item(
        &self,
        tenant_id: &str,
        item_id: Uuid,
    ) -> anyhow::Result<Option<TenantMasterExtension>>;
    async fn find_by_tenant_and_category(
        &self,
        tenant_id: &str,
        category_id: Uuid,
    ) -> anyhow::Result<Vec<TenantMasterExtension>>;
    async fn upsert(
        &self,
        tenant_id: &str,
        item_id: Uuid,
        input: &UpsertTenantMasterExtension,
    ) -> anyhow::Result<TenantMasterExtension>;
    async fn delete(&self, tenant_id: &str, item_id: Uuid) -> anyhow::Result<()>;
}
