use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::service::Service;
use crate::domain::repository::ServiceRepository;

/// GetServiceError はサービス取得に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum GetServiceError {
    #[error("service not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

/// GetServiceUseCase はサービス情報取得ユースケース。
pub struct GetServiceUseCase {
    service_repo: Arc<dyn ServiceRepository>,
}

impl GetServiceUseCase {
    pub fn new(service_repo: Arc<dyn ServiceRepository>) -> Self {
        Self { service_repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<Service, GetServiceError> {
        match self.service_repo.find_by_id(id).await {
            Ok(Some(service)) => Ok(service),
            Ok(None) => Err(GetServiceError::NotFound(id)),
            Err(e) => Err(GetServiceError::Internal(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::service_repository::MockServiceRepository;

    #[tokio::test]
    async fn test_get_service_not_found() {
        let mut mock = MockServiceRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetServiceUseCase::new(Arc::new(mock));
        let id = Uuid::new_v4();
        let result = uc.execute(id).await;
        assert!(matches!(result, Err(GetServiceError::NotFound(_))));
    }
}
