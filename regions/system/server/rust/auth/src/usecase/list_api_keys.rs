use std::sync::Arc;

use crate::domain::entity::api_key::ApiKeySummary;
use crate::domain::repository::api_key_repository::ApiKeyRepository;

/// ListApiKeysError は API キー一覧取得に関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum ListApiKeysError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// ListApiKeysUseCase は テナントの API キー一覧取得ユースケース。
pub struct ListApiKeysUseCase {
    repo: Arc<dyn ApiKeyRepository>,
}

impl ListApiKeysUseCase {
    pub fn new(repo: Arc<dyn ApiKeyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<ApiKeySummary>, ListApiKeysError> {
        if tenant_id.is_empty() {
            return Err(ListApiKeysError::Validation(
                "tenant_id is required".to_string(),
            ));
        }

        let keys = self
            .repo
            .list_by_tenant(tenant_id)
            .await
            .map_err(|e| ListApiKeysError::Internal(e.to_string()))?;

        Ok(keys.iter().map(ApiKeySummary::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::api_key::ApiKey;
    use crate::domain::repository::api_key_repository::MockApiKeyRepository;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_list_api_keys_success() {
        let mut mock = MockApiKeyRepository::new();
        mock.expect_list_by_tenant()
            .withf(|t| t == "tenant-1")
            .returning(|_| {
                let now = Utc::now();
                Ok(vec![ApiKey {
                    id: Uuid::new_v4(),
                    tenant_id: "tenant-1".to_string(),
                    name: "Key 1".to_string(),
                    key_hash: "hash".to_string(),
                    prefix: "k1s0_ab12".to_string(),
                    scopes: vec!["read".to_string()],
                    expires_at: None,
                    revoked: false,
                    created_at: now,
                    updated_at: now,
                }])
            });

        let uc = ListApiKeysUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant-1").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_list_api_keys_empty_tenant() {
        let mock = MockApiKeyRepository::new();
        let uc = ListApiKeysUseCase::new(Arc::new(mock));
        let result = uc.execute("").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ListApiKeysError::Validation(msg) => assert!(msg.contains("tenant_id")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
