use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::{Plan, Tenant};
use crate::domain::repository::TenantRepository;
use crate::infrastructure::kafka_producer::{NoopTenantEventPublisher, TenantEventPublisher};
use crate::usecase::watch_tenant::TenantChangeEvent;

#[derive(Debug, thiserror::Error)]
pub enum UpdateTenantError {
    #[error("tenant not found: {0}")]
    NotFound(String),
    #[error("invalid status: {0}")]
    InvalidStatus(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct UpdateTenantInput {
    pub id: Uuid,
    pub display_name: String,
    pub plan: Plan,
}

pub struct UpdateTenantUseCase {
    tenant_repo: Arc<dyn TenantRepository>,
    event_publisher: Arc<dyn TenantEventPublisher>,
    watch_sender: Option<tokio::sync::broadcast::Sender<TenantChangeEvent>>,
}

impl UpdateTenantUseCase {
    pub fn new(tenant_repo: Arc<dyn TenantRepository>) -> Self {
        Self {
            tenant_repo,
            event_publisher: Arc::new(NoopTenantEventPublisher),
            watch_sender: None,
        }
    }

    pub fn with_event_publisher(mut self, event_publisher: Arc<dyn TenantEventPublisher>) -> Self {
        self.event_publisher = event_publisher;
        self
    }

    pub fn with_watch_sender(mut self, sender: tokio::sync::broadcast::Sender<TenantChangeEvent>) -> Self {
        self.watch_sender = Some(sender);
        self
    }

    pub async fn execute(&self, input: UpdateTenantInput) -> Result<Tenant, UpdateTenantError> {
        let mut tenant = self
            .tenant_repo
            .find_by_id(&input.id)
            .await
            .map_err(|e| UpdateTenantError::Internal(e.to_string()))?
            .ok_or_else(|| UpdateTenantError::NotFound(input.id.to_string()))?;

        tenant.display_name = input.display_name;
        tenant.plan = input.plan;

        self.tenant_repo
            .update(&tenant)
            .await
            .map_err(|e| UpdateTenantError::Internal(e.to_string()))?;

        if let Err(e) = self.event_publisher.publish_tenant_updated(&tenant).await {
            tracing::warn!(tenant_id = %tenant.id, error = %e, "failed to publish tenant.updated event");
        }

        if let Some(sender) = &self.watch_sender {
            let _ = sender.send(TenantChangeEvent {
                tenant_id: tenant.id.to_string(),
                change_type: "UPDATED".to_string(),
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
    use crate::domain::entity::TenantStatus;
    use crate::domain::repository::tenant_repository::MockTenantRepository;

    #[tokio::test]
    async fn test_update_tenant_success() {
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

        let uc = UpdateTenantUseCase::new(Arc::new(mock));
        let input = UpdateTenantInput {
            id: tenant_id,
            display_name: "ACME Corp Updated".to_string(),
            plan: Plan::Professional,
        };

        let tenant = uc.execute(input).await.unwrap();
        assert_eq!(tenant.display_name, "ACME Corp Updated");
        assert_eq!(tenant.plan, Plan::Professional);
    }

    #[tokio::test]
    async fn test_update_tenant_not_found() {
        let mut mock = MockTenantRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = UpdateTenantUseCase::new(Arc::new(mock));
        let input = UpdateTenantInput {
            id: Uuid::new_v4(),
            display_name: "test".to_string(),
            plan: Plan::Free,
        };

        let result = uc.execute(input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            UpdateTenantError::NotFound(_) => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }
}
