use std::sync::Arc;

use crate::domain::repository::RateLimitStateStore;

/// ResetRateLimitInput はレートリミットリセットの入力。
pub struct ResetRateLimitInput {
    /// STATIC-CRITICAL-001: テナントスコープでリセット対象を特定する
    pub tenant_id: String,
    pub scope: String,
    pub identifier: String,
}

/// ResetRateLimitError はレートリミットリセットに関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum ResetRateLimitError {
    #[error("validation error: {0}")]
    ValidationError(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// ResetRateLimitUseCase はレートリミットリセットユースケース。
pub struct ResetRateLimitUseCase {
    state_store: Arc<dyn RateLimitStateStore>,
}

impl ResetRateLimitUseCase {
    pub fn new(state_store: Arc<dyn RateLimitStateStore>) -> Self {
        Self { state_store }
    }

    /// STATIC-CRITICAL-001 監査対応: テナントスコープのレートリミット状態をリセットする。
    pub async fn execute(&self, input: &ResetRateLimitInput) -> Result<(), ResetRateLimitError> {
        if input.scope.is_empty() {
            return Err(ResetRateLimitError::ValidationError(
                "scope is required".to_string(),
            ));
        }
        if input.identifier.is_empty() {
            return Err(ResetRateLimitError::ValidationError(
                "identifier is required".to_string(),
            ));
        }

        // Redis キー: ratelimit:{tenant_id}:{scope}:{identifier}
        // STATIC-CRITICAL-001: テナントIDプレフィックスでテナント間のレートリミット状態を分離する
        let key = format!(
            "ratelimit:{}:{}:{}",
            input.tenant_id, input.scope, input.identifier
        );
        self.state_store
            .reset(&key)
            .await
            .map_err(|e| ResetRateLimitError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::rate_limit_repository::MockRateLimitStateStore;

    /// システムテナントUUID: テスト共通
    const SYSTEM_TENANT: &str = "00000000-0000-0000-0000-000000000001";

    #[tokio::test]
    async fn test_reset_rate_limit_success() {
        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_reset()
            // STATIC-CRITICAL-001: テナントIDプレフィックスが正しく使われることを確認する
            .withf(|key| {
                key == "ratelimit:00000000-0000-0000-0000-000000000001:service:user-123"
            })
            .once()
            .returning(|_| Ok(()));

        let uc = ResetRateLimitUseCase::new(Arc::new(state_store));
        let result = uc
            .execute(&ResetRateLimitInput {
                tenant_id: SYSTEM_TENANT.to_string(),
                scope: "service".to_string(),
                identifier: "user-123".to_string(),
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reset_rate_limit_empty_scope_error() {
        let state_store = MockRateLimitStateStore::new();
        let uc = ResetRateLimitUseCase::new(Arc::new(state_store));
        let result = uc
            .execute(&ResetRateLimitInput {
                tenant_id: SYSTEM_TENANT.to_string(),
                scope: "".to_string(),
                identifier: "user-123".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResetRateLimitError::ValidationError(_)
        ));
    }
}
