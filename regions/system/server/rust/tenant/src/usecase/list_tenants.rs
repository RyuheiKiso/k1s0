use std::sync::Arc;

use crate::domain::entity::Tenant;
use crate::domain::repository::TenantRepository;

#[derive(Debug, thiserror::Error)]
pub enum ListTenantsError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListTenantsUseCase {
    tenant_repo: Arc<dyn TenantRepository>,
}

impl ListTenantsUseCase {
    pub fn new(tenant_repo: Arc<dyn TenantRepository>) -> Self {
        Self { tenant_repo }
    }

    pub async fn execute(
        &self,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<Tenant>, i64), ListTenantsError> {
        let page = if page < 1 { 1 } else { page };
        let page_size = if page_size < 1 { 20 } else { page_size };

        self.tenant_repo
            .list(page, page_size)
            .await
            .map_err(|e| ListTenantsError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::Plan;
    use crate::domain::repository::tenant_repository::MockTenantRepository;

    #[tokio::test]
    async fn test_list_tenants_multiple() {
        let mut mock = MockTenantRepository::new();
        mock.expect_list().returning(|_, _| {
            Ok((
                vec![
                    Tenant::new(
                        "t1".to_string(),
                        "T1".to_string(),
                        Plan::Free.as_str().to_string(),
                        None,
                    ),
                    Tenant::new(
                        "t2".to_string(),
                        "T2".to_string(),
                        Plan::Professional.as_str().to_string(),
                        None,
                    ),
                ],
                2,
            ))
        });

        let uc = ListTenantsUseCase::new(Arc::new(mock));
        let (tenants, total) = uc.execute(1, 20).await.unwrap();
        assert_eq!(tenants.len(), 2);
        assert_eq!(total, 2);
    }

    #[tokio::test]
    async fn test_list_tenants_empty() {
        let mut mock = MockTenantRepository::new();
        mock.expect_list().returning(|_, _| Ok((vec![], 0)));

        let uc = ListTenantsUseCase::new(Arc::new(mock));
        let (tenants, total) = uc.execute(1, 20).await.unwrap();
        assert!(tenants.is_empty());
        assert_eq!(total, 0);
    }
}
