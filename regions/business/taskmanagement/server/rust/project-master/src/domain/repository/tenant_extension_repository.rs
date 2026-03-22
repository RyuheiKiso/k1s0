// テナント拡張リポジトリ trait。
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::tenant_project_extension::{
    TenantMergedStatus, TenantProjectExtension, UpsertTenantExtension,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TenantExtensionRepository: Send + Sync {
    async fn find(
        &self,
        tenant_id: &str,
        status_definition_id: Uuid,
    ) -> anyhow::Result<Option<TenantProjectExtension>>;
    async fn list_merged(
        &self,
        tenant_id: &str,
        project_type_id: Uuid,
        active_only: bool,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<TenantMergedStatus>>;
    async fn count_merged(
        &self,
        tenant_id: &str,
        project_type_id: Uuid,
    ) -> anyhow::Result<i64>;
    async fn upsert(
        &self,
        input: &UpsertTenantExtension,
    ) -> anyhow::Result<TenantProjectExtension>;
    async fn delete(&self, tenant_id: &str, status_definition_id: Uuid) -> anyhow::Result<()>;
}
