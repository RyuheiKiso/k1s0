use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::ServiceRepository;

/// DeleteServiceError はサービス削除に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum DeleteServiceError {
    #[error("service not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

/// DeleteServiceUseCase はサービス削除ユースケース。
pub struct DeleteServiceUseCase {
    service_repo: Arc<dyn ServiceRepository>,
}

impl DeleteServiceUseCase {
    pub fn new(service_repo: Arc<dyn ServiceRepository>) -> Self {
        Self { service_repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<(), DeleteServiceError> {
        // Verify service exists
        match self.service_repo.find_by_id(id).await {
            Ok(Some(_)) => {}
            Ok(None) => return Err(DeleteServiceError::NotFound(id)),
            Err(e) => return Err(DeleteServiceError::Internal(e.to_string())),
        }

        self.service_repo
            .delete(id)
            .await
            .map_err(|e| DeleteServiceError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::service_repository::MockServiceRepository;

    #[tokio::test]
    async fn test_delete_service_not_found() {
        let mut mock = MockServiceRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = DeleteServiceUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(matches!(result, Err(DeleteServiceError::NotFound(_))));
    }
}
