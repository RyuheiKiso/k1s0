// テナント拡張管理ユースケース。
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::tenant_project_extension::{
    TenantMergedStatus, TenantProjectExtension, UpsertTenantExtension,
};
use crate::domain::repository::tenant_extension_repository::TenantExtensionRepository;
use crate::usecase::event_publisher::{
    ProjectMasterEventPublisher, TenantExtensionChangedEvent,
};

pub struct ManageTenantExtensionsUseCase {
    repo: Arc<dyn TenantExtensionRepository>,
    publisher: Arc<dyn ProjectMasterEventPublisher>,
}

impl ManageTenantExtensionsUseCase {
    pub fn new(
        repo: Arc<dyn TenantExtensionRepository>,
        publisher: Arc<dyn ProjectMasterEventPublisher>,
    ) -> Self {
        Self { repo, publisher }
    }

    pub async fn get(
        &self,
        tenant_id: &str,
        status_definition_id: Uuid,
    ) -> anyhow::Result<Option<TenantProjectExtension>> {
        self.repo.find(tenant_id, status_definition_id).await
    }

    pub async fn list_merged(
        &self,
        tenant_id: &str,
        project_type_id: Uuid,
        active_only: bool,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<(Vec<TenantMergedStatus>, i64)> {
        let items = self
            .repo
            .list_merged(tenant_id, project_type_id, active_only, limit, offset)
            .await?;
        let total = self.repo.count_merged(tenant_id, project_type_id).await?;
        Ok((items, total))
    }

    pub async fn upsert(
        &self,
        input: &UpsertTenantExtension,
    ) -> anyhow::Result<TenantProjectExtension> {
        let ext = self.repo.upsert(input).await?;
        let event = TenantExtensionChangedEvent {
            tenant_id: ext.tenant_id.clone(),
            status_definition_id: ext.status_definition_id.to_string(),
            change_type: "upserted".to_string(),
        };
        self.publisher
            .publish_tenant_extension_changed(&event)
            .await?;
        Ok(ext)
    }

    pub async fn delete(
        &self,
        tenant_id: &str,
        status_definition_id: Uuid,
    ) -> anyhow::Result<()> {
        self.repo.delete(tenant_id, status_definition_id).await?;
        let event = TenantExtensionChangedEvent {
            tenant_id: tenant_id.to_string(),
            status_definition_id: status_definition_id.to_string(),
            change_type: "deleted".to_string(),
        };
        self.publisher
            .publish_tenant_extension_changed(&event)
            .await?;
        Ok(())
    }
}
