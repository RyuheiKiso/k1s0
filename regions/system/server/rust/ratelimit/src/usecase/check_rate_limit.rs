use std::sync::Arc;

use crate::domain::entity::{Algorithm, RateLimitDecision};
use crate::domain::repository::{RateLimitRepository, RateLimitStateStore};
use crate::domain::service::RateLimitDomainService;

/// CheckRateLimitError はレートリミットチェックに関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum CheckRateLimitError {
    #[error("rule not found: {0}")]
    #[allow(dead_code)]
    RuleNotFound(String),

    #[error("rule disabled: {0}")]
    #[allow(dead_code)]
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
    fail_open: bool,
    default_limit: u32,
    default_window_seconds: u32,
}

impl CheckRateLimitUseCase {
    #[allow(dead_code)]
    pub fn new(
        rule_repo: Arc<dyn RateLimitRepository>,
        state_store: Arc<dyn RateLimitStateStore>,
    ) -> Self {
        Self {
            rule_repo,
            state_store,
            fail_open: true,
            default_limit: 100,
            default_window_seconds: 60,
        }
    }

    pub fn with_fallback_policy(
        rule_repo: Arc<dyn RateLimitRepository>,
        state_store: Arc<dyn RateLimitStateStore>,
        fail_open: bool,
        default_limit: u32,
        default_window_seconds: u32,
    ) -> Self {
        Self {
            rule_repo,
            state_store,
            fail_open,
            default_limit: default_limit.max(1),
            default_window_seconds: default_window_seconds.max(1),
        }
    }

    pub async fn execute(
        &self,
        scope: &str,
        identifier: &str,
        window_secs: i64,
    ) -> Result<RateLimitDecision, CheckRateLimitError> {
        RateLimitDomainService::validate_scope(scope)
            .map_err(CheckRateLimitError::ValidationError)?;
        RateLimitDomainService::validate_identifier(identifier)
            .map_err(CheckRateLimitError::ValidationError)?;

        // scope で候補ルールを検索し、identifier 完全一致 -> "*" の順でマッチさせる
        let rules = self
            .rule_repo
            .find_by_scope(scope)
            .await
            .map_err(|e| CheckRateLimitError::Internal(e.to_string()))?;

        let matched_rule = rules
            .iter()
            .filter(|r| r.enabled)
            .find(|r| r.identifier_pattern == identifier)
            .or_else(|| {
                rules
                    .iter()
                    .filter(|r| r.enabled)
                    .find(|r| r.identifier_pattern == "*")
            })
            .or_else(|| {
                rules
                    .iter()
                    .filter(|r| r.enabled)
                    .find(|r| identifier_matches(&r.identifier_pattern, identifier))
            });
        let (limit, effective_window) = RateLimitDomainService::effective_limit_and_window(
            matched_rule,
            self.default_limit,
            self.default_window_seconds,
            window_secs,
        );

        // Redis キー: ratelimit:{scope}:{identifier}
        let redis_key = format!("ratelimit:{}:{}", scope, identifier);

        // マッチするルールがある場合はそのアルゴリズムを使用、なければトークンバケット
        let algorithm = RateLimitDomainService::resolve_algorithm(matched_rule);

        let backend_decision = match algorithm {
            Algorithm::TokenBucket => {
                self.state_store
                    .check_token_bucket(&redis_key, i64::from(limit), i64::from(effective_window))
                    .await
            }
            Algorithm::FixedWindow => {
                self.state_store
                    .check_fixed_window(&redis_key, i64::from(limit), i64::from(effective_window))
                    .await
            }
            Algorithm::SlidingWindow => {
                self.state_store
                    .check_sliding_window(&redis_key, i64::from(limit), i64::from(effective_window))
                    .await
            }
            Algorithm::LeakyBucket => {
                self.state_store
                    .check_leaky_bucket(&redis_key, i64::from(limit), i64::from(effective_window))
                    .await
            }
        };

        // バックエンド（Redis等）のエラー時は fail_open/fail_closed ポリシーに従う
        let mut decision = match backend_decision {
            Ok(decision) => decision,
            Err(e) => {
                if self.fail_open {
                    // セキュリティ上の注意: fail-open はバックエンド障害時にリクエストを許可する。
                    // 攻撃者がバックエンドを意図的にダウンさせた場合、レートリミットが無効化される可能性がある。
                    // 運用チームはこの警告を監視し、頻発する場合は fail_open=false への切替を検討すること。
                    tracing::warn!(
                        scope = scope,
                        identifier = identifier,
                        error = %e,
                        "レートリミットバックエンドエラーにより fail-open でリクエストを許可しました。レートリミットが一時的に無効化されています"
                    );
                    RateLimitDomainService::fail_open_decision(
                        scope,
                        identifier,
                        limit,
                        effective_window,
                        matched_rule.map(|r| r.id.to_string()),
                    )
                } else {
                    return Err(CheckRateLimitError::Internal(e.to_string()));
                }
            }
        };

        decision.scope = scope.to_string();
        decision.identifier = identifier.to_string();
        decision.used = (decision.limit - decision.remaining).max(0);
        decision.rule_id = matched_rule.map(|r| r.id.to_string()).unwrap_or_default();

        Ok(decision)
    }
}

fn identifier_matches(pattern: &str, identifier: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return identifier.starts_with(prefix);
    }
    pattern == identifier
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::{Algorithm, RateLimitDecision, RateLimitRule};
    use crate::domain::repository::rate_limit_repository::{
        MockRateLimitRepository, MockRateLimitStateStore,
    };
    use chrono::TimeZone;

    fn ts(seconds: i64) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc.timestamp_opt(seconds, 0).single().unwrap()
    }

    fn make_rule() -> RateLimitRule {
        RateLimitRule::new(
            "service".to_string(),
            "*".to_string(),
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
            .returning(|_, _, _| Ok(RateLimitDecision::allowed(100, 99, ts(1700000060))));

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
                    100,
                    0,
                    ts(1700000060),
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
        repo.expect_find_by_scope().returning(|_| Ok(vec![]));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_token_bucket()
            .returning(|_, _, _| Ok(RateLimitDecision::allowed(100, 99, ts(1700000060))));

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
        assert!(matches!(
            result.unwrap_err(),
            CheckRateLimitError::ValidationError(_)
        ));
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
            .returning(|_, _, _| Ok(RateLimitDecision::allowed(100, 50, ts(1700000060))));

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
            .returning(|_, _, _| Ok(RateLimitDecision::allowed(100, 75, ts(1700000060))));

        let uc = CheckRateLimitUseCase::new(Arc::new(repo), Arc::new(state_store));
        let result = uc.execute("service", "user-123", 60).await;

        assert!(result.is_ok());
        assert!(result.unwrap().allowed);
    }

    #[tokio::test]
    async fn test_check_rate_limit_leaky_bucket() {
        let mut rule = make_rule();
        rule.algorithm = Algorithm::LeakyBucket;

        let mut repo = MockRateLimitRepository::new();
        let return_rule = rule.clone();
        repo.expect_find_by_scope()
            .returning(move |_| Ok(vec![return_rule.clone()]));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_leaky_bucket()
            .returning(|_, _, _| Ok(RateLimitDecision::allowed(100, 80, ts(1700000060))));

        let uc = CheckRateLimitUseCase::new(Arc::new(repo), Arc::new(state_store));
        let result = uc.execute("service", "user-123", 60).await;

        assert!(result.is_ok());
        assert!(result.unwrap().allowed);
    }

    #[tokio::test]
    async fn test_check_rate_limit_fail_open_on_backend_error() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_scope().returning(|_| Ok(vec![]));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_token_bucket()
            .returning(|_, _, _| Err(anyhow::anyhow!("redis unavailable")));

        let uc = CheckRateLimitUseCase::with_fallback_policy(
            Arc::new(repo),
            Arc::new(state_store),
            true,
            100,
            60,
        );
        let result = uc.execute("service", "user-123", 60).await;
        assert!(result.is_ok());
        assert!(result.unwrap().allowed);
    }

    #[tokio::test]
    async fn test_check_rate_limit_fail_closed_on_backend_error() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_by_scope().returning(|_| Ok(vec![]));

        let mut state_store = MockRateLimitStateStore::new();
        state_store
            .expect_check_token_bucket()
            .returning(|_, _, _| Err(anyhow::anyhow!("redis unavailable")));

        let uc = CheckRateLimitUseCase::with_fallback_policy(
            Arc::new(repo),
            Arc::new(state_store),
            false,
            100,
            60,
        );
        let result = uc.execute("service", "user-123", 60).await;
        assert!(matches!(result, Err(CheckRateLimitError::Internal(_))));
    }
}
