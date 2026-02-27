use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::api_key::ApiKeySummary;
use crate::domain::repository::api_key_repository::ApiKeyRepository;

/// GetApiKeyError は API キー取得に関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum GetApiKeyError {
    #[error("api key not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// GetApiKeyUseCase は API キー取得ユースケース。
pub struct GetApiKeyUseCase {
    repo: Arc<dyn ApiKeyRepository>,
}

impl GetApiKeyUseCase {
    pub fn new(repo: Arc<dyn ApiKeyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<ApiKeySummary, GetApiKeyError> {
        match self.repo.find_by_id(id).await {
            Ok(Some(key)) => Ok(ApiKeySummary::from(&key)),
            Ok(None) => Err(GetApiKeyError::NotFound(id.to_string())),
            Err(e) => Err(GetApiKeyError::Internal(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::api_key::ApiKey;
    use crate::domain::repository::api_key_repository::MockApiKeyRepository;
    use chrono::Utc;

    #[tokio::test]
    async fn test_get_api_key_success() {
        let mut mock = MockApiKeyRepository::new();
        let id = Uuid::new_v4();
        let expected_id = id;

        mock.expect_find_by_id()
            .withf(move |i| *i == expected_id)
            .returning(move |_| {
                let now = Utc::now();
                Ok(Some(ApiKey {
                    id,
                    tenant_id: "tenant-1".to_string(),
                    name: "Test Key".to_string(),
                    key_hash: "hash".to_string(),
                    prefix: "k1s0_ab12".to_string(),
                    scopes: vec!["read".to_string()],
                    expires_at: None,
                    revoked: false,
                    created_at: now,
                    updated_at: now,
                }))
            });

        let uc = GetApiKeyUseCase::new(Arc::new(mock));
        let result = uc.execute(expected_id).await;
        assert!(result.is_ok());

        let summary = result.unwrap();
        assert_eq!(summary.name, "Test Key");
    }

    #[tokio::test]
    async fn test_get_api_key_not_found() {
        let mut mock = MockApiKeyRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetApiKeyUseCase::new(Arc::new(mock));
        let result = uc.execute(Uuid::new_v4()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            GetApiKeyError::NotFound(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
