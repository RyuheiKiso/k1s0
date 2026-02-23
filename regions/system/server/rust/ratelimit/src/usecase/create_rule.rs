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
    pub name: String,
    pub key: String,
    pub limit: i64,
    pub window_secs: i64,
    pub algorithm: String,
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
        if input.name.is_empty() {
            return Err(CreateRuleError::Validation("name is required".to_string()));
        }
        if input.key.is_empty() {
            return Err(CreateRuleError::Validation("key is required".to_string()));
        }
        if input.limit <= 0 {
            return Err(CreateRuleError::Validation(
                "limit must be positive".to_string(),
            ));
        }
        if input.window_secs <= 0 {
            return Err(CreateRuleError::Validation(
                "window_secs must be positive".to_string(),
            ));
        }

        let algorithm = Algorithm::from_str(&input.algorithm)
            .map_err(|e| CreateRuleError::InvalidAlgorithm(e))?;

        // 重複チェック
        if let Ok(Some(_)) = self.repo.find_by_name(&input.name).await {
            return Err(CreateRuleError::AlreadyExists(input.name.clone()));
        }

        let rule = RateLimitRule::new(
            input.name.clone(),
            input.key.clone(),
            input.limit,
            input.window_secs,
            algorithm,
        );

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
                name: "api-global".to_string(),
                key: "global".to_string(),
                limit: 100,
                window_secs: 60,
                algorithm: "token_bucket".to_string(),
            })
            .await;

        assert!(result.is_ok());
        let rule = result.unwrap();
        assert_eq!(rule.name, "api-global");
        assert_eq!(rule.key, "global");
        assert_eq!(rule.limit, 100);
        assert_eq!(rule.window_secs, 60);
        assert_eq!(rule.algorithm, Algorithm::TokenBucket);
        assert!(rule.enabled);
    }

    #[tokio::test]
    async fn test_create_rule_duplicate_name() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_name().returning(|_| {
            Ok(Some(RateLimitRule::new(
                "api-global".to_string(),
                "global".to_string(),
                100,
                60,
                Algorithm::TokenBucket,
            )))
        });

        let uc = CreateRuleUseCase::new(Arc::new(repo));
        let result = uc
            .execute(&CreateRuleInput {
                name: "api-global".to_string(),
                key: "global".to_string(),
                limit: 100,
                window_secs: 60,
                algorithm: "token_bucket".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CreateRuleError::AlreadyExists(_)));
    }

    #[tokio::test]
    async fn test_create_rule_invalid_algorithm() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_name().returning(|_| Ok(None));

        let uc = CreateRuleUseCase::new(Arc::new(repo));
        let result = uc
            .execute(&CreateRuleInput {
                name: "test".to_string(),
                key: "test".to_string(),
                limit: 100,
                window_secs: 60,
                algorithm: "unknown_algo".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateRuleError::InvalidAlgorithm(_)
        ));
    }

    #[tokio::test]
    async fn test_create_rule_empty_name() {
        let repo = MockRateLimitRepository::new();
        let uc = CreateRuleUseCase::new(Arc::new(repo));
        let result = uc
            .execute(&CreateRuleInput {
                name: "".to_string(),
                key: "test".to_string(),
                limit: 100,
                window_secs: 60,
                algorithm: "token_bucket".to_string(),
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
                name: "test".to_string(),
                key: "test".to_string(),
                limit: 0,
                window_secs: 60,
                algorithm: "token_bucket".to_string(),
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
                name: "test".to_string(),
                key: "test".to_string(),
                limit: 100,
                window_secs: -1,
                algorithm: "token_bucket".to_string(),
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CreateRuleError::Validation(_)));
    }
}
