use std::time::Duration;

use k1s0_quota_client::{
    CachedQuotaClient, InMemoryQuotaClient, QuotaClient, QuotaClientConfig, QuotaClientError,
    QuotaPeriod, QuotaPolicy, QuotaStatus, QuotaUsage,
};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

// QuotaClientConfigのデフォルト値が正しく設定されることを確認する。
#[test]
fn config_new_defaults() {
    let cfg = QuotaClientConfig::new("http://quota-server:8080");
    assert_eq!(cfg.server_url, "http://quota-server:8080");
    assert_eq!(cfg.timeout, Duration::from_secs(5));
    assert_eq!(cfg.policy_cache_ttl, Duration::from_secs(60));
}

// with_timeoutでタイムアウトが正しく設定されることを確認する。
#[test]
fn config_with_timeout() {
    let cfg = QuotaClientConfig::new("http://localhost:9000").with_timeout(Duration::from_secs(30));
    assert_eq!(cfg.timeout, Duration::from_secs(30));
}

// with_policy_cache_ttlでポリシーキャッシュTTLが正しく設定されることを確認する。
#[test]
fn config_with_policy_cache_ttl() {
    let cfg = QuotaClientConfig::new("http://localhost:9000")
        .with_policy_cache_ttl(Duration::from_secs(300));
    assert_eq!(cfg.policy_cache_ttl, Duration::from_secs(300));
}

// ビルダーメソッドのチェーンが正しく機能することを確認する。
#[test]
fn config_builder_chaining() {
    let cfg = QuotaClientConfig::new("http://localhost:9000")
        .with_timeout(Duration::from_secs(10))
        .with_policy_cache_ttl(Duration::from_secs(120));
    assert_eq!(cfg.timeout, Duration::from_secs(10));
    assert_eq!(cfg.policy_cache_ttl, Duration::from_secs(120));
}

// ---------------------------------------------------------------------------
// Model types
// ---------------------------------------------------------------------------

// QuotaPeriodの各バリアントが正しくデバッグ表示されることを確認する。
#[test]
fn quota_period_variants() {
    let h = QuotaPeriod::Hourly;
    let d = QuotaPeriod::Daily;
    let m = QuotaPeriod::Monthly;
    let c = QuotaPeriod::Custom(7200);

    assert!(format!("{:?}", h).contains("Hourly"));
    assert!(format!("{:?}", d).contains("Daily"));
    assert!(format!("{:?}", m).contains("Monthly"));
    assert!(format!("{:?}", c).contains("7200"));
}

// QuotaPeriodがJSONでシリアライズ・デシリアライズできることを確認する。
#[test]
fn quota_period_serde_roundtrip() {
    let periods = vec![
        QuotaPeriod::Hourly,
        QuotaPeriod::Daily,
        QuotaPeriod::Monthly,
        QuotaPeriod::Custom(3600),
    ];
    for p in &periods {
        let json = serde_json::to_string(p).unwrap();
        let deserialized: QuotaPeriod = serde_json::from_str(&json).unwrap();
        assert_eq!(*p, deserialized);
    }
}

// QuotaPolicyがJSONでシリアライズ・デシリアライズできることを確認する。
#[test]
fn quota_policy_serde_roundtrip() {
    let policy = QuotaPolicy {
        quota_id: "api-calls".to_string(),
        limit: 5000,
        period: QuotaPeriod::Daily,
        reset_strategy: "rolling".to_string(),
    };
    let json = serde_json::to_string(&policy).unwrap();
    let deserialized: QuotaPolicy = serde_json::from_str(&json).unwrap();
    assert_eq!(policy, deserialized);
}

// ---------------------------------------------------------------------------
// Error variants
// ---------------------------------------------------------------------------

// 各エラーバリアントの表示メッセージが適切な情報を含むことを確認する。
#[test]
fn error_display_messages() {
    let err = QuotaClientError::ConnectionError("timeout".to_string());
    assert!(err.to_string().contains("timeout"));

    let err = QuotaClientError::QuotaExceeded {
        quota_id: "q1".to_string(),
        remaining: 0,
    };
    assert!(err.to_string().contains("q1"));
    assert!(err.to_string().contains("0"));

    let err = QuotaClientError::NotFound("missing".to_string());
    assert!(err.to_string().contains("missing"));

    let err = QuotaClientError::InvalidResponse("bad json".to_string());
    assert!(err.to_string().contains("bad json"));

    let err = QuotaClientError::Internal("crash".to_string());
    assert!(err.to_string().contains("crash"));
}

// ---------------------------------------------------------------------------
// InMemoryQuotaClient — basic operations
// ---------------------------------------------------------------------------

#[tokio::test]
async fn inmemory_check_new_quota_is_allowed() {
    let client = InMemoryQuotaClient::new();
    let status: QuotaStatus = client.check("new-quota", 1).await.unwrap();
    assert!(status.allowed);
    assert_eq!(status.limit, 1000); // default limit
    assert_eq!(status.remaining, 1000);
}

#[tokio::test]
async fn inmemory_check_within_limit_is_allowed() {
    let client = InMemoryQuotaClient::new();
    let _: QuotaUsage = client.increment("q", 500).await.unwrap();

    let status: QuotaStatus = client.check("q", 400).await.unwrap();
    assert!(status.allowed);
    assert_eq!(status.remaining, 500);
}

#[tokio::test]
async fn inmemory_check_exceeding_limit_is_denied() {
    let client = InMemoryQuotaClient::new();
    let _: QuotaUsage = client.increment("q", 900).await.unwrap();

    let status: QuotaStatus = client.check("q", 200).await.unwrap();
    assert!(!status.allowed);
    assert_eq!(status.remaining, 100);
}

#[tokio::test]
async fn inmemory_check_exact_remaining_is_allowed() {
    let client = InMemoryQuotaClient::new();
    let _: QuotaUsage = client.increment("q", 500).await.unwrap();

    let status: QuotaStatus = client.check("q", 500).await.unwrap();
    assert!(status.allowed);
}

#[tokio::test]
async fn inmemory_check_zero_amount_is_allowed() {
    let client = InMemoryQuotaClient::new();
    let _: QuotaUsage = client.increment("q", 999).await.unwrap();

    let status: QuotaStatus = client.check("q", 0).await.unwrap();
    assert!(status.allowed);
}

// ---------------------------------------------------------------------------
// InMemoryQuotaClient — increment
// ---------------------------------------------------------------------------

#[tokio::test]
async fn inmemory_increment_returns_updated_usage() {
    let client = InMemoryQuotaClient::new();

    let usage: QuotaUsage = client.increment("q", 100).await.unwrap();
    assert_eq!(usage.quota_id, "q");
    assert_eq!(usage.used, 100);
    assert_eq!(usage.limit, 1000);
}

#[tokio::test]
async fn inmemory_increment_accumulates() {
    let client = InMemoryQuotaClient::new();

    let _: QuotaUsage = client.increment("q", 200).await.unwrap();
    let _: QuotaUsage = client.increment("q", 300).await.unwrap();
    let usage: QuotaUsage = client.increment("q", 100).await.unwrap();
    assert_eq!(usage.used, 600);
}

#[tokio::test]
async fn inmemory_increment_beyond_limit_saturates() {
    let client = InMemoryQuotaClient::new();
    let usage: QuotaUsage = client.increment("q", u64::MAX).await.unwrap();
    // saturating_add should not overflow
    assert!(usage.used > 0);
}

#[tokio::test]
async fn inmemory_increment_zero_amount() {
    let client = InMemoryQuotaClient::new();
    let usage: QuotaUsage = client.increment("q", 0).await.unwrap();
    assert_eq!(usage.used, 0);
}

// ---------------------------------------------------------------------------
// InMemoryQuotaClient — get_usage
// ---------------------------------------------------------------------------

#[tokio::test]
async fn inmemory_get_usage_new_quota_starts_at_zero() {
    let client = InMemoryQuotaClient::new();
    let usage: QuotaUsage = client.get_usage("fresh").await.unwrap();
    assert_eq!(usage.quota_id, "fresh");
    assert_eq!(usage.used, 0);
    assert_eq!(usage.limit, 1000);
    assert_eq!(usage.period, QuotaPeriod::Daily);
}

#[tokio::test]
async fn inmemory_get_usage_reflects_increments() {
    let client = InMemoryQuotaClient::new();
    let _: QuotaUsage = client.increment("q", 42).await.unwrap();

    let usage: QuotaUsage = client.get_usage("q").await.unwrap();
    assert_eq!(usage.used, 42);
}

// ---------------------------------------------------------------------------
// InMemoryQuotaClient — get_policy
// ---------------------------------------------------------------------------

#[tokio::test]
async fn inmemory_get_policy_returns_default_for_unknown() {
    let client = InMemoryQuotaClient::new();
    let policy: QuotaPolicy = client.get_policy("unknown").await.unwrap();
    assert_eq!(policy.quota_id, "unknown");
    assert_eq!(policy.limit, 1000);
    assert_eq!(policy.period, QuotaPeriod::Daily);
    assert_eq!(policy.reset_strategy, "fixed");
}

#[tokio::test]
async fn inmemory_get_policy_returns_custom_policy() {
    let client = InMemoryQuotaClient::new();
    client.set_policy(
        "premium",
        QuotaPolicy {
            quota_id: "premium".to_string(),
            limit: 50000,
            period: QuotaPeriod::Monthly,
            reset_strategy: "rolling".to_string(),
        },
    );

    let policy: QuotaPolicy = client.get_policy("premium").await.unwrap();
    assert_eq!(policy.limit, 50000);
    assert_eq!(policy.period, QuotaPeriod::Monthly);
    assert_eq!(policy.reset_strategy, "rolling");
}

// ---------------------------------------------------------------------------
// InMemoryQuotaClient — custom policy affects quota behavior
// ---------------------------------------------------------------------------

#[tokio::test]
async fn inmemory_custom_policy_affects_new_quota_limit() {
    let client = InMemoryQuotaClient::new();
    client.set_policy(
        "small",
        QuotaPolicy {
            quota_id: "small".to_string(),
            limit: 10,
            period: QuotaPeriod::Hourly,
            reset_strategy: "fixed".to_string(),
        },
    );

    let usage: QuotaUsage = client.get_usage("small").await.unwrap();
    assert_eq!(usage.limit, 10);
    assert_eq!(usage.period, QuotaPeriod::Hourly);

    let status: QuotaStatus = client.check("small", 11).await.unwrap();
    assert!(!status.allowed);
}

// ---------------------------------------------------------------------------
// InMemoryQuotaClient — multiple independent quotas
// ---------------------------------------------------------------------------

#[tokio::test]
async fn inmemory_quotas_are_independent() {
    let client = InMemoryQuotaClient::new();
    let _: QuotaUsage = client.increment("alpha", 100).await.unwrap();
    let _: QuotaUsage = client.increment("beta", 200).await.unwrap();

    let alpha_usage: QuotaUsage = client.get_usage("alpha").await.unwrap();
    let beta_usage: QuotaUsage = client.get_usage("beta").await.unwrap();
    assert_eq!(alpha_usage.used, 100);
    assert_eq!(beta_usage.used, 200);
}

// ---------------------------------------------------------------------------
// InMemoryQuotaClient — Default trait
// ---------------------------------------------------------------------------

#[tokio::test]
async fn inmemory_default_works() {
    let client = InMemoryQuotaClient::default();
    let status: QuotaStatus = client.check("test", 1).await.unwrap();
    assert!(status.allowed);
}

// ---------------------------------------------------------------------------
// InMemoryQuotaClient — full flow
// ---------------------------------------------------------------------------

#[tokio::test]
async fn inmemory_full_flow_check_increment_check() {
    let client = InMemoryQuotaClient::new();

    // Initially allowed
    let status: QuotaStatus = client.check("api-calls", 500).await.unwrap();
    assert!(status.allowed);
    assert_eq!(status.remaining, 1000);

    // Consume some
    let usage: QuotaUsage = client.increment("api-calls", 500).await.unwrap();
    assert_eq!(usage.used, 500);

    // Still allowed for 500
    let status: QuotaStatus = client.check("api-calls", 500).await.unwrap();
    assert!(status.allowed);
    assert_eq!(status.remaining, 500);

    // Not allowed for 501
    let status: QuotaStatus = client.check("api-calls", 501).await.unwrap();
    assert!(!status.allowed);

    // Increment more
    let _: QuotaUsage = client.increment("api-calls", 400).await.unwrap();
    let usage: QuotaUsage = client.get_usage("api-calls").await.unwrap();
    assert_eq!(usage.used, 900);

    // Only 100 remaining
    let status: QuotaStatus = client.check("api-calls", 100).await.unwrap();
    assert!(status.allowed);
    assert_eq!(status.remaining, 100);
}

// ---------------------------------------------------------------------------
// CachedQuotaClient — caching behavior
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cached_client_delegates_check() {
    let inner = InMemoryQuotaClient::new();
    let _: QuotaUsage = inner.increment("q", 100).await.unwrap();

    let cached = CachedQuotaClient::new(inner, Duration::from_secs(60));
    let status: QuotaStatus = cached.check("q", 50).await.unwrap();
    assert!(status.allowed);
    assert_eq!(status.remaining, 900);
}

#[tokio::test]
async fn cached_client_delegates_increment() {
    let inner = InMemoryQuotaClient::new();
    let cached = CachedQuotaClient::new(inner, Duration::from_secs(60));

    let usage: QuotaUsage = cached.increment("q", 250).await.unwrap();
    assert_eq!(usage.used, 250);
}

#[tokio::test]
async fn cached_client_delegates_get_usage() {
    let inner = InMemoryQuotaClient::new();
    let _: QuotaUsage = inner.increment("q", 77).await.unwrap();

    let cached = CachedQuotaClient::new(inner, Duration::from_secs(60));
    let usage: QuotaUsage = cached.get_usage("q").await.unwrap();
    assert_eq!(usage.used, 77);
}

#[tokio::test]
async fn cached_client_caches_policy() {
    let inner = InMemoryQuotaClient::new();
    inner.set_policy(
        "cached-q",
        QuotaPolicy {
            quota_id: "cached-q".to_string(),
            limit: 9999,
            period: QuotaPeriod::Monthly,
            reset_strategy: "fixed".to_string(),
        },
    );

    let cached = CachedQuotaClient::new(inner, Duration::from_secs(60));

    // First call fetches
    let p1: QuotaPolicy = cached.get_policy("cached-q").await.unwrap();
    assert_eq!(p1.limit, 9999);

    // Second call should return cached value
    let p2: QuotaPolicy = cached.get_policy("cached-q").await.unwrap();
    assert_eq!(p1, p2);
}

#[tokio::test]
async fn cached_client_expired_cache_refetches() {
    let inner = InMemoryQuotaClient::new();
    let cached = CachedQuotaClient::new(inner, Duration::from_millis(1));

    let p1: QuotaPolicy = cached.get_policy("q").await.unwrap();
    tokio::time::sleep(Duration::from_millis(10)).await;
    let p2: QuotaPolicy = cached.get_policy("q").await.unwrap();
    // Both succeed — cache expired and refetched
    assert_eq!(p1, p2);
}

// ---------------------------------------------------------------------------
// QuotaStatus reset_at is in the future
// ---------------------------------------------------------------------------

#[tokio::test]
async fn quota_status_reset_at_in_future() {
    let client = InMemoryQuotaClient::new();
    let status: QuotaStatus = client.check("q", 1).await.unwrap();
    assert!(status.reset_at > chrono::Utc::now() - chrono::Duration::seconds(5));
}
