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
    use crate::domain::entity::service::{Service, ServiceLifecycle, ServiceTier};
    use crate::domain::repository::service_repository::MockServiceRepository;
    use chrono::Utc;

    /// テスト用 Service ヘルパー
    fn make_service(id: Uuid) -> Service {
        Service {
            id,
            name: "test-service".to_string(),
            description: None,
            team_id: Uuid::new_v4(),
            tier: ServiceTier::Standard,
            lifecycle: ServiceLifecycle::Development,
            repository_url: None,
            api_endpoint: None,
            healthcheck_url: None,
            tags: vec![],
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 存在するサービスを正常に削除できる
    #[tokio::test]
    async fn test_delete_service_success() {
        let id = Uuid::new_v4();
        let svc = make_service(id);
        let mut mock = MockServiceRepository::new();
        mock.expect_find_by_id().returning(move |_| Ok(Some(svc.clone())));
        mock.expect_delete().returning(|_| Ok(()));

        let uc = DeleteServiceUseCase::new(Arc::new(mock));
        let result = uc.execute(id).await;
        assert!(result.is_ok());
    }

    /// 存在しないサービスは NotFound エラーを返す
    #[tokio::test]
    async fn test_delete_service_not_found() {
        let mut mock = MockServiceRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = DeleteServiceUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(matches!(result, Err(DeleteServiceError::NotFound(_))));
    }

    /// リポジトリエラーは Internal エラーに変換される
    #[tokio::test]
    async fn test_delete_service_repo_error() {
        let mut mock = MockServiceRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("connection refused")));

        let uc = DeleteServiceUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(matches!(result, Err(DeleteServiceError::Internal(_))));
    }
}
