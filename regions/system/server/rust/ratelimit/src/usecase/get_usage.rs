use std::sync::Arc;

use uuid::Uuid;

use crate::domain::repository::{RateLimitRepository, RateLimitStateStore};

/// GetUsageError はレートリミット使用状況取得に関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum GetUsageError {
    #[error("rule not found: {0}")]
    NotFound(String),

    #[error("invalid rule_id: {0}")]
    InvalidRuleId(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// UsageInfo はレートリミットの使用状況。
#[derive(Debug, Clone, serde::Serialize)]
pub struct UsageInfo {
    pub rule_id: String,
    pub rule_name: String,
    pub limit: i64,
    pub window_seconds: i64,
    pub algorithm: String,
    pub enabled: bool,
    pub used: Option<i64>,
    pub remaining: Option<i64>,
    pub reset_at: Option<i64>,
}

/// GetUsageUseCase はレートリミット使用状況取得ユースケース。
pub struct GetUsageUseCase {
    rule_repo: Arc<dyn RateLimitRepository>,
    state_store: Option<Arc<dyn RateLimitStateStore>>,
}

impl GetUsageUseCase {
    pub fn new(rule_repo: Arc<dyn RateLimitRepository>) -> Self {
        Self {
            rule_repo,
            state_store: None,
        }
    }

    pub fn with_state_store(
        rule_repo: Arc<dyn RateLimitRepository>,
        state_store: Arc<dyn RateLimitStateStore>,
    ) -> Self {
        Self {
            rule_repo,
            state_store: Some(state_store),
        }
    }

    pub async fn execute(&self, rule_id: &str) -> Result<UsageInfo, GetUsageError> {
        let id = Uuid::parse_str(rule_id)
            .map_err(|_| GetUsageError::InvalidRuleId(rule_id.to_string()))?;

        let rule = self
            .rule_repo
            .find_by_id(&id)
            .await
            .map_err(|e| GetUsageError::NotFound(e.to_string()))?;

        let key = format!("ratelimit:{}:{}", rule.scope, rule.identifier_pattern);
        let (used, remaining, reset_at) = if let Some(ref store) = self.state_store {
            match store.get_usage(&key, rule.limit, rule.window_seconds).await {
                Ok(Some(snapshot)) => (
                    Some(snapshot.used),
                    Some(snapshot.remaining),
                    Some(snapshot.reset_at),
                ),
                _ => (None, None, None),
            }
        } else {
            (None, None, None)
        };

        Ok(UsageInfo {
            rule_id: rule.id.to_string(),
            rule_name: rule.scope.clone(),
            limit: rule.limit,
            window_seconds: rule.window_seconds,
            algorithm: rule.algorithm.as_str().to_string(),
            enabled: rule.enabled,
            used,
            remaining,
            reset_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{Algorithm, RateLimitRule};
    use crate::domain::repository::rate_limit_repository::MockRateLimitRepository;

    #[tokio::test]
    async fn test_get_usage_success() {
        let rule = RateLimitRule::new(
            "service".to_string(),
            "global".to_string(),
            100,
            60,
            Algorithm::TokenBucket,
        );
        let rule_id = rule.id;
        let return_rule = rule.clone();

        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(return_rule.clone()));

        let uc = GetUsageUseCase::new(Arc::new(repo));
        let result = uc.execute(&rule_id.to_string()).await;
        assert!(result.is_ok());

        let info = result.unwrap();
        assert_eq!(info.rule_name, "service");
        assert_eq!(info.limit, 100);
        assert!(info.used.is_none());
        assert!(info.remaining.is_none());
        assert!(info.reset_at.is_none());
    }

    #[tokio::test]
    async fn test_get_usage_not_found() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("not found")));

        let uc = GetUsageUseCase::new(Arc::new(repo));
        let result = uc
            .execute("550e8400-e29b-41d4-a716-446655440000")
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GetUsageError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_get_usage_invalid_uuid() {
        let repo = MockRateLimitRepository::new();
        let uc = GetUsageUseCase::new(Arc::new(repo));
        let result = uc.execute("not-a-uuid").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GetUsageError::InvalidRuleId(_)
        ));
    }
}
