use k1s0_ratelimit_client::{
    InMemoryRateLimitClient, RateLimitClient, RateLimitError, RateLimitPolicy,
};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

async fn client_with_policy(key: &str, limit: u32, window_secs: u64) -> InMemoryRateLimitClient {
    let client = InMemoryRateLimitClient::new();
    client
        .set_policy(
            key,
            RateLimitPolicy {
                key: key.to_string(),
                limit,
                window_secs,
                algorithm: "token_bucket".to_string(),
            },
        )
        .await;
    client
}

// ===========================================================================
// check — within limit
// ===========================================================================

#[tokio::test]
async fn check_within_limit_returns_allowed() {
    let client = client_with_policy("k", 10, 60).await;
    let status = client.check("k", 1).await.unwrap();
    assert!(status.allowed);
    assert_eq!(status.remaining, 9);
    assert!(status.retry_after_secs.is_none());
}

#[tokio::test]
async fn check_exact_limit_is_allowed() {
    let client = client_with_policy("k", 5, 60).await;
    let status = client.check("k", 5).await.unwrap();
    assert!(status.allowed);
    assert_eq!(status.remaining, 0);
}

#[tokio::test]
async fn check_does_not_consume_quota() {
    let client = client_with_policy("k", 10, 60).await;
    client.check("k", 3).await.unwrap();
    client.check("k", 3).await.unwrap();
    assert_eq!(client.used_count("k").await, 0);
}

// ===========================================================================
// check — exceed limit
// ===========================================================================

#[tokio::test]
async fn check_exceeding_limit_returns_denied() {
    let client = client_with_policy("k", 5, 120).await;
    let status = client.check("k", 6).await.unwrap();
    assert!(!status.allowed);
    assert_eq!(status.remaining, 0);
    assert_eq!(status.retry_after_secs, Some(120));
}

#[tokio::test]
async fn check_after_partial_consume_detects_overflow() {
    let client = client_with_policy("k", 10, 60).await;
    client.consume("k", 8).await.unwrap();
    let status = client.check("k", 5).await.unwrap();
    assert!(!status.allowed);
}

// ===========================================================================
// consume — basic
// ===========================================================================

#[tokio::test]
async fn consume_decrements_remaining() {
    let client = client_with_policy("k", 100, 60).await;
    let r = client.consume("k", 10).await.unwrap();
    assert_eq!(r.remaining, 90);
    assert_eq!(client.used_count("k").await, 10);
}

#[tokio::test]
async fn consume_multiple_accumulates() {
    let client = client_with_policy("k", 100, 60).await;
    client.consume("k", 20).await.unwrap();
    client.consume("k", 30).await.unwrap();
    let r = client.consume("k", 10).await.unwrap();
    assert_eq!(r.remaining, 40);
    assert_eq!(client.used_count("k").await, 60);
}

#[tokio::test]
async fn consume_exact_limit_succeeds() {
    let client = client_with_policy("k", 5, 60).await;
    let r = client.consume("k", 5).await.unwrap();
    assert_eq!(r.remaining, 0);
}

// ===========================================================================
// consume — exceed limit (error)
// ===========================================================================

#[tokio::test]
async fn consume_exceeding_limit_returns_error() {
    let client = client_with_policy("k", 3, 60).await;
    let err = client.consume("k", 4).await.unwrap_err();
    assert!(matches!(
        err,
        RateLimitError::LimitExceeded {
            retry_after_secs: 60
        }
    ));
}

#[tokio::test]
async fn consume_after_exhaustion_returns_error() {
    let client = client_with_policy("k", 5, 60).await;
    client.consume("k", 5).await.unwrap();
    let err = client.consume("k", 1).await.unwrap_err();
    assert!(matches!(err, RateLimitError::LimitExceeded { .. }));
}

// ===========================================================================
// policy CRUD
// ===========================================================================

#[tokio::test]
async fn default_policy_is_applied_to_unknown_keys() {
    let client = InMemoryRateLimitClient::new();
    let policy = client.get_limit("unknown").await.unwrap();
    assert_eq!(policy.key, "default");
    assert_eq!(policy.limit, 100);
    assert_eq!(policy.window_secs, 3600);
    assert_eq!(policy.algorithm, "token_bucket");
}

#[tokio::test]
async fn set_policy_overrides_default() {
    let client = client_with_policy("api", 50, 120).await;
    let policy = client.get_limit("api").await.unwrap();
    assert_eq!(policy.key, "api");
    assert_eq!(policy.limit, 50);
    assert_eq!(policy.window_secs, 120);
}

#[tokio::test]
async fn overwrite_existing_policy() {
    let client = client_with_policy("api", 50, 120).await;
    client
        .set_policy(
            "api",
            RateLimitPolicy {
                key: "api".to_string(),
                limit: 200,
                window_secs: 300,
                algorithm: "sliding_window".to_string(),
            },
        )
        .await;
    let policy = client.get_limit("api").await.unwrap();
    assert_eq!(policy.limit, 200);
    assert_eq!(policy.window_secs, 300);
    assert_eq!(policy.algorithm, "sliding_window");
}

// ===========================================================================
// sliding window simulation (multiple keys)
// ===========================================================================

#[tokio::test]
async fn independent_keys_have_independent_counters() {
    let client = InMemoryRateLimitClient::new();
    client
        .set_policy(
            "a",
            RateLimitPolicy {
                key: "a".to_string(),
                limit: 10,
                window_secs: 60,
                algorithm: "token_bucket".to_string(),
            },
        )
        .await;
    client
        .set_policy(
            "b",
            RateLimitPolicy {
                key: "b".to_string(),
                limit: 10,
                window_secs: 60,
                algorithm: "token_bucket".to_string(),
            },
        )
        .await;

    client.consume("a", 7).await.unwrap();
    client.consume("b", 3).await.unwrap();

    assert_eq!(client.used_count("a").await, 7);
    assert_eq!(client.used_count("b").await, 3);

    let sa = client.check("a", 1).await.unwrap();
    assert_eq!(sa.remaining, 2);
    let sb = client.check("b", 1).await.unwrap();
    assert_eq!(sb.remaining, 6);
}

// ===========================================================================
// check-before-execute pattern
// ===========================================================================

#[tokio::test]
async fn check_then_consume_pattern() {
    let client = client_with_policy("api", 10, 60).await;

    let status = client.check("api", 1).await.unwrap();
    assert!(status.allowed);
    assert_eq!(client.used_count("api").await, 0);

    client.consume("api", 1).await.unwrap();
    assert_eq!(client.used_count("api").await, 1);
}

// ===========================================================================
// reset_at is in the future
// ===========================================================================

#[tokio::test]
async fn reset_at_is_in_the_future() {
    let client = client_with_policy("k", 10, 60).await;
    let status = client.check("k", 1).await.unwrap();
    assert!(status.reset_at > chrono::Utc::now());
}

// ===========================================================================
// error variant coverage
// ===========================================================================

// LimitExceededエラーの表示メッセージに待機秒数が含まれることを確認する。
#[test]
fn error_display_limit_exceeded() {
    let e = RateLimitError::LimitExceeded {
        retry_after_secs: 42,
    };
    let msg = format!("{e}");
    assert!(msg.contains("42"));
}

// KeyNotFoundエラーの表示メッセージにキー名が含まれることを確認する。
#[test]
fn error_display_key_not_found() {
    let e = RateLimitError::KeyNotFound {
        key: "missing".to_string(),
    };
    let msg = format!("{e}");
    assert!(msg.contains("missing"));
}

// ServerErrorの表示メッセージにエラー内容が含まれることを確認する。
#[test]
fn error_display_server_error() {
    let e = RateLimitError::ServerError("boom".to_string());
    let msg = format!("{e}");
    assert!(msg.contains("boom"));
}

// Timeoutエラーの表示メッセージが空でないことを確認する。
#[test]
fn error_display_timeout() {
    let e = RateLimitError::Timeout;
    let msg = format!("{e}");
    assert!(!msg.is_empty());
}

// ===========================================================================
// used_count helper
// ===========================================================================

#[tokio::test]
async fn used_count_starts_at_zero() {
    let client = InMemoryRateLimitClient::new();
    assert_eq!(client.used_count("anything").await, 0);
}

// ===========================================================================
// Default trait
// ===========================================================================

#[tokio::test]
async fn default_trait_creates_valid_client() {
    let client = InMemoryRateLimitClient::default();
    let policy = client.get_limit("any").await.unwrap();
    assert_eq!(policy.limit, 100);
}
