use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::{Plan, Tenant};
use crate::domain::repository::TenantRepository;
use crate::infrastructure::kafka_producer::{NoopTenantEventPublisher, TenantEventPublisher};
use crate::usecase::watch_tenant::TenantChangeEvent;
use crate::infrastructure::keycloak_admin::{KeycloakAdmin, NoopKeycloakAdmin};
use crate::infrastructure::saga_client::{NoopSagaClient, SagaClient};

#[derive(Debug, thiserror::Error)]
pub enum CreateTenantError {
    #[error("name conflict: {0}")]
    NameConflict(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateTenantInput {
    pub name: String,
    pub display_name: String,
    pub plan: Plan,
    pub owner_id: Option<Uuid>,
}

pub struct CreateTenantUseCase {
    tenant_repo: Arc<dyn TenantRepository>,
    saga_client: Arc<dyn SagaClient>,
    event_publisher: Arc<dyn TenantEventPublisher>,
    keycloak_admin: Arc<dyn KeycloakAdmin>,
    watch_sender: Option<tokio::sync::broadcast::Sender<TenantChangeEvent>>,
}

impl CreateTenantUseCase {
    pub fn new(tenant_repo: Arc<dyn TenantRepository>) -> Self {
        Self {
            tenant_repo,
            saga_client: Arc::new(NoopSagaClient),
            event_publisher: Arc::new(NoopTenantEventPublisher),
            keycloak_admin: Arc::new(NoopKeycloakAdmin),
            watch_sender: None,
        }
    }

    pub fn with_saga_client(mut self, saga_client: Arc<dyn SagaClient>) -> Self {
        self.saga_client = saga_client;
        self
    }

    pub fn with_event_publisher(mut self, event_publisher: Arc<dyn TenantEventPublisher>) -> Self {
        self.event_publisher = event_publisher;
        self
    }

    pub fn with_keycloak_admin(mut self, keycloak_admin: Arc<dyn KeycloakAdmin>) -> Self {
        self.keycloak_admin = keycloak_admin;
        self
    }

    pub fn with_watch_sender(mut self, sender: tokio::sync::broadcast::Sender<TenantChangeEvent>) -> Self {
        self.watch_sender = Some(sender);
        self
    }

    pub async fn execute(&self, input: CreateTenantInput) -> Result<Tenant, CreateTenantError> {
        // Check name uniqueness
        if let Some(_existing) = self
            .tenant_repo
            .find_by_name(&input.name)
            .await
            .map_err(|e| CreateTenantError::Internal(e.to_string()))?
        {
            return Err(CreateTenantError::NameConflict(input.name));
        }

        let mut tenant = Tenant::new(input.name, input.display_name, input.plan, input.owner_id);

        self.tenant_repo
            .create(&tenant)
            .await
            .map_err(|e| CreateTenantError::Internal(e.to_string()))?;

        // Keycloak realm provisioning (failure is non-fatal)
        let realm_name = format!("k1s0-{}", tenant.name);
        if let Err(e) = self.keycloak_admin.create_realm(&realm_name).await {
            tracing::warn!(
                tenant_id = %tenant.id,
                error = %e,
                "failed to create keycloak realm, tenant created but realm not provisioned"
            );
        } else {
            tenant.keycloak_realm = Some(realm_name);
            if let Err(e) = self.tenant_repo.update(&tenant).await {
                tracing::warn!(
                    tenant_id = %tenant.id,
                    error = %e,
                    "failed to persist keycloak realm after provisioning"
                );
            }
        }

        if let Err(e) = self.event_publisher.publish_tenant_created(&tenant).await {
            tracing::warn!(tenant_id = %tenant.id, error = %e, "failed to publish tenant.created event");
        }

        if let Some(sender) = &self.watch_sender {
            let _ = sender.send(TenantChangeEvent {
                tenant_id: tenant.id.to_string(),
                change_type: "CREATED".to_string(),
                tenant_name: tenant.name.clone(),
                tenant_display_name: tenant.display_name.clone(),
                tenant_status: tenant.status.as_str().to_string(),
                tenant_plan: tenant.plan.as_str().to_string(),
            });
        }

        // Start provisioning saga (failure is non-fatal)
        if let Err(e) = self
            .saga_client
            .start_provisioning_saga(&tenant.id.to_string(), &tenant.name)
            .await
        {
            tracing::warn!(
                tenant_id = %tenant.id,
                error = %e,
                "failed to start provisioning saga, tenant created but saga not triggered"
            );
        }

        Ok(tenant)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{Plan, TenantStatus};
    use crate::domain::repository::tenant_repository::MockTenantRepository;

    #[tokio::test]
    async fn test_create_tenant_success() {
        let mut mock = MockTenantRepository::new();
        mock.expect_find_by_name().returning(|_| Ok(None));
        mock.expect_create().returning(|_| Ok(()));
        mock.expect_update().returning(|_| Ok(()));

        let uc = CreateTenantUseCase::new(Arc::new(mock));
        let input = CreateTenantInput {
            name: "acme-corp".to_string(),
            display_name: "ACME Corporation".to_string(),
            plan: Plan::Professional,
            owner_id: Some(Uuid::new_v4()),
        };

        let tenant = uc.execute(input).await.unwrap();
        assert_eq!(tenant.name, "acme-corp");
        assert_eq!(tenant.display_name, "ACME Corporation");
        assert_eq!(tenant.status, TenantStatus::Provisioning);
        assert_eq!(tenant.plan, Plan::Professional);
    }

    #[tokio::test]
    async fn test_create_tenant_name_conflict() {
        let mut mock = MockTenantRepository::new();
        mock.expect_find_by_name().returning(|name| {
            Ok(Some(Tenant::new(
                name.to_string(),
                "Existing".to_string(),
                Plan::Free,
                None,
            )))
        });

        let uc = CreateTenantUseCase::new(Arc::new(mock));
        let input = CreateTenantInput {
            name: "acme-corp".to_string(),
            display_name: "ACME Corporation".to_string(),
            plan: Plan::Professional,
            owner_id: None,
        };

        let result = uc.execute(input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateTenantError::NameConflict(name) => assert_eq!(name, "acme-corp"),
            e => panic!("unexpected error: {:?}", e),
        }
    }
}
