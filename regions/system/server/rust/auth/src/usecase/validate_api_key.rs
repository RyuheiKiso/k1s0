use std::sync::Arc;

use chrono::Utc;

use crate::domain::repository::api_key_repository::ApiKeyRepository;

/// ValidateApiKeyError は API キー検証に関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum ValidateApiKeyError {
    #[error("invalid api key")]
    Invalid,

    #[error("api key revoked")]
    Revoked,

    #[error("api key expired")]
    Expired,

    #[error("internal error: {0}")]
    Internal(String),
}

/// ValidateApiKeyResult は検証成功時の結果。
#[derive(Debug, Clone)]
pub struct ValidateApiKeyResult {
    pub tenant_id: String,
    pub name: String,
    pub scopes: Vec<String>,
}

/// ValidateApiKeyUseCase は API キー検証ユースケース。
pub struct ValidateApiKeyUseCase {
    repo: Arc<dyn ApiKeyRepository>,
}

impl ValidateApiKeyUseCase {
    pub fn new(repo: Arc<dyn ApiKeyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        raw_key: &str,
    ) -> Result<ValidateApiKeyResult, ValidateApiKeyError> {
        if raw_key.len() < 13 {
            return Err(ValidateApiKeyError::Invalid);
        }

        let prefix = &raw_key[..13];
        let api_key = self
            .repo
            .find_by_prefix(prefix)
            .await
            .map_err(|e| ValidateApiKeyError::Internal(e.to_string()))?
            .ok_or(ValidateApiKeyError::Invalid)?;

        // verify hash matches
        let computed_hash = hash_key(raw_key);
        if computed_hash != api_key.key_hash {
            return Err(ValidateApiKeyError::Invalid);
        }

        if api_key.revoked {
            return Err(ValidateApiKeyError::Revoked);
        }

        if let Some(expires_at) = api_key.expires_at {
            if Utc::now() > expires_at {
                return Err(ValidateApiKeyError::Expired);
            }
        }

        Ok(ValidateApiKeyResult {
            tenant_id: api_key.tenant_id,
            name: api_key.name,
            scopes: api_key.scopes,
        })
    }
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
    use crate::domain::entity::api_key::ApiKey;
    use crate::domain::repository::api_key_repository::MockApiKeyRepository;
    use uuid::Uuid;

    fn make_api_key(raw_key: &str, revoked: bool, expired: bool) -> ApiKey {
        let now = Utc::now();
        let expires_at = if expired {
            Some(now - chrono::Duration::hours(1))
        } else {
            None
        };

        ApiKey {
            id: Uuid::new_v4(),
            tenant_id: "tenant-1".to_string(),
            name: "Test Key".to_string(),
            key_hash: hash_key(raw_key),
            prefix: raw_key[..13].to_string(),
            scopes: vec!["read".to_string()],
            expires_at,
            revoked,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn test_validate_api_key_success() {
        let raw_key = "k1s0_abcdef1234567890abcdef";
        let api_key = make_api_key(raw_key, false, false);

        let mut mock = MockApiKeyRepository::new();
        mock.expect_find_by_prefix()
            .withf(|p| p == "k1s0_abcdef12")
            .returning(move |_| Ok(Some(api_key.clone())));

        let uc = ValidateApiKeyUseCase::new(Arc::new(mock));
        let result = uc.execute(raw_key).await;
        assert!(result.is_ok());

        let val = result.unwrap();
        assert_eq!(val.tenant_id, "tenant-1");
        assert_eq!(val.scopes, vec!["read"]);
    }

    #[tokio::test]
    async fn test_validate_api_key_not_found() {
        let mut mock = MockApiKeyRepository::new();
        mock.expect_find_by_prefix().returning(|_| Ok(None));

        let uc = ValidateApiKeyUseCase::new(Arc::new(mock));
        let result = uc.execute("k1s0_nonexistent_key").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ValidateApiKeyError::Invalid => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_validate_api_key_revoked() {
        let raw_key = "k1s0_revoked_1234567890abc";
        let api_key = make_api_key(raw_key, true, false);

        let mut mock = MockApiKeyRepository::new();
        mock.expect_find_by_prefix()
            .returning(move |_| Ok(Some(api_key.clone())));

        let uc = ValidateApiKeyUseCase::new(Arc::new(mock));
        let result = uc.execute(raw_key).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ValidateApiKeyError::Revoked => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_validate_api_key_expired() {
        let raw_key = "k1s0_expired_1234567890abc";
        let api_key = make_api_key(raw_key, false, true);

        let mut mock = MockApiKeyRepository::new();
        mock.expect_find_by_prefix()
            .returning(move |_| Ok(Some(api_key.clone())));

        let uc = ValidateApiKeyUseCase::new(Arc::new(mock));
        let result = uc.execute(raw_key).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ValidateApiKeyError::Expired => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_validate_api_key_too_short() {
        let mock = MockApiKeyRepository::new();
        let uc = ValidateApiKeyUseCase::new(Arc::new(mock));
        let result = uc.execute("short").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ValidateApiKeyError::Invalid => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
