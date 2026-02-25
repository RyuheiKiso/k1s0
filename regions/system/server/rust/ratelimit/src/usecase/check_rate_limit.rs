use std::sync::Arc;

use crate::domain::entity::{Algorithm, RateLimitDecision};
use crate::domain::repository::{RateLimitRepository, RateLimitStateStore};

/// CheckRateLimitError はレートリミットチェックに関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum CheckRateLimitError {
    #[error("rule not found: {0}")]
    RuleNotFound(String),

    #[error("rule disabled: {0}")]
    RuleDisabled(String),

    #[error("validation error: {0}")]
    ValidationError(String),

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
        scope: &str,
        identifier: &str,
        window_secs: i64,
    ) -> Result<RateLimitDecision, CheckRateLimitError> {
        if scope.is_empty() {
            return Err(CheckRateLimitError::ValidationError("scope is required".to_string()));
        }
        if identifier.is_empty() {
            return Err(CheckRateLimitError::ValidationError("identifier is required".to_string()));
        }

        // scopeでルールを検索し、最初のenabledなルールを使用
        let rules = self
            .rule_repo
            .find_by_scope(scope)
            .await
            .map_err(|e| CheckRateLimitError::Internal(e.to_string()))?;

        let (limit, effective_window) = rules
            .iter()
            .find(|r| r.enabled)
            .map(|r| (r.limit, r.window_seconds))
            .unwrap_or((100, if window_secs > 0 { window_secs } else { 60 }));

        // Redis キー: ratelimit:{scope}:{identifier}
        let redis_key = format!("ratelimit:{}:{}", scope, identifier);

        // マッチするルールがある場合はそのアルゴリズムを使用、なければトークンバケット
        let algorithm = rules
            .iter()
            .find(|r| r.enabled)
            .map(|r| r.algorithm.clone())
            .unwrap_or(Algorithm::TokenBucket);

        let decision = match algorithm {
            Algorithm::TokenBucket => {
                self.state_store
                    .check_token_bucket(&redis_key, limit, effective_window)
                    .await
            }
            Algorithm::FixedWindow => {
                self.state_store
                    .check_fixed_window(&redis_key, limit, effective_window)
                    .await
            }
            Algorithm::SlidingWindow => {
                self.state_store
                    .check_sliding_window(&redis_key, limit, effective_window)
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
    use crate::domain::entity::{Algorithm, RateLimitDecision, RateLimitRule};
    use crate::domain::repository::rate_limit_repository::{
        MockRateLimitRepository, MockRateLimitStateStore,
    };

    fn make_rule() -> RateLimitRule {
        RateLimitRule::new(
            "service".to_string(),
            "global".to_string(),
            100,
            60,
            Algorithm::TokenBucket,
        )
    }

    #[tokio::test]
    async fn test_check_rate_limit_allowed() {
        let rule = make_rule();

        let mut repo = MockRateLimitRepository::new();
        let return_rule = rule.clone();
        repo.expect_find_by_scope()
            .returning(move |_| Ok(vec![return_rule.clone()]));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_token_bucket()
            .returning(|_, _, _| Ok(RateLimitDecision::allowed(99, 1700000060)));

        let uc = CheckRateLimitUseCase::new(Arc::new(repo), Arc::new(state_store));
        let result = uc.execute("service", "user-123", 60).await;

        assert!(result.is_ok());
        let decision = result.unwrap();
        assert!(decision.allowed);
        assert_eq!(decision.remaining, 99);
    }

    #[tokio::test]
    async fn test_check_rate_limit_denied() {
        let rule = make_rule();

        let mut repo = MockRateLimitRepository::new();
        let return_rule = rule.clone();
        repo.expect_find_by_scope()
            .returning(move |_| Ok(vec![return_rule.clone()]));

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
        let result = uc.execute("service", "user-123", 60).await;

        assert!(result.is_ok());
        let decision = result.unwrap();
        assert!(!decision.allowed);
        assert_eq!(decision.remaining, 0);
        assert_eq!(decision.reason, "rate limit exceeded");
    }

    #[tokio::test]
    async fn test_check_rate_limit_no_rule_uses_default() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_scope()
            .returning(|_| Ok(vec![]));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_token_bucket()
            .returning(|_, _, _| Ok(RateLimitDecision::allowed(99, 1700000060)));

        let uc = CheckRateLimitUseCase::new(Arc::new(repo), Arc::new(state_store));
        let result = uc.execute("user", "user-123", 60).await;

        assert!(result.is_ok());
        let decision = result.unwrap();
        assert!(decision.allowed);
    }

    #[tokio::test]
    async fn test_check_rate_limit_empty_scope_error() {
        let repo = MockRateLimitRepository::new();
        let state_store = MockRateLimitStateStore::new();

        let uc = CheckRateLimitUseCase::new(Arc::new(repo), Arc::new(state_store));
        let result = uc.execute("", "user-123", 60).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CheckRateLimitError::ValidationError(_)));
    }

    #[tokio::test]
    async fn test_check_rate_limit_fixed_window() {
        let mut rule = make_rule();
        rule.algorithm = Algorithm::FixedWindow;

        let mut repo = MockRateLimitRepository::new();
        let return_rule = rule.clone();
        repo.expect_find_by_scope()
            .returning(move |_| Ok(vec![return_rule.clone()]));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_fixed_window()
            .returning(|_, _, _| Ok(RateLimitDecision::allowed(50, 1700000060)));

        let uc = CheckRateLimitUseCase::new(Arc::new(repo), Arc::new(state_store));
        let result = uc.execute("service", "user-123", 60).await;

        assert!(result.is_ok());
        assert!(result.unwrap().allowed);
    }

    #[tokio::test]
    async fn test_check_rate_limit_sliding_window() {
        let mut rule = make_rule();
        rule.algorithm = Algorithm::SlidingWindow;

        let mut repo = MockRateLimitRepository::new();
        let return_rule = rule.clone();
        repo.expect_find_by_scope()
            .returning(move |_| Ok(vec![return_rule.clone()]));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_sliding_window()
            .returning(|_, _, _| Ok(RateLimitDecision::allowed(75, 1700000060)));

        let uc = CheckRateLimitUseCase::new(Arc::new(repo), Arc::new(state_store));
        let result = uc.execute("service", "user-123", 60).await;

        assert!(result.is_ok());
        assert!(result.unwrap().allowed);
    }
}
