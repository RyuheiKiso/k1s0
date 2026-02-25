use std::sync::Arc;

use crate::domain::entity::RateLimitRule;
use crate::domain::repository::RateLimitRepository;

/// ListRulesError はルール一覧取得に関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum ListRulesError {
    #[error("internal error: {0}")]
    Internal(String),
}

/// ListRulesUseCase はルール一覧取得ユースケース。
pub struct ListRulesUseCase {
    repo: Arc<dyn RateLimitRepository>,
}

impl ListRulesUseCase {
    pub fn new(repo: Arc<dyn RateLimitRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self) -> Result<Vec<RateLimitRule>, ListRulesError> {
        self.repo
            .find_all()
            .await
            .map_err(|e| ListRulesError::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::Algorithm;
    use crate::domain::repository::rate_limit_repository::MockRateLimitRepository;

    #[tokio::test]
    async fn test_list_rules_success() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_all().returning(|| {
            Ok(vec![
                RateLimitRule::new(
                    "service".to_string(),
                    "pattern-1".to_string(),
                    100,
                    60,
                    Algorithm::TokenBucket,
                ),
                RateLimitRule::new(
                    "user".to_string(),
                    "pattern-2".to_string(),
                    200,
                    120,
                    Algorithm::FixedWindow,
                ),
            ])
        });

        let uc = ListRulesUseCase::new(Arc::new(repo));
        let result = uc.execute().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_list_rules_empty() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_all().returning(|| Ok(vec![]));

        let uc = ListRulesUseCase::new(Arc::new(repo));
        let result = uc.execute().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_list_rules_internal_error() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_all()
            .returning(|| Err(anyhow::anyhow!("db error")));

        let uc = ListRulesUseCase::new(Arc::new(repo));
        let result = uc.execute().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ListRulesError::Internal(msg) => assert!(msg.contains("db error")),
        }
    }
}
