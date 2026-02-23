use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::{Algorithm, RateLimitDecision};
use crate::domain::repository::{RateLimitRepository, RateLimitStateStore};

/// CheckRateLimitError はレートリミットチェックに関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum CheckRateLimitError {
    #[error("rule not found: {0}")]
    RuleNotFound(String),

    #[error("rule disabled: {0}")]
    RuleDisabled(String),

    #[error("invalid rule_id: {0}")]
    InvalidRuleId(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// CheckRateLimitUseCase はレートリミットチェックユースケース。
pub struct CheckRateLimitUseCase {
    rule_repo: Arc<dyn RateLimitRepository>,
    state_store: Arc<dyn RateLimitStateStore>,
}

impl CheckRateLimitUseCase {
    pub fn new(
        rule_repo: Arc<dyn RateLimitRepository>,
        state_store: Arc<dyn RateLimitStateStore>,
    ) -> Self {
        Self {
            rule_repo,
            state_store,
        }
    }

    pub async fn execute(
        &self,
        rule_id: &str,
        subject: &str,
    ) -> Result<RateLimitDecision, CheckRateLimitError> {
        let id = Uuid::parse_str(rule_id)
            .map_err(|_| CheckRateLimitError::InvalidRuleId(rule_id.to_string()))?;

        let rule = self
            .rule_repo
            .find_by_id(&id)
            .await
            .map_err(|e| CheckRateLimitError::RuleNotFound(e.to_string()))?;

        if !rule.enabled {
            return Err(CheckRateLimitError::RuleDisabled(rule.name.clone()));
        }

        // Redis キー: ratelimit:{rule_key}:{subject}
        let redis_key = format!("ratelimit:{}:{}", rule.key, subject);

        let decision = match rule.algorithm {
            Algorithm::TokenBucket => {
                self.state_store
                    .check_token_bucket(&redis_key, rule.limit, rule.window_secs)
                    .await
            }
            Algorithm::FixedWindow => {
                self.state_store
                    .check_fixed_window(&redis_key, rule.limit, rule.window_secs)
                    .await
            }
            Algorithm::SlidingWindow => {
                self.state_store
                    .check_sliding_window(&redis_key, rule.limit, rule.window_secs)
                    .await
            }
        }
        .map_err(|e| CheckRateLimitError::Internal(e.to_string()))?;

        Ok(decision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::RateLimitRule;
    use crate::domain::repository::rate_limit_repository::{
        MockRateLimitRepository, MockRateLimitStateStore,
    };

    fn make_rule() -> RateLimitRule {
        RateLimitRule::new(
            "api-global".to_string(),
            "global".to_string(),
            100,
            60,
            Algorithm::TokenBucket,
        )
    }

    #[tokio::test]
    async fn test_check_rate_limit_allowed() {
        let rule = make_rule();
        let rule_id = rule.id;

        let mut repo = MockRateLimitRepository::new();
        let return_rule = rule.clone();
        repo.expect_find_by_id()
            .returning(move |_| Ok(return_rule.clone()));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_token_bucket()
            .returning(|_, _, _| Ok(RateLimitDecision::allowed(99, 1700000060)));

        let uc = CheckRateLimitUseCase::new(Arc::new(repo), Arc::new(state_store));
        let result = uc.execute(&rule_id.to_string(), "user-123").await;

        assert!(result.is_ok());
        let decision = result.unwrap();
        assert!(decision.allowed);
        assert_eq!(decision.remaining, 99);
    }

    #[tokio::test]
    async fn test_check_rate_limit_denied() {
        let rule = make_rule();
        let rule_id = rule.id;

        let mut repo = MockRateLimitRepository::new();
        let return_rule = rule.clone();
        repo.expect_find_by_id()
            .returning(move |_| Ok(return_rule.clone()));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_token_bucket()
            .returning(|_, _, _| {
                Ok(RateLimitDecision::denied(
                    0,
                    1700000060,
                    "rate limit exceeded".to_string(),
                ))
            });

        let uc = CheckRateLimitUseCase::new(Arc::new(repo), Arc::new(state_store));
        let result = uc.execute(&rule_id.to_string(), "user-123").await;

        assert!(result.is_ok());
        let decision = result.unwrap();
        assert!(!decision.allowed);
        assert_eq!(decision.remaining, 0);
        assert_eq!(decision.reason, "rate limit exceeded");
    }

    #[tokio::test]
    async fn test_check_rate_limit_rule_not_found() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("not found")));

        let state_store = MockRateLimitStateStore::new();

        let uc = CheckRateLimitUseCase::new(Arc::new(repo), Arc::new(state_store));
        let result = uc
            .execute("550e8400-e29b-41d4-a716-446655440000", "user-123")
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CheckRateLimitError::RuleNotFound(_)));
    }

    #[tokio::test]
    async fn test_check_rate_limit_invalid_uuid() {
        let repo = MockRateLimitRepository::new();
        let state_store = MockRateLimitStateStore::new();

        let uc = CheckRateLimitUseCase::new(Arc::new(repo), Arc::new(state_store));
        let result = uc.execute("not-a-uuid", "user-123").await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CheckRateLimitError::InvalidRuleId(_)
        ));
    }

    #[tokio::test]
    async fn test_check_rate_limit_rule_disabled() {
        let mut rule = make_rule();
        rule.enabled = false;
        let rule_id = rule.id;

        let mut repo = MockRateLimitRepository::new();
        let return_rule = rule.clone();
        repo.expect_find_by_id()
            .returning(move |_| Ok(return_rule.clone()));

        let state_store = MockRateLimitStateStore::new();

        let uc = CheckRateLimitUseCase::new(Arc::new(repo), Arc::new(state_store));
        let result = uc.execute(&rule_id.to_string(), "user-123").await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CheckRateLimitError::RuleDisabled(_)
        ));
    }

    #[tokio::test]
    async fn test_check_rate_limit_fixed_window() {
        let mut rule = make_rule();
        rule.algorithm = Algorithm::FixedWindow;
        let rule_id = rule.id;

        let mut repo = MockRateLimitRepository::new();
        let return_rule = rule.clone();
        repo.expect_find_by_id()
            .returning(move |_| Ok(return_rule.clone()));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_fixed_window()
            .returning(|_, _, _| Ok(RateLimitDecision::allowed(50, 1700000060)));

        let uc = CheckRateLimitUseCase::new(Arc::new(repo), Arc::new(state_store));
        let result = uc.execute(&rule_id.to_string(), "user-123").await;

        assert!(result.is_ok());
        assert!(result.unwrap().allowed);
    }

    #[tokio::test]
    async fn test_check_rate_limit_sliding_window() {
        let mut rule = make_rule();
        rule.algorithm = Algorithm::SlidingWindow;
        let rule_id = rule.id;

        let mut repo = MockRateLimitRepository::new();
        let return_rule = rule.clone();
        repo.expect_find_by_id()
            .returning(move |_| Ok(return_rule.clone()));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_sliding_window()
            .returning(|_, _, _| Ok(RateLimitDecision::allowed(75, 1700000060)));

        let uc = CheckRateLimitUseCase::new(Arc::new(repo), Arc::new(state_store));
        let result = uc.execute(&rule_id.to_string(), "user-123").await;

        assert!(result.is_ok());
        assert!(result.unwrap().allowed);
    }
}
