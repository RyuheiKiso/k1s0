#![allow(clippy::unwrap_used)]
use k1s0_circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, CircuitBreakerState,
};
use std::sync::Arc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fast_config(
    failure_threshold: u32,
    success_threshold: u32,
    timeout_ms: u64,
) -> CircuitBreakerConfig {
    CircuitBreakerConfig {
        failure_threshold,
        success_threshold,
        timeout: Duration::from_millis(timeout_ms),
    }
}

fn default_config() -> CircuitBreakerConfig {
    fast_config(3, 2, 100)
}

// ===========================================================================
// State transition tests
// ===========================================================================

// サーキットブレーカーの初期状態が Closed であることを確認する。
#[tokio::test]
async fn initial_state_is_closed() {
    let cb = CircuitBreaker::new(default_config());
    assert_eq!(cb.state().await, CircuitBreakerState::Closed);
}

// 失敗回数が閾値に達した後に Closed から Open へ遷移することを確認する。
#[tokio::test]
async fn closed_to_open_after_failure_threshold() {
    let cb = CircuitBreaker::new(fast_config(3, 1, 100));

    // Two failures: still Closed
    cb.record_failure().await;
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitBreakerState::Closed);

    // Third failure: transitions to Open
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitBreakerState::Open);
}

// タイムアウト経過後に Open から HalfOpen へ遷移することを確認する。
#[tokio::test]
async fn open_to_half_open_after_timeout() {
    let cb = CircuitBreaker::new(fast_config(1, 1, 50));

    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitBreakerState::Open);

    tokio::time::sleep(Duration::from_millis(60)).await;
    assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);
}

// HalfOpen 状態で成功回数が閾値に達した後に Closed へ遷移することを確認する。
#[tokio::test]
async fn half_open_to_closed_after_success_threshold() {
    let cb = CircuitBreaker::new(fast_config(1, 2, 50));

    // Drive to HalfOpen
    cb.record_failure().await;
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);

    // One success: still HalfOpen
    cb.record_success().await;
    assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);

    // Second success: transitions to Closed
    cb.record_success().await;
    assert_eq!(cb.state().await, CircuitBreakerState::Closed);
}

// HalfOpen 状態で失敗が発生した場合に再び Open へ戻ることを確認する。
#[tokio::test]
async fn half_open_to_open_on_failure() {
    let cb = CircuitBreaker::new(fast_config(1, 2, 50));

    cb.record_failure().await;
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);

    // A single failure in HalfOpen should reopen
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitBreakerState::Open);
}

// Closed → Open → HalfOpen → Closed の完全な状態遷移サイクルを検証する。
#[tokio::test]
async fn full_cycle_closed_open_half_open_closed() {
    let cb = CircuitBreaker::new(fast_config(2, 1, 50));

    // Closed -> Open
    cb.record_failure().await;
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitBreakerState::Open);

    // Open -> HalfOpen (after timeout)
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);

    // HalfOpen -> Closed (after success threshold)
    cb.record_success().await;
    assert_eq!(cb.state().await, CircuitBreakerState::Closed);
}

// タイムアウト前は Open 状態が維持されることを確認する。
#[tokio::test]
async fn stays_open_before_timeout_elapses() {
    let cb = CircuitBreaker::new(fast_config(1, 1, 200));

    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitBreakerState::Open);

    // Check immediately -- should still be Open
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert_eq!(cb.state().await, CircuitBreakerState::Open);
}

// ===========================================================================
// call() tests
// ===========================================================================

// call が成功した場合に値が正しく返されることを確認する。
#[tokio::test]
async fn call_success_returns_value() {
    let cb = CircuitBreaker::new(default_config());
    let result: Result<i32, CircuitBreakerError<String>> = cb.call(|| async { Ok(42) }).await;
    assert_eq!(result.unwrap(), 42);
}

// call が失敗した場合に Inner エラーが返されることを確認する。
#[tokio::test]
async fn call_failure_returns_inner_error() {
    let cb = CircuitBreaker::new(default_config());
    let result: Result<i32, CircuitBreakerError<String>> =
        cb.call(|| async { Err("boom".to_string()) }).await;

    match result {
        Err(CircuitBreakerError::Inner(e)) => assert_eq!(e, "boom"),
        other => panic!("expected Inner error, got {:?}", other),
    }
}

// Open 状態のとき call がリジェクトされることを確認する。
#[tokio::test]
async fn call_rejected_when_open() {
    let cb = CircuitBreaker::new(default_config());

    for _ in 0..3 {
        cb.record_failure().await;
    }

    let result: Result<i32, CircuitBreakerError<String>> = cb.call(|| async { Ok(42) }).await;
    assert!(matches!(result, Err(CircuitBreakerError::Open)));
}

// HalfOpen 状態のとき call が許可され成功後に Closed へ遷移することを確認する。
#[tokio::test]
async fn call_allowed_in_half_open() {
    let cb = CircuitBreaker::new(fast_config(1, 1, 50));

    cb.record_failure().await;
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);

    let result: Result<i32, CircuitBreakerError<String>> = cb.call(|| async { Ok(99) }).await;
    assert_eq!(result.unwrap(), 99);
    // Success in HalfOpen with success_threshold=1 should close it
    assert_eq!(cb.state().await, CircuitBreakerState::Closed);
}

// HalfOpen 状態で call が失敗した場合に Open へ再遷移することを確認する。
#[tokio::test]
async fn call_failure_in_half_open_reopens() {
    let cb = CircuitBreaker::new(fast_config(1, 2, 50));

    cb.record_failure().await;
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);

    let result: Result<i32, CircuitBreakerError<String>> =
        cb.call(|| async { Err("fail".to_string()) }).await;
    assert!(matches!(result, Err(CircuitBreakerError::Inner(_))));

    assert_eq!(cb.state().await, CircuitBreakerState::Open);
}

// ===========================================================================
// Failure counting / threshold edge cases
// ===========================================================================

// 失敗回数が閾値未満の場合は Closed 状態が維持されることを確認する。
#[tokio::test]
async fn failures_below_threshold_stay_closed() {
    let cb = CircuitBreaker::new(fast_config(5, 1, 100));

    for _ in 0..4 {
        cb.record_failure().await;
    }
    assert_eq!(cb.state().await, CircuitBreakerState::Closed);
}

// 閾値が 1 の場合は最初の失敗で即座に Open になることを確認する。
#[tokio::test]
async fn threshold_of_one_opens_immediately() {
    let cb = CircuitBreaker::new(fast_config(1, 1, 100));
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitBreakerState::Open);
}

// Closed 状態での成功記録は状態に影響を与えないことを確認する。
#[tokio::test]
async fn successes_in_closed_do_not_affect_state() {
    let cb = CircuitBreaker::new(default_config());
    for _ in 0..10 {
        cb.record_success().await;
    }
    assert_eq!(cb.state().await, CircuitBreakerState::Closed);
}

// ===========================================================================
// Config validation / defaults
// ===========================================================================

// デフォルト設定が適切な値を持つことを確認する。
#[test]
fn default_config_has_sane_values() {
    let config = CircuitBreakerConfig::default();
    assert_eq!(config.failure_threshold, 5);
    assert_eq!(config.success_threshold, 3);
    assert_eq!(config.timeout, Duration::from_secs(30));
}

// 設定をクローンした場合に同じ値が保持されることを確認する。
#[test]
fn config_clone_is_equal() {
    let config = CircuitBreakerConfig {
        failure_threshold: 10,
        success_threshold: 5,
        timeout: Duration::from_secs(60),
    };
    let cloned = config.clone();
    assert_eq!(cloned.failure_threshold, 10);
    assert_eq!(cloned.success_threshold, 5);
    assert_eq!(cloned.timeout, Duration::from_secs(60));
}

// ===========================================================================
// Metrics tracking
// ===========================================================================

// メトリクスの初期状態が全てゼロかつ Closed であることを確認する。
#[tokio::test]
async fn metrics_initial_state() {
    let cb = CircuitBreaker::new(default_config());
    let m = cb.metrics();
    assert_eq!(m.success_count, 0);
    assert_eq!(m.failure_count, 0);
    assert_eq!(m.state, "Closed");
}

// メトリクスが成功数と失敗数を正しく記録することを確認する。
#[tokio::test]
async fn metrics_track_successes_and_failures() {
    let cb = CircuitBreaker::new(default_config());

    cb.record_success().await;
    cb.record_success().await;
    cb.record_failure().await;

    let m = cb.metrics();
    assert_eq!(m.success_count, 2);
    assert_eq!(m.failure_count, 1);
}

// メトリクスの状態フィールドが状態遷移に応じて更新されることを確認する。
#[tokio::test]
async fn metrics_reflect_state_changes() {
    let cb = CircuitBreaker::new(fast_config(1, 1, 50));

    let m = cb.metrics();
    assert_eq!(m.state, "Closed");

    cb.record_failure().await;
    let m = cb.metrics();
    assert_eq!(m.state, "Open");

    tokio::time::sleep(Duration::from_millis(60)).await;
    // Trigger state check
    let _ = cb.state().await;
    let m = cb.metrics();
    assert_eq!(m.state, "HalfOpen");

    cb.record_success().await;
    let m = cb.metrics();
    assert_eq!(m.state, "Closed");
}

// call を通じた成功と失敗がメトリクスに蓄積されることを確認する。
#[tokio::test]
async fn metrics_accumulate_through_call() {
    let cb = CircuitBreaker::new(default_config());

    let _: Result<i32, CircuitBreakerError<String>> = cb.call(|| async { Ok(1) }).await;
    let _: Result<i32, CircuitBreakerError<String>> =
        cb.call(|| async { Err("e".to_string()) }).await;

    let m = cb.metrics();
    assert_eq!(m.success_count, 1);
    assert_eq!(m.failure_count, 1);
}

// ===========================================================================
// Concurrent access safety
// ===========================================================================

// 並行して失敗を記録した場合に正しく Open へ遷移することを確認する。
#[tokio::test]
async fn concurrent_failures_transition_correctly() {
    let cb = Arc::new(CircuitBreaker::new(fast_config(10, 1, 100)));

    let mut handles = Vec::new();
    for _ in 0..10 {
        let cb_clone = cb.clone();
        handles.push(tokio::spawn(async move {
            cb_clone.record_failure().await;
        }));
    }
    for h in handles {
        h.await.unwrap();
    }

    assert_eq!(cb.state().await, CircuitBreakerState::Open);
    let m = cb.metrics();
    assert_eq!(m.failure_count, 10);
}

// Open 状態のとき並行する全ての call がリジェクトされることを確認する。
#[tokio::test]
async fn concurrent_calls_when_open_all_rejected() {
    let cb = Arc::new(CircuitBreaker::new(fast_config(1, 1, 5000)));
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitBreakerState::Open);

    let mut handles = Vec::new();
    for _ in 0..5 {
        let cb_clone = cb.clone();
        handles.push(tokio::spawn(async move {
            let r: Result<i32, CircuitBreakerError<String>> =
                cb_clone.call(|| async { Ok(1) }).await;
            r
        }));
    }

    for h in handles {
        let result = h.await.unwrap();
        assert!(matches!(result, Err(CircuitBreakerError::Open)));
    }
}

// ===========================================================================
// Error display
// ===========================================================================

// Open エラーの表示文字列が正しいことを確認する。
#[test]
fn error_display_open() {
    let err: CircuitBreakerError<String> = CircuitBreakerError::Open;
    assert_eq!(format!("{}", err), "circuit breaker is open");
}

// Inner エラーの表示文字列が内部エラーを含むことを確認する。
#[test]
fn error_display_inner() {
    let err: CircuitBreakerError<String> = CircuitBreakerError::Inner("db down".to_string());
    assert_eq!(format!("{}", err), "inner error: db down");
}

// エラーが Debug トレイトを実装していることを確認する。
#[test]
fn error_is_debug() {
    let err: CircuitBreakerError<String> = CircuitBreakerError::Open;
    let debug = format!("{:?}", err);
    assert!(debug.contains("Open"));
}
