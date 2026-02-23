use k1s0_retry::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState};
use k1s0_retry::error::RetryError;
use k1s0_retry::policy::RetryConfig;
use k1s0_retry::retry::with_retry;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_retry_succeeds_on_first_attempt() {
    let config = RetryConfig::new(3);
    let result: Result<&str, RetryError<String>> =
        with_retry(&config, || async { Ok("success") }).await;
    assert_eq!(result.unwrap(), "success");
}

#[tokio::test]
async fn test_retry_succeeds_on_third_attempt() {
    let counter = Arc::new(AtomicU32::new(0));
    let config = RetryConfig::new(3)
        .with_initial_delay(Duration::from_millis(1))
        .with_jitter(false);

    let counter_clone = counter.clone();
    let result: Result<&str, RetryError<String>> = with_retry(&config, move || {
        let c = counter_clone.clone();
        async move {
            let attempt = c.fetch_add(1, Ordering::SeqCst);
            if attempt < 2 {
                Err("not yet".to_string())
            } else {
                Ok("success")
            }
        }
    })
    .await;

    assert_eq!(result.unwrap(), "success");
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_retry_exhausted() {
    let config = RetryConfig::new(3)
        .with_initial_delay(Duration::from_millis(1))
        .with_jitter(false);

    let result: Result<&str, RetryError<String>> =
        with_retry(&config, || async { Err("always fails".to_string()) }).await;

    match result {
        Err(RetryError::ExhaustedRetries {
            attempts,
            last_error,
        }) => {
            assert_eq!(attempts, 3);
            assert_eq!(last_error, "always fails");
        }
        _ => panic!("ExhaustedRetries エラーが期待される"),
    }
}

#[test]
fn test_compute_delay_exponential() {
    let config = RetryConfig::new(5)
        .with_initial_delay(Duration::from_millis(100))
        .with_multiplier(2.0)
        .with_max_delay(Duration::from_secs(30))
        .with_jitter(false);

    let delay0 = config.compute_delay(0);
    let delay1 = config.compute_delay(1);
    let delay2 = config.compute_delay(2);

    assert_eq!(delay0.as_millis(), 100);
    assert_eq!(delay1.as_millis(), 200);
    assert_eq!(delay2.as_millis(), 400);
}

#[test]
fn test_compute_delay_no_jitter() {
    let config = RetryConfig::new(3)
        .with_initial_delay(Duration::from_millis(50))
        .with_multiplier(3.0)
        .with_jitter(false);

    // ジッターなしなので毎回同じ値
    let d1 = config.compute_delay(1);
    let d2 = config.compute_delay(1);
    assert_eq!(d1, d2);
    assert_eq!(d1.as_millis(), 150); // 50 * 3^1
}

#[tokio::test]
async fn test_circuit_breaker_opens_after_threshold() {
    let cb = CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout: Duration::from_secs(30),
    });

    assert!(!cb.is_open().await);

    cb.record_failure().await;
    cb.record_failure().await;
    assert!(!cb.is_open().await);

    cb.record_failure().await;
    assert!(cb.is_open().await);
}

#[tokio::test]
async fn test_circuit_breaker_transitions_to_half_open() {
    let cb = CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 1,
        timeout: Duration::from_millis(50),
    });

    cb.record_failure().await;
    cb.record_failure().await;
    assert!(cb.is_open().await);

    // タイムアウト待ち
    tokio::time::sleep(Duration::from_millis(60)).await;

    // タイムアウト後は HalfOpen に遷移し、is_open は false を返す
    assert!(!cb.is_open().await);
    assert_eq!(cb.get_state().await, CircuitBreakerState::HalfOpen);
}

#[tokio::test]
async fn test_circuit_breaker_closes_after_successes() {
    let cb = CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout: Duration::from_millis(50),
    });

    // Open 状態にする
    cb.record_failure().await;
    cb.record_failure().await;
    assert!(cb.is_open().await);

    // タイムアウト後 HalfOpen へ
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert!(!cb.is_open().await);
    assert_eq!(cb.get_state().await, CircuitBreakerState::HalfOpen);

    // HalfOpen で success_threshold 回成功 → Closed
    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitBreakerState::HalfOpen);
    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitBreakerState::Closed);
}
