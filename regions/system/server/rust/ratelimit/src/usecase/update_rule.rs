use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::{Algorithm, RateLimitRule};
use crate::domain::repository::RateLimitRepository;

/// UpdateRuleError はルール更新に関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum UpdateRuleError {
    #[error("rule not found: {0}")]
    NotFound(String),

    #[error("invalid algorithm: {0}")]
    InvalidAlgorithm(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// UpdateRuleInput はルール更新の入力。
pub struct UpdateRuleInput {
    pub id: String,
    pub name: String,
    pub key: String,
    pub limit: i64,
    pub window_secs: i64,
    pub algorithm: String,
    pub enabled: bool,
}

/// UpdateRuleUseCase はルール更新ユースケース。
pub struct UpdateRuleUseCase {
    repo: Arc<dyn RateLimitRepository>,
}

impl UpdateRuleUseCase {
    pub fn new(repo: Arc<dyn RateLimitRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &UpdateRuleInput) -> Result<RateLimitRule, UpdateRuleError> {
        if input.name.is_empty() {
            return Err(UpdateRuleError::Validation("name is required".to_string()));
        }
        if input.limit <= 0 {
            return Err(UpdateRuleError::Validation(
                "limit must be positive".to_string(),
            ));
        }
        if input.window_secs <= 0 {
            return Err(UpdateRuleError::Validation(
                "window_secs must be positive".to_string(),
            ));
        }

        let algorithm = Algorithm::from_str(&input.algorithm)
            .map_err(|e| UpdateRuleError::InvalidAlgorithm(e))?;

        let id = Uuid::parse_str(&input.id)
            .map_err(|_| UpdateRuleError::NotFound(input.id.clone()))?;

        let mut rule = self
            .repo
            .find_by_id(&id)
            .await
            .map_err(|e| UpdateRuleError::NotFound(e.to_string()))?;

        rule.name = input.name.clone();
        rule.key = input.key.clone();
        rule.limit = input.limit;
        rule.window_secs = input.window_secs;
        rule.algorithm = algorithm;
        rule.enabled = input.enabled;
        rule.updated_at = chrono::Utc::now();

        self.repo
            .update(&rule)
            .await
            .map_err(|e| UpdateRuleError::Internal(e.to_string()))?;

        Ok(rule)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::rate_limit_repository::MockRateLimitRepository;

    #[tokio::test]
    async fn test_update_rule_success() {
        let mut repo = MockRateLimitRepository::new();
        let rule = RateLimitRule::new(
            "api-global".to_string(),
            "global".to_string(),
            100,
            60,
            Algorithm::TokenBucket,
        );
        let rule_id = rule.id;
        let return_rule = rule.clone();

        repo.expect_find_by_id()
            .returning(move |_| Ok(return_rule.clone()));
        repo.expect_update().returning(|_| Ok(()));

        let uc = UpdateRuleUseCase::new(Arc::new(repo));
        let input = UpdateRuleInput {
            id: rule_id.to_string(),
            name: "updated-rule".to_string(),
            key: "updated-key".to_string(),
            limit: 200,
            window_secs: 120,
            algorithm: "fixed_window".to_string(),
            enabled: false,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.name, "updated-rule");
        assert_eq!(updated.limit, 200);
        assert_eq!(updated.algorithm, Algorithm::FixedWindow);
        assert!(!updated.enabled);
    }

    #[tokio::test]
    async fn test_update_rule_not_found() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("not found")));

        let uc = UpdateRuleUseCase::new(Arc::new(repo));
        let input = UpdateRuleInput {
            id: Uuid::new_v4().to_string(),
            name: "test".to_string(),
            key: "test".to_string(),
            limit: 100,
            window_secs: 60,
            algorithm: "token_bucket".to_string(),
            enabled: true,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UpdateRuleError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_update_rule_invalid_algorithm() {
        let repo = MockRateLimitRepository::new();
        let uc = UpdateRuleUseCase::new(Arc::new(repo));
        let input = UpdateRuleInput {
            id: Uuid::new_v4().to_string(),
            name: "test".to_string(),
            key: "test".to_string(),
            limit: 100,
            window_secs: 60,
            algorithm: "bad_algo".to_string(),
            enabled: true,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UpdateRuleError::InvalidAlgorithm(_)
        ));
    }
}
