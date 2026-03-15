use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::{Tenant, TenantStatus};
use crate::domain::repository::TenantRepository;
use crate::domain::service::TenantDomainService;
use crate::usecase::watch_tenant::TenantChangeEvent;

#[derive(Debug, thiserror::Error)]
pub enum ActivateTenantError {
    #[error("tenant not found: {0}")]
    NotFound(String),
    #[error("invalid status: {0}")]
    InvalidStatus(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ActivateTenantUseCase {
    tenant_repo: Arc<dyn TenantRepository>,
    watch_sender: Option<tokio::sync::broadcast::Sender<TenantChangeEvent>>,
}

impl ActivateTenantUseCase {
    pub fn new(tenant_repo: Arc<dyn TenantRepository>) -> Self {
        Self {
            tenant_repo,
            watch_sender: None,
        }
    }

    pub fn with_watch_sender(
        mut self,
        sender: tokio::sync::broadcast::Sender<TenantChangeEvent>,
    ) -> Self {
        self.watch_sender = Some(sender);
        self
    }

    pub async fn execute(&self, tenant_id: Uuid) -> Result<Tenant, ActivateTenantError> {
        let mut tenant = self
            .tenant_repo
            .find_by_id(&tenant_id)
            .await
            .map_err(|e| ActivateTenantError::Internal(e.to_string()))?
            .ok_or_else(|| ActivateTenantError::NotFound(tenant_id.to_string()))?;

        if !TenantDomainService::can_activate(&tenant.status) {
            return Err(ActivateTenantError::InvalidStatus(format!(
                "tenant must be suspended to activate, current status: {}",
                tenant.status.as_str()
            )));
        }

        tenant.status = TenantStatus::Active;

        self.tenant_repo
            .update(&tenant)
            .await
            .map_err(|e| ActivateTenantError::Internal(e.to_string()))?;

        if let Some(sender) = &self.watch_sender {
            let _ = sender.send(TenantChangeEvent {
                tenant_id: tenant.id.to_string(),
                change_type: "ACTIVATED".to_string(),
                tenant_name: tenant.name.clone(),
                tenant_display_name: tenant.display_name.clone(),
                tenant_status: tenant.status.as_str().to_string(),
                tenant_plan: tenant.plan.as_str().to_string(),
            });
        }

        Ok(tenant)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::Plan;
    use crate::domain::repository::tenant_repository::MockTenantRepository;

    #[tokio::test]
    async fn test_activate_tenant_from_suspended() {
        let mut mock = MockTenantRepository::new();
        let tenant_id = Uuid::new_v4();
        let tid = tenant_id;
        mock.expect_find_by_id()
            .withf(move |id| *id == tid)
            .returning(move |_| {
                Ok(Some(Tenant {
                    id: tenant_id,
                    name: "acme-corp".to_string(),
                    display_name: "ACME Corporation".to_string(),
                    status: TenantStatus::Suspended,
                    plan: Plan::Free,
                    owner_id: None,
                    settings: serde_json::json!({}),
                    keycloak_realm: None,
                    db_schema: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            });
        mock.expect_update().returning(|_| Ok(()));

        let uc = ActivateTenantUseCase::new(Arc::new(mock));
        let tenant = uc.execute(tenant_id).await.unwrap();
        assert_eq!(tenant.status, TenantStatus::Active);
    }

    #[tokio::test]
    async fn test_activate_tenant_from_provisioning_succeeds() {
        let mut mock = MockTenantRepository::new();
        let tenant_id = Uuid::new_v4();
        let tid = tenant_id;
        mock.expect_find_by_id()
            .withf(move |id| *id == tid)
            .returning(move |_| {
                Ok(Some(Tenant {
                    id: tenant_id,
                    name: "acme-corp".to_string(),
                    display_name: "ACME Corporation".to_string(),
                    status: TenantStatus::Provisioning,
                    plan: Plan::Free,
                    owner_id: None,
                    settings: serde_json::json!({}),
                    keycloak_realm: None,
                    db_schema: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            });
        mock.expect_update().returning(|_| Ok(()));

        let uc = ActivateTenantUseCase::new(Arc::new(mock));
        let tenant = uc.execute(tenant_id).await.unwrap();
        assert_eq!(tenant.status, TenantStatus::Active);
    }

    #[tokio::test]
    async fn test_activate_deleted_tenant_fails() {
        let mut mock = MockTenantRepository::new();
        let tenant_id = Uuid::new_v4();
        let tid = tenant_id;
        mock.expect_find_by_id()
            .withf(move |id| *id == tid)
            .returning(move |_| {
                Ok(Some(Tenant {
                    id: tenant_id,
                    name: "acme-corp".to_string(),
                    display_name: "ACME Corporation".to_string(),
                    status: TenantStatus::Deleted,
                    plan: Plan::Free,
                    owner_id: None,
                    settings: serde_json::json!({}),
                    keycloak_realm: None,
                    db_schema: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            });

        let uc = ActivateTenantUseCase::new(Arc::new(mock));
        let result = uc.execute(tenant_id).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ActivateTenantError::InvalidStatus(_) => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_activate_tenant_not_found() {
        let mut mock = MockTenantRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = ActivateTenantUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ActivateTenantError::NotFound(_) => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }
}
