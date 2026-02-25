use std::sync::Arc;

use crate::domain::entity::{Algorithm, RateLimitRule};
use crate::domain::repository::RateLimitRepository;

/// CreateRuleError はルール作成に関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum CreateRuleError {
    #[error("invalid algorithm: {0}")]
    InvalidAlgorithm(String),

    #[error("rule already exists: {0}")]
    AlreadyExists(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// CreateRuleInput はルール作成の入力。
pub struct CreateRuleInput {
    pub scope: String,
    pub identifier_pattern: String,
    pub limit: i64,
    pub window_seconds: i64,
    pub enabled: bool,
}

/// CreateRuleUseCase はルール作成ユースケース。
pub struct CreateRuleUseCase {
    repo: Arc<dyn RateLimitRepository>,
}

impl CreateRuleUseCase {
    pub fn new(repo: Arc<dyn RateLimitRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &CreateRuleInput) -> Result<RateLimitRule, CreateRuleError> {
        // バリデーション
        if input.scope.is_empty() {
            return Err(CreateRuleError::Validation("scope is required".to_string()));
        }
        if input.identifier_pattern.is_empty() {
            return Err(CreateRuleError::Validation("identifier_pattern is required".to_string()));
        }
        if input.limit <= 0 {
            return Err(CreateRuleError::Validation(
                "limit must be positive".to_string(),
            ));
        }
        if input.window_seconds <= 0 {
            return Err(CreateRuleError::Validation(
                "window_seconds must be positive".to_string(),
            ));
        }

        // 重複チェック（scope+identifier_patternで確認）
        if let Ok(Some(_)) = self.repo.find_by_name(&input.scope).await {
            return Err(CreateRuleError::AlreadyExists(input.scope.clone()));
        }

        let mut rule = RateLimitRule::new(
            input.scope.clone(),
            input.identifier_pattern.clone(),
            input.limit,
            input.window_seconds,
            Algorithm::TokenBucket,
        );
        rule.enabled = input.enabled;

        let created = self
            .repo
            .create(&rule)
            .await
            .map_err(|e| CreateRuleError::Internal(e.to_string()))?;

        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::rate_limit_repository::MockRateLimitRepository;

    #[tokio::test]
    async fn test_create_rule_success() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_name().returning(|_| Ok(None));
        repo.expect_create()
            .returning(|rule| Ok(rule.clone()));

        let uc = CreateRuleUseCase::new(Arc::new(repo));
        let result = uc
            .execute(&CreateRuleInput {
                scope: "service".to_string(),
                identifier_pattern: "global".to_string(),
                limit: 100,
                window_seconds: 60,
                enabled: true,
            })
            .await;

        assert!(result.is_ok());
        let rule = result.unwrap();
        assert_eq!(rule.scope, "service");
        assert_eq!(rule.identifier_pattern, "global");
        assert_eq!(rule.limit, 100);
        assert_eq!(rule.window_seconds, 60);
        assert_eq!(rule.algorithm, Algorithm::TokenBucket);
        assert!(rule.enabled);
    }

    #[tokio::test]
    async fn test_create_rule_duplicate_scope() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_name().returning(|_| {
            Ok(Some(RateLimitRule::new(
                "service".to_string(),
                "global".to_string(),
                100,
                60,
                Algorithm::TokenBucket,
            )))
        });

        let uc = CreateRuleUseCase::new(Arc::new(repo));
        let result = uc
            .execute(&CreateRuleInput {
                scope: "service".to_string(),
                identifier_pattern: "global".to_string(),
                limit: 100,
                window_seconds: 60,
                enabled: true,
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CreateRuleError::AlreadyExists(_)));
    }

    #[tokio::test]
    async fn test_create_rule_empty_scope() {
        let repo = MockRateLimitRepository::new();
        let uc = CreateRuleUseCase::new(Arc::new(repo));
        let result = uc
            .execute(&CreateRuleInput {
                scope: "".to_string(),
                identifier_pattern: "test".to_string(),
                limit: 100,
                window_seconds: 60,
                enabled: true,
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CreateRuleError::Validation(_)));
    }

    #[tokio::test]
    async fn test_create_rule_invalid_limit() {
        let repo = MockRateLimitRepository::new();
        let uc = CreateRuleUseCase::new(Arc::new(repo));
        let result = uc
            .execute(&CreateRuleInput {
                scope: "service".to_string(),
                identifier_pattern: "test".to_string(),
                limit: 0,
                window_seconds: 60,
                enabled: true,
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CreateRuleError::Validation(_)));
    }

    #[tokio::test]
    async fn test_create_rule_invalid_window() {
        let repo = MockRateLimitRepository::new();
        let uc = CreateRuleUseCase::new(Arc::new(repo));
        let result = uc
            .execute(&CreateRuleInput {
                scope: "service".to_string(),
                identifier_pattern: "test".to_string(),
                limit: 100,
                window_seconds: -1,
                enabled: true,
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CreateRuleError::Validation(_)));
    }
}
