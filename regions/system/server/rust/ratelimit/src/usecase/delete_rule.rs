use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::RateLimitRepository;

/// DeleteRuleError はルール削除に関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum DeleteRuleError {
    #[error("rule not found: {0}")]
    NotFound(String),

    #[error("invalid rule_id: {0}")]
    InvalidRuleId(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// DeleteRuleUseCase はルール削除ユースケース。
pub struct DeleteRuleUseCase {
    repo: Arc<dyn RateLimitRepository>,
}

impl DeleteRuleUseCase {
    pub fn new(repo: Arc<dyn RateLimitRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, rule_id: &str) -> Result<(), DeleteRuleError> {
        let id = Uuid::parse_str(rule_id)
            .map_err(|_| DeleteRuleError::InvalidRuleId(rule_id.to_string()))?;

        let deleted = self
            .repo
            .delete(&id)
            .await
            .map_err(|e| DeleteRuleError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteRuleError::NotFound(rule_id.to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::rate_limit_repository::MockRateLimitRepository;

    #[tokio::test]
    async fn test_delete_rule_success() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_delete().returning(|_| Ok(true));

        let uc = DeleteRuleUseCase::new(Arc::new(repo));
        let result = uc
            .execute("550e8400-e29b-41d4-a716-446655440000")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_rule_not_found() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_delete().returning(|_| Ok(false));

        let uc = DeleteRuleUseCase::new(Arc::new(repo));
        let result = uc
            .execute("550e8400-e29b-41d4-a716-446655440000")
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DeleteRuleError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_delete_rule_invalid_uuid() {
        let repo = MockRateLimitRepository::new();
        let uc = DeleteRuleUseCase::new(Arc::new(repo));
        let result = uc.execute("not-a-uuid").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DeleteRuleError::InvalidRuleId(_)
        ));
    }
}
