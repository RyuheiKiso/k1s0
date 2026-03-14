use std::collections::HashMap;

use async_trait::async_trait;
use k1s0_health::{
    CheckResult, CompositeHealthChecker, HealthCheck, HealthError, HealthResponse, HealthStatus,
    HealthzResponse,
};

// ─── Test Helpers ───────────────────────────────────────────────────────────

struct AlwaysHealthy;

#[async_trait]
impl HealthCheck for AlwaysHealthy {
    fn name(&self) -> &str {
        "always-healthy"
    }
    async fn check(&self) -> Result<(), HealthError> {
        Ok(())
    }
}

struct AlwaysUnhealthy;

#[async_trait]
impl HealthCheck for AlwaysUnhealthy {
    fn name(&self) -> &str {
        "always-unhealthy"
    }
    async fn check(&self) -> Result<(), HealthError> {
        Err(HealthError::CheckFailed("service is down".to_string()))
    }
}

struct NamedCheck {
    name: String,
    healthy: bool,
}

impl NamedCheck {
    fn new(name: &str, healthy: bool) -> Self {
        Self {
            name: name.to_string(),
            healthy,
        }
    }
}

#[async_trait]
impl HealthCheck for NamedCheck {
    fn name(&self) -> &str {
        &self.name
    }
    async fn check(&self) -> Result<(), HealthError> {
        if self.healthy {
            Ok(())
        } else {
            Err(HealthError::CheckFailed(format!("{} is down", self.name)))
        }
    }
}

struct SlowCheck {
    delay: std::time::Duration,
}

#[async_trait]
impl HealthCheck for SlowCheck {
    fn name(&self) -> &str {
        "slow-check"
    }
    async fn check(&self) -> Result<(), HealthError> {
        tokio::time::sleep(self.delay).await;
        Ok(())
    }
}

struct TimeoutCheck {
    delay: std::time::Duration,
}

#[async_trait]
impl HealthCheck for TimeoutCheck {
    fn name(&self) -> &str {
        "timeout-check"
    }
    async fn check(&self) -> Result<(), HealthError> {
        tokio::time::sleep(self.delay).await;
        Err(HealthError::Timeout("check timed out".to_string()))
    }
}

// ─── Single Check ───────────────────────────────────────────────────────────

// ヘルシーなチェックを1件だけ登録した場合に全体ステータスが Healthy になることを確認する。
#[tokio::test]
async fn single_healthy_check_returns_healthy() {
    let mut checker = CompositeHealthChecker::new();
    checker.add_check(Box::new(AlwaysHealthy));

    let response = checker.run_all().await;

    assert_eq!(response.status, HealthStatus::Healthy);
    assert_eq!(response.checks.len(), 1);
    assert_eq!(
        response.checks["always-healthy"].status,
        HealthStatus::Healthy
    );
    assert!(response.checks["always-healthy"].message.is_none());
}

// 異常なチェックを1件だけ登録した場合に全体ステータスが Unhealthy になることを確認する。
#[tokio::test]
async fn single_unhealthy_check_returns_unhealthy() {
    let mut checker = CompositeHealthChecker::new();
    checker.add_check(Box::new(AlwaysUnhealthy));

    let response = checker.run_all().await;

    assert_eq!(response.status, HealthStatus::Unhealthy);
    assert_eq!(response.checks.len(), 1);
    assert_eq!(
        response.checks["always-unhealthy"].status,
        HealthStatus::Unhealthy
    );
    assert!(response.checks["always-unhealthy"].message.is_some());
}

// ─── CompositeHealthChecker ─────────────────────────────────────────────────

// 全チェックがヘルシーな場合に全体ステータスが Healthy になることを確認する。
#[tokio::test]
async fn all_healthy_checks_returns_up() {
    let mut checker = CompositeHealthChecker::new();
    checker.add_check(Box::new(NamedCheck::new("db", true)));
    checker.add_check(Box::new(NamedCheck::new("cache", true)));
    checker.add_check(Box::new(NamedCheck::new("queue", true)));

    let response = checker.run_all().await;

    assert_eq!(response.status, HealthStatus::Healthy);
    assert_eq!(response.checks.len(), 3);
    for (_, result) in &response.checks {
        assert_eq!(result.status, HealthStatus::Healthy);
    }
}

// 1件でも異常なチェックがある場合に全体ステータスが Unhealthy になることを確認する。
#[tokio::test]
async fn one_unhealthy_check_returns_down() {
    let mut checker = CompositeHealthChecker::new();
    checker.add_check(Box::new(NamedCheck::new("db", true)));
    checker.add_check(Box::new(NamedCheck::new("cache", false)));
    checker.add_check(Box::new(NamedCheck::new("queue", true)));

    let response = checker.run_all().await;

    assert_eq!(response.status, HealthStatus::Unhealthy);
    assert_eq!(response.checks.len(), 3);
    assert_eq!(response.checks["db"].status, HealthStatus::Healthy);
    assert_eq!(response.checks["cache"].status, HealthStatus::Unhealthy);
    assert_eq!(response.checks["queue"].status, HealthStatus::Healthy);
}

// 複数の異常チェックがある場合でも全体ステータスが Unhealthy になることを確認する。
#[tokio::test]
async fn multiple_unhealthy_checks_returns_down() {
    let mut checker = CompositeHealthChecker::new();
    checker.add_check(Box::new(NamedCheck::new("db", false)));
    checker.add_check(Box::new(NamedCheck::new("cache", false)));
    checker.add_check(Box::new(NamedCheck::new("queue", true)));

    let response = checker.run_all().await;

    assert_eq!(response.status, HealthStatus::Unhealthy);
    assert_eq!(response.checks["db"].status, HealthStatus::Unhealthy);
    assert_eq!(response.checks["cache"].status, HealthStatus::Unhealthy);
    assert_eq!(response.checks["queue"].status, HealthStatus::Healthy);
}

// チェックが1件も登録されていない場合に全体ステータスが Healthy になることを確認する。
#[tokio::test]
async fn empty_checker_returns_healthy() {
    let checker = CompositeHealthChecker::new();
    let response = checker.run_all().await;

    assert_eq!(response.status, HealthStatus::Healthy);
    assert!(response.checks.is_empty());
}

// Default トレイトで生成したチェッカーがチェックなし・Healthy 状態であることを確認する。
#[tokio::test]
async fn default_trait_creates_empty_checker() {
    let checker = CompositeHealthChecker::default();
    let response = checker.run_all().await;

    assert_eq!(response.status, HealthStatus::Healthy);
    assert!(response.checks.is_empty());
}

// ─── Timeout Behavior ──────────────────────────────────────────────────────

// 遅延があるチェックでも完了まで待機してヘルシー結果を返すことを確認する。
#[tokio::test]
async fn slow_check_still_completes() {
    let mut checker = CompositeHealthChecker::new();
    checker.add_check(Box::new(SlowCheck {
        delay: std::time::Duration::from_millis(50),
    }));

    let response = checker.run_all().await;
    assert_eq!(response.status, HealthStatus::Healthy);
    assert_eq!(response.checks["slow-check"].status, HealthStatus::Healthy);
}

// タイムアウトエラーが Unhealthy として記録され、メッセージに "timed out" が含まれることを確認する。
#[tokio::test]
async fn timeout_error_reported_as_unhealthy() {
    let mut checker = CompositeHealthChecker::new();
    checker.add_check(Box::new(TimeoutCheck {
        delay: std::time::Duration::from_millis(10),
    }));

    let response = checker.run_all().await;
    assert_eq!(response.status, HealthStatus::Unhealthy);
    let result = &response.checks["timeout-check"];
    assert_eq!(result.status, HealthStatus::Unhealthy);
    assert!(result.message.as_ref().unwrap().contains("timed out"));
}

// ─── readyz / healthz ───────────────────────────────────────────────────────

// readyz が run_all と同じ結果を返すことを確認する。
#[tokio::test]
async fn readyz_returns_same_as_run_all() {
    let mut checker = CompositeHealthChecker::new();
    checker.add_check(Box::new(AlwaysHealthy));

    let response = checker.readyz().await;
    assert_eq!(response.status, HealthStatus::Healthy);
    assert_eq!(response.checks.len(), 1);
}

// 異常チェックがある場合に readyz が Unhealthy を返すことを確認する。
#[tokio::test]
async fn readyz_with_failing_check_returns_unhealthy() {
    let mut checker = CompositeHealthChecker::new();
    checker.add_check(Box::new(AlwaysUnhealthy));

    let response = checker.readyz().await;
    assert_eq!(response.status, HealthStatus::Unhealthy);
}

// healthz が常に status="ok" を返すことを確認する。
#[test]
fn healthz_always_returns_ok() {
    let checker = CompositeHealthChecker::new();
    let response = checker.healthz();
    assert_eq!(response.status, "ok");
}

// 異常チェックが登録されていても healthz は常に status="ok" を返すことを確認する。
#[test]
fn healthz_returns_ok_regardless_of_checks() {
    let mut checker = CompositeHealthChecker::new();
    checker.add_check(Box::new(AlwaysUnhealthy));

    // healthz is a liveness probe, should always be ok
    let response = checker.healthz();
    assert_eq!(response.status, "ok");
}

// ─── HealthResponse / HealthzResponse Construction ──────────────────────────

// HealthResponse を手動構築したとき各フィールドが正しく設定されることを確認する。
#[test]
fn health_response_manual_construction() {
    let mut checks = HashMap::new();
    checks.insert(
        "db".to_string(),
        CheckResult {
            status: HealthStatus::Healthy,
            message: None,
        },
    );
    checks.insert(
        "cache".to_string(),
        CheckResult {
            status: HealthStatus::Unhealthy,
            message: Some("connection refused".to_string()),
        },
    );

    let response = HealthResponse {
        status: HealthStatus::Unhealthy,
        checks,
        timestamp: "1234567890".to_string(),
    };

    assert_eq!(response.status, HealthStatus::Unhealthy);
    assert_eq!(response.checks.len(), 2);
    assert_eq!(response.checks["db"].status, HealthStatus::Healthy);
    assert!(response.checks["db"].message.is_none());
    assert_eq!(response.checks["cache"].status, HealthStatus::Unhealthy);
    assert_eq!(
        response.checks["cache"].message.as_deref(),
        Some("connection refused")
    );
    assert_eq!(response.timestamp, "1234567890");
}

// HealthzResponse を手動構築したとき status フィールドが正しく設定されることを確認する。
#[test]
fn healthz_response_manual_construction() {
    let response = HealthzResponse {
        status: "ok".to_string(),
    };
    assert_eq!(response.status, "ok");
}

// ─── HealthStatus Equality ──────────────────────────────────────────────────

// HealthStatus の等値・非等値比較が正しく動作することを確認する。
#[test]
fn health_status_equality() {
    assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
    assert_eq!(HealthStatus::Unhealthy, HealthStatus::Unhealthy);
    assert_eq!(HealthStatus::Degraded, HealthStatus::Degraded);
    assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
    assert_ne!(HealthStatus::Unhealthy, HealthStatus::Degraded);
}

// HealthStatus を clone した結果が元の値と等しいことを確認する。
#[test]
fn health_status_clone() {
    let status = HealthStatus::Healthy;
    let cloned = status.clone();
    assert_eq!(status, cloned);
}

// HealthStatus の Debug 表示が期待する文字列になることを確認する。
#[test]
fn health_status_debug() {
    let debug = format!("{:?}", HealthStatus::Healthy);
    assert_eq!(debug, "Healthy");
}

// ─── HealthError Display ────────────────────────────────────────────────────

// HealthError の Display 出力が期待するメッセージフォーマットになることを確認する。
#[test]
fn health_error_display() {
    let err = HealthError::CheckFailed("db down".to_string());
    assert_eq!(err.to_string(), "health check failed: db down");

    let err = HealthError::Timeout("5s exceeded".to_string());
    assert_eq!(err.to_string(), "timeout: 5s exceeded");
}

// ─── Timestamp ──────────────────────────────────────────────────────────────

// ヘルスチェック実行後のレスポンスにタイムスタンプが含まれ、数値として解析できることを確認する。
#[tokio::test]
async fn response_has_non_empty_timestamp() {
    let checker = CompositeHealthChecker::new();
    let response = checker.run_all().await;
    assert!(!response.timestamp.is_empty());
    // Should be parseable as a number (unix timestamp seconds)
    let _ts: u64 = response.timestamp.parse().expect("timestamp should be numeric");
}

// ─── Check Result Message ───────────────────────────────────────────────────

// 異常チェックの結果にエラーメッセージが含まれることを確認する。
#[tokio::test]
async fn unhealthy_check_includes_error_message() {
    let mut checker = CompositeHealthChecker::new();
    checker.add_check(Box::new(AlwaysUnhealthy));

    let response = checker.run_all().await;
    let result = &response.checks["always-unhealthy"];
    assert!(result.message.is_some());
    assert!(result.message.as_ref().unwrap().contains("service is down"));
}

// ヘルシーなチェックの結果にメッセージが含まれないことを確認する。
#[tokio::test]
async fn healthy_check_has_no_message() {
    let mut checker = CompositeHealthChecker::new();
    checker.add_check(Box::new(AlwaysHealthy));

    let response = checker.run_all().await;
    let result = &response.checks["always-healthy"];
    assert!(result.message.is_none());
}
