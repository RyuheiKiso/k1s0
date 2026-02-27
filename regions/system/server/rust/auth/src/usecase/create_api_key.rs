use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::entity::api_key::{ApiKey, CreateApiKeyRequest, CreateApiKeyResponse};
use crate::domain::repository::api_key_repository::ApiKeyRepository;

/// CreateApiKeyError は API キー作成に関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum CreateApiKeyError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// CreateApiKeyUseCase は API キー作成ユースケース。
pub struct CreateApiKeyUseCase {
    repo: Arc<dyn ApiKeyRepository>,
}

impl CreateApiKeyUseCase {
    pub fn new(repo: Arc<dyn ApiKeyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        req: CreateApiKeyRequest,
    ) -> Result<CreateApiKeyResponse, CreateApiKeyError> {
        if req.name.is_empty() {
            return Err(CreateApiKeyError::Validation(
                "name is required".to_string(),
            ));
        }
        if req.tenant_id.is_empty() {
            return Err(CreateApiKeyError::Validation(
                "tenant_id is required".to_string(),
            ));
        }

        let id = Uuid::new_v4();
        let raw_key = format!("k1s0_{}", generate_random_key());
        let prefix = raw_key[..13].to_string();
        let key_hash = hash_key(&raw_key);
        let now = Utc::now();

        let api_key = ApiKey {
            id,
            tenant_id: req.tenant_id,
            name: req.name.clone(),
            key_hash,
            prefix: prefix.clone(),
            scopes: req.scopes.clone(),
            expires_at: req.expires_at,
            revoked: false,
            created_at: now,
            updated_at: now,
        };

        self.repo
            .create(&api_key)
            .await
            .map_err(|e| CreateApiKeyError::Internal(e.to_string()))?;

        Ok(CreateApiKeyResponse {
            id,
            name: req.name,
            prefix,
            raw_key,
            scopes: req.scopes,
            expires_at: req.expires_at,
            created_at: now,
        })
    }
}

fn generate_random_key() -> String {
    use std::fmt::Write;
    let bytes: [u8; 24] = {
        let mut buf = [0u8; 24];
        // Use uuid v4 randomness as source
        let u1 = Uuid::new_v4();
        let u2 = Uuid::new_v4();
        buf[..16].copy_from_slice(u1.as_bytes());
        buf[16..].copy_from_slice(&u2.as_bytes()[..8]);
        buf
    };
    let mut s = String::with_capacity(48);
    for b in &bytes {
        write!(s, "{:02x}", b).unwrap();
    }
    s
}

fn hash_key(raw_key: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    raw_key.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::api_key_repository::MockApiKeyRepository;

    #[tokio::test]
    async fn test_create_api_key_success() {
        let mut mock = MockApiKeyRepository::new();
        mock.expect_create().returning(|_| Ok(()));

        let uc = CreateApiKeyUseCase::new(Arc::new(mock));
        let req = CreateApiKeyRequest {
            tenant_id: "tenant-1".to_string(),
            name: "My Key".to_string(),
            scopes: vec!["read".to_string()],
            expires_at: None,
        };

        let result = uc.execute(req).await;
        assert!(result.is_ok());

        let resp = result.unwrap();
        assert_eq!(resp.name, "My Key");
        assert!(resp.raw_key.starts_with("k1s0_"));
        assert!(!resp.prefix.is_empty());
    }

    #[tokio::test]
    async fn test_create_api_key_empty_name() {
        let mock = MockApiKeyRepository::new();
        let uc = CreateApiKeyUseCase::new(Arc::new(mock));

        let req = CreateApiKeyRequest {
            tenant_id: "tenant-1".to_string(),
            name: String::new(),
            scopes: vec![],
            expires_at: None,
        };

        let result = uc.execute(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateApiKeyError::Validation(msg) => assert!(msg.contains("name")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_create_api_key_empty_tenant() {
        let mock = MockApiKeyRepository::new();
        let uc = CreateApiKeyUseCase::new(Arc::new(mock));

        let req = CreateApiKeyRequest {
            tenant_id: String::new(),
            name: "Key".to_string(),
            scopes: vec![],
            expires_at: None,
        };

        let result = uc.execute(req).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            CreateApiKeyError::Validation(msg) => assert!(msg.contains("tenant_id")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
