use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::{Tenant, TenantStatus};
use crate::domain::repository::TenantRepository;
use crate::infrastructure::kafka_producer::{NoopTenantEventPublisher, TenantEventPublisher};
use crate::infrastructure::keycloak_admin::{KeycloakAdmin, NoopKeycloakAdmin};
use crate::infrastructure::saga_client::{NoopSagaClient, SagaClient};

#[derive(Debug, thiserror::Error)]
pub enum DeleteTenantError {
    #[error("tenant not found: {0}")]
    NotFound(String),
    #[error("invalid status: {0}")]
    InvalidStatus(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteTenantUseCase {
    tenant_repo: Arc<dyn TenantRepository>,
    saga_client: Arc<dyn SagaClient>,
    keycloak_admin: Arc<dyn KeycloakAdmin>,
    event_publisher: Arc<dyn TenantEventPublisher>,
}

impl DeleteTenantUseCase {
    pub fn new(tenant_repo: Arc<dyn TenantRepository>) -> Self {
        Self {
            tenant_repo,
            saga_client: Arc::new(NoopSagaClient),
            keycloak_admin: Arc::new(NoopKeycloakAdmin),
            event_publisher: Arc::new(NoopTenantEventPublisher),
        }
    }

    pub fn with_saga_client(mut self, saga_client: Arc<dyn SagaClient>) -> Self {
        self.saga_client = saga_client;
        self
    }

    pub fn with_keycloak_admin(mut self, keycloak_admin: Arc<dyn KeycloakAdmin>) -> Self {
        self.keycloak_admin = keycloak_admin;
        self
    }

    pub fn with_event_publisher(mut self, event_publisher: Arc<dyn TenantEventPublisher>) -> Self {
        self.event_publisher = event_publisher;
        self
    }

    pub async fn execute(&self, tenant_id: Uuid) -> Result<Tenant, DeleteTenantError> {
        let mut tenant = self
            .tenant_repo
            .find_by_id(&tenant_id)
            .await
            .map_err(|e| DeleteTenantError::Internal(e.to_string()))?
            .ok_or_else(|| DeleteTenantError::NotFound(tenant_id.to_string()))?;

        if tenant.status == TenantStatus::Deleted {
            return Err(DeleteTenantError::InvalidStatus(
                "tenant is already deleted".to_string(),
            ));
        }

        tenant.status = TenantStatus::Deleted;

        self.tenant_repo
            .update(&tenant)
            .await
            .map_err(|e| DeleteTenantError::Internal(e.to_string()))?;

        if let Err(e) = self.event_publisher.publish_tenant_deleted(&tenant).await {
            tracing::warn!(
                tenant_id = %tenant.id,
                error = %e,
                "failed to publish tenant_deleted event"
            );
        }

        // Start deprovisioning saga (failure is non-fatal)
        if let Err(e) = self
            .saga_client
            .start_deprovisioning_saga(&tenant.id.to_string(), &tenant.name)
            .await
        {
            tracing::warn!(
                tenant_id = %tenant.id,
                error = %e,
                "failed to start deprovisioning saga after delete"
            );
        }

        // Keycloak realm cleanup (failure is non-fatal)
        if let Some(realm) = tenant.keycloak_realm.clone() {
            if let Err(e) = self.keycloak_admin.delete_realm(&realm).await {
                tracing::warn!(
                    tenant_id = %tenant.id,
                    realm = %realm,
                    error = %e,
                    "failed to cleanup keycloak realm after delete"
                );
            }
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
    async fn test_delete_tenant_success() {
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
                    status: TenantStatus::Active,
                    plan: Plan::Free.as_str().to_string(),
                    settings: serde_json::json!({}),
                    keycloak_realm: None,
                    db_schema: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            });
        mock.expect_update().returning(|_| Ok(()));

        let uc = DeleteTenantUseCase::new(Arc::new(mock));
        let tenant = uc.execute(tenant_id).await.unwrap();
        assert_eq!(tenant.status, TenantStatus::Deleted);
    }

    #[tokio::test]
    async fn test_delete_tenant_not_found() {
        let mut mock = MockTenantRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = DeleteTenantUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DeleteTenantError::NotFound(_) => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_delete_tenant_already_deleted() {
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
                    plan: Plan::Free.as_str().to_string(),
                    settings: serde_json::json!({}),
                    keycloak_realm: None,
                    db_schema: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            });

        let uc = DeleteTenantUseCase::new(Arc::new(mock));
        let result = uc.execute(tenant_id).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            DeleteTenantError::InvalidStatus(_) => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }
}
