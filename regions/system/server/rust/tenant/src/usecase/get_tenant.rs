use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::Tenant;
use crate::domain::repository::TenantRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetTenantError {
    #[error("tenant not found: {0}")]
    NotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetTenantUseCase {
    tenant_repo: Arc<dyn TenantRepository>,
}

impl GetTenantUseCase {
    pub fn new(tenant_repo: Arc<dyn TenantRepository>) -> Self {
        Self { tenant_repo }
    }

    pub async fn execute(&self, tenant_id: Uuid) -> Result<Tenant, GetTenantError> {
        self.tenant_repo
            .find_by_id(&tenant_id)
            .await
            .map_err(|e| GetTenantError::Internal(e.to_string()))?
            .ok_or_else(|| GetTenantError::NotFound(tenant_id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{Plan, TenantStatus};
    use crate::domain::repository::tenant_repository::MockTenantRepository;

    #[tokio::test]
    async fn test_get_tenant_found() {
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
                    plan: Plan::Professional.as_str().to_string(),
                    created_at: chrono::Utc::now(),
                }))
            });

        let uc = GetTenantUseCase::new(Arc::new(mock));
        let tenant = uc.execute(tenant_id).await.unwrap();
        assert_eq!(tenant.id, tenant_id);
        assert_eq!(tenant.name, "acme-corp");
    }

    #[tokio::test]
    async fn test_get_tenant_not_found() {
        let mut mock = MockTenantRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetTenantUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            GetTenantError::NotFound(_) => {}
            e => panic!("unexpected error: {:?}", e),
        }
    }
}
