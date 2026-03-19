use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::{Algorithm, RateLimitRule};
use crate::domain::repository::RateLimitRepository;
use crate::domain::service::RateLimitDomainService;

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
    pub scope: String,
    pub identifier_pattern: String,
    pub limit: u32,
    pub window_seconds: u32,
    pub algorithm: Option<String>,
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
        RateLimitDomainService::validate_rule_input(
            &input.scope,
            &input.identifier_pattern,
            input.limit,
            input.window_seconds,
        )
        .map_err(UpdateRuleError::Validation)?;

        let id =
            Uuid::parse_str(&input.id).map_err(|_| UpdateRuleError::NotFound(input.id.clone()))?;

        let mut rule = self
            .repo
            .find_by_id(&id)
            .await
            .map_err(|e| UpdateRuleError::NotFound(e.to_string()))?;

        rule.name = format!("{}:{}", input.scope, input.identifier_pattern);
        rule.scope = input.scope.clone();
        rule.identifier_pattern = input.identifier_pattern.clone();
        rule.limit = input.limit;
        rule.window_seconds = input.window_seconds;
        if let Some(ref algorithm) = input.algorithm {
            rule.algorithm =
                Algorithm::from_str(algorithm).map_err(UpdateRuleError::InvalidAlgorithm)?;
        }
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::Algorithm;
    use crate::domain::repository::rate_limit_repository::MockRateLimitRepository;

    #[tokio::test]
    async fn test_update_rule_success() {
        let mut repo = MockRateLimitRepository::new();
        let rule = RateLimitRule::new(
            "service".to_string(),
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
            scope: "user".to_string(),
            identifier_pattern: "updated-pattern".to_string(),
            limit: 200,
            window_seconds: 120,
            algorithm: Some("fixed_window".to_string()),
            enabled: false,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.scope, "user");
        assert_eq!(updated.limit, 200);
        assert_eq!(updated.window_seconds, 120);
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
            scope: "service".to_string(),
            identifier_pattern: "test".to_string(),
            limit: 100,
            window_seconds: 60,
            algorithm: None,
            enabled: true,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UpdateRuleError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_update_rule_empty_scope() {
        let repo = MockRateLimitRepository::new();
        let uc = UpdateRuleUseCase::new(Arc::new(repo));
        let input = UpdateRuleInput {
            id: Uuid::new_v4().to_string(),
            scope: "".to_string(),
            identifier_pattern: "test".to_string(),
            limit: 100,
            window_seconds: 60,
            algorithm: None,
            enabled: true,
        };

        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UpdateRuleError::Validation(_)
        ));
    }
}
