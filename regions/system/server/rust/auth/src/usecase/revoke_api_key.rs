use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::api_key_repository::ApiKeyRepository;

/// RevokeApiKeyError は API キー失効に関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum RevokeApiKeyError {
    #[error("api key not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// RevokeApiKeyUseCase は API キー失効ユースケース。
pub struct RevokeApiKeyUseCase {
    repo: Arc<dyn ApiKeyRepository>,
}

impl RevokeApiKeyUseCase {
    pub fn new(repo: Arc<dyn ApiKeyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<(), RevokeApiKeyError> {
        self.repo.revoke(id).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                RevokeApiKeyError::NotFound(id.to_string())
            } else {
                RevokeApiKeyError::Internal(msg)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::api_key_repository::MockApiKeyRepository;

    #[tokio::test]
    async fn test_revoke_api_key_success() {
        let mut mock = MockApiKeyRepository::new();
        mock.expect_revoke().returning(|_| Ok(()));

        let uc = RevokeApiKeyUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_revoke_api_key_not_found() {
        let mut mock = MockApiKeyRepository::new();
        mock.expect_revoke()
            .returning(|_| Err(anyhow::anyhow!("api key not found")));

        let uc = RevokeApiKeyUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            RevokeApiKeyError::NotFound(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
