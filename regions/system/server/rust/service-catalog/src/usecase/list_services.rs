use std::sync::Arc;

use crate::domain::entity::service::Service;
use crate::domain::repository::ServiceRepository;
use crate::domain::repository::service_repository::ServiceListFilters;

/// ListServicesUseCase はサービス一覧取得ユースケース。
pub struct ListServicesUseCase {
    service_repo: Arc<dyn ServiceRepository>,
}

impl ListServicesUseCase {
    pub fn new(service_repo: Arc<dyn ServiceRepository>) -> Self {
        Self { service_repo }
    }

    pub async fn execute(&self, filters: ServiceListFilters) -> anyhow::Result<Vec<Service>> {
        self.service_repo.list(filters).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::service_repository::MockServiceRepository;

    #[tokio::test]
    async fn test_list_services_success() {
        let mut mock = MockServiceRepository::new();
        mock.expect_list().returning(|_| Ok(vec![]));

        let uc = ListServicesUseCase::new(Arc::new(mock));
        let result = uc.execute(ServiceListFilters::default()).await.unwrap();
        assert!(result.is_empty());
    }
}
