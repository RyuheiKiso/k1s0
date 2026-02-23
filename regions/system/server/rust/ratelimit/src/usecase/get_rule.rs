use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::RateLimitRule;
use crate::domain::repository::RateLimitRepository;

/// GetRuleError はルール取得に関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum GetRuleError {
    #[error("rule not found: {0}")]
    NotFound(String),

    #[error("invalid rule_id: {0}")]
    InvalidRuleId(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// GetRuleUseCase はルール取得ユースケース。
pub struct GetRuleUseCase {
    repo: Arc<dyn RateLimitRepository>,
}

impl GetRuleUseCase {
    pub fn new(repo: Arc<dyn RateLimitRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, rule_id: &str) -> Result<RateLimitRule, GetRuleError> {
        let id = Uuid::parse_str(rule_id)
            .map_err(|_| GetRuleError::InvalidRuleId(rule_id.to_string()))?;

        let rule = self
            .repo
            .find_by_id(&id)
            .await
            .map_err(|e| GetRuleError::NotFound(e.to_string()))?;

        Ok(rule)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{Algorithm, RateLimitRule};
    use crate::domain::repository::rate_limit_repository::MockRateLimitRepository;

    #[tokio::test]
    async fn test_get_rule_success() {
        let rule = RateLimitRule::new(
            "api-global".to_string(),
            "global".to_string(),
            100,
            60,
            Algorithm::TokenBucket,
        );
        let rule_id = rule.id;

        let mut repo = MockRateLimitRepository::new();
        let return_rule = rule.clone();
        repo.expect_find_by_id()
            .returning(move |_| Ok(return_rule.clone()));

        let uc = GetRuleUseCase::new(Arc::new(repo));
        let result = uc.execute(&rule_id.to_string()).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.id, rule_id);
        assert_eq!(found.name, "api-global");
    }

    #[tokio::test]
    async fn test_get_rule_not_found() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("not found")));

        let uc = GetRuleUseCase::new(Arc::new(repo));
        let result = uc
            .execute("550e8400-e29b-41d4-a716-446655440000")
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GetRuleError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_get_rule_invalid_uuid() {
        let repo = MockRateLimitRepository::new();
        let uc = GetRuleUseCase::new(Arc::new(repo));
        let result = uc.execute("invalid-uuid").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GetRuleError::InvalidRuleId(_)));
    }
}
