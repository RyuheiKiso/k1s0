use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::Mutex;

use crate::client::RateLimitClient;
use crate::error::RateLimitError;
use crate::types::{RateLimitPolicy, RateLimitResult, RateLimitStatus};

/// テスト用インメモリレート制限クライアント。
/// ポリシーとカウンターをハッシュマップで管理する。
pub struct InMemoryRateLimitClient {
    inner: Mutex<InMemoryState>,
}

struct InMemoryState {
    counters: HashMap<String, u32>,
    policies: HashMap<String, RateLimitPolicy>,
}

impl InMemoryRateLimitClient {
    /// 新しい `InMemoryRateLimitClient` を生成する。
    /// デフォルトポリシー（limit=100, window_secs=3600, algorithm=token_bucket）が設定される。
    pub fn new() -> Self {
        let mut policies = HashMap::new();
        policies.insert(
            "default".to_string(),
            RateLimitPolicy {
                key: "default".to_string(),
                limit: 100,
                window_secs: 3600,
                algorithm: "token_bucket".to_string(),
            },
        );
        Self {
            inner: Mutex::new(InMemoryState {
                counters: HashMap::new(),
                policies,
            }),
        }
    }

    /// キーに対するポリシーを設定する。
    pub async fn set_policy(&self, key: impl Into<String>, policy: RateLimitPolicy) {
        let mut state = self.inner.lock().await;
        state.policies.insert(key.into(), policy);
    }

    /// キーの現在の使用済みカウントを返す。テスト用ヘルパー。
    pub async fn used_count(&self, key: &str) -> u32 {
        let state = self.inner.lock().await;
        state.counters.get(key).copied().unwrap_or(0)
    }
}

impl Default for InMemoryRateLimitClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RateLimitClient for InMemoryRateLimitClient {
    async fn check(&self, key: &str, cost: u32) -> Result<RateLimitStatus, RateLimitError> {
        let state = self.inner.lock().await;
        let policy = get_policy(&state, key);
        let used = state.counters.get(key).copied().unwrap_or(0);
        let reset_at = Utc::now() + chrono::Duration::seconds(policy.window_secs as i64);

        if used + cost > policy.limit {
            let retry_after = policy.window_secs;
            return Ok(RateLimitStatus {
                allowed: false,
                remaining: 0,
                reset_at,
                retry_after_secs: Some(retry_after),
            });
        }

        let remaining = policy.limit - used - cost;
        Ok(RateLimitStatus {
            allowed: true,
            remaining,
            reset_at,
            retry_after_secs: None,
        })
    }

    async fn consume(&self, key: &str, cost: u32) -> Result<RateLimitResult, RateLimitError> {
        let mut state = self.inner.lock().await;
        let policy = get_policy(&state, key);
        let used = state.counters.get(key).copied().unwrap_or(0);

        if used + cost > policy.limit {
            return Err(RateLimitError::LimitExceeded {
                retry_after_secs: policy.window_secs,
            });
        }

        let new_used = used + cost;
        state.counters.insert(key.to_string(), new_used);
        let remaining = policy.limit - new_used;
        let reset_at = Utc::now() + chrono::Duration::seconds(policy.window_secs as i64);

        Ok(RateLimitResult {
            remaining,
            reset_at,
        })
    }

    async fn get_limit(&self, key: &str) -> Result<RateLimitPolicy, RateLimitError> {
        let state = self.inner.lock().await;
        let policy = get_policy(&state, key);
        Ok(policy)
    }
}

fn get_policy(state: &InMemoryState, key: &str) -> RateLimitPolicy {
    if let Some(p) = state.policies.get(key) {
        return p.clone();
    }
    state
        .policies
        .get("default")
        .cloned()
        .unwrap_or_else(|| RateLimitPolicy {
            key: "default".to_string(),
            limit: 100,
            window_secs: 3600,
            algorithm: "token_bucket".to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_client_has_default_policy() {
        let client = InMemoryRateLimitClient::new();
        let policy = client.get_limit("any-key").await.unwrap();
        assert_eq!(policy.key, "default");
        assert_eq!(policy.limit, 100);
        assert_eq!(policy.window_secs, 3600);
        assert_eq!(policy.algorithm, "token_bucket");
    }

    #[tokio::test]
    async fn test_set_policy_and_get_limit() {
        let client = InMemoryRateLimitClient::new();
        client
            .set_policy(
                "tenant:TENANT-001",
                RateLimitPolicy {
                    key: "tenant:TENANT-001".to_string(),
                    limit: 50,
                    window_secs: 60,
                    algorithm: "sliding_window".to_string(),
                },
            )
            .await;

        let policy = client.get_limit("tenant:TENANT-001").await.unwrap();
        assert_eq!(policy.key, "tenant:TENANT-001");
        assert_eq!(policy.limit, 50);
        assert_eq!(policy.window_secs, 60);
    }

    #[tokio::test]
    async fn test_check_allowed_within_limit() {
        let client = InMemoryRateLimitClient::new();
        client
            .set_policy(
                "test-key",
                RateLimitPolicy {
                    key: "test-key".to_string(),
                    limit: 10,
                    window_secs: 60,
                    algorithm: "token_bucket".to_string(),
                },
            )
            .await;

        let status = client.check("test-key", 1).await.unwrap();
        assert!(status.allowed);
        assert_eq!(status.remaining, 9);
        assert!(status.retry_after_secs.is_none());
    }

    #[tokio::test]
    async fn test_check_denied_exceeds_limit() {
        let client = InMemoryRateLimitClient::new();
        client
            .set_policy(
                "test-key",
                RateLimitPolicy {
                    key: "test-key".to_string(),
                    limit: 5,
                    window_secs: 60,
                    algorithm: "token_bucket".to_string(),
                },
            )
            .await;

        let status = client.check("test-key", 10).await.unwrap();
        assert!(!status.allowed);
        assert_eq!(status.remaining, 0);
        assert_eq!(status.retry_after_secs, Some(60));
    }

    #[tokio::test]
    async fn test_consume_updates_counter() {
        let client = InMemoryRateLimitClient::new();
        client
            .set_policy(
                "test-key",
                RateLimitPolicy {
                    key: "test-key".to_string(),
                    limit: 100,
                    window_secs: 60,
                    algorithm: "token_bucket".to_string(),
                },
            )
            .await;

        let result = client.consume("test-key", 5).await.unwrap();
        assert_eq!(result.remaining, 95);

        let used = client.used_count("test-key").await;
        assert_eq!(used, 5);
    }

    #[tokio::test]
    async fn test_consume_returns_error_when_limit_exceeded() {
        let client = InMemoryRateLimitClient::new();
        client
            .set_policy(
                "test-key",
                RateLimitPolicy {
                    key: "test-key".to_string(),
                    limit: 3,
                    window_secs: 60,
                    algorithm: "token_bucket".to_string(),
                },
            )
            .await;

        let result = client.consume("test-key", 10).await;
        assert!(matches!(
            result,
            Err(RateLimitError::LimitExceeded { retry_after_secs: 60 })
        ));
    }

    #[tokio::test]
    async fn test_used_count_starts_at_zero() {
        let client = InMemoryRateLimitClient::new();
        let count = client.used_count("unknown-key").await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_used_count_after_multiple_consumes() {
        let client = InMemoryRateLimitClient::new();
        client
            .set_policy(
                "test-key",
                RateLimitPolicy {
                    key: "test-key".to_string(),
                    limit: 100,
                    window_secs: 60,
                    algorithm: "token_bucket".to_string(),
                },
            )
            .await;

        client.consume("test-key", 3).await.unwrap();
        client.consume("test-key", 7).await.unwrap();

        let count = client.used_count("test-key").await;
        assert_eq!(count, 10);
    }

    #[tokio::test]
    async fn test_check_uses_default_policy_for_unknown_key() {
        let client = InMemoryRateLimitClient::new();
        // デフォルトポリシーは limit=100
        let status = client.check("unknown-key", 1).await.unwrap();
        assert!(status.allowed);
        assert_eq!(status.remaining, 99);
    }

    #[tokio::test]
    async fn test_check_before_execute_pattern() {
        let client = InMemoryRateLimitClient::new();
        client
            .set_policy(
                "api-key",
                RateLimitPolicy {
                    key: "api-key".to_string(),
                    limit: 10,
                    window_secs: 60,
                    algorithm: "token_bucket".to_string(),
                },
            )
            .await;

        // check before execute パターン
        let status = client.check("api-key", 1).await.unwrap();
        assert!(status.allowed);

        // check は counter を更新しない
        let used_after_check = client.used_count("api-key").await;
        assert_eq!(used_after_check, 0);

        // consume で消費
        client.consume("api-key", 1).await.unwrap();
        let used_after_consume = client.used_count("api-key").await;
        assert_eq!(used_after_consume, 1);
    }
}
