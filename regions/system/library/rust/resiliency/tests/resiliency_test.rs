use k1s0_resiliency::{
    BulkheadConfig, CircuitBreakerConfig, ExponentialBackoff, ResiliencyDecorator, ResiliencyError,
    ResiliencyMetrics, ResiliencyPolicy, RetryConfig,
};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Test error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct TestError(String);

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TestError {}

// ===========================================================================
// ResiliencyPolicyBuilder tests
// ===========================================================================

#[test]
fn builder_default_produces_empty_policy() {
    let policy = ResiliencyPolicy::builder().build();
    assert!(policy.retry.is_none());
    assert!(policy.circuit_breaker.is_none());
    assert!(policy.bulkhead.is_none());
    assert!(policy.timeout.is_none());
    assert!(policy.backoff.is_none());
    assert!(policy.retryable_errors.is_empty());
}

#[test]
fn builder_with_retry() {
    let policy = ResiliencyPolicy::builder()
        .retry(RetryConfig::new(5))
        .build();
    assert!(policy.retry.is_some());
    assert_eq!(policy.retry.unwrap().max_attempts, 5);
}

#[test]
fn builder_with_circuit_breaker() {
    let policy = ResiliencyPolicy::builder()
        .circuit_breaker(CircuitBreakerConfig {
            failure_threshold: 10,
            success_threshold: 3,
            timeout: Duration::from_secs(30),
        })
        .build();
    let cb = policy.circuit_breaker.unwrap();
    assert_eq!(cb.failure_threshold, 10);
    assert_eq!(cb.success_threshold, 3);
    assert_eq!(cb.timeout, Duration::from_secs(30));
}

#[test]
fn builder_with_bulkhead() {
    let policy = ResiliencyPolicy::builder()
        .bulkhead(BulkheadConfig {
            max_concurrent_calls: 20,
            max_wait_duration: Duration::from_millis(500),
        })
        .build();
    let bh = policy.bulkhead.unwrap();
    assert_eq!(bh.max_concurrent_calls, 20);
    assert_eq!(bh.max_wait_duration, Duration::from_millis(500));
}

#[test]
fn builder_with_timeout() {
    let policy = ResiliencyPolicy::builder()
        .timeout(Duration::from_secs(5))
        .build();
    assert_eq!(policy.timeout, Some(Duration::from_secs(5)));
}

#[test]
fn builder_with_backoff() {
    let policy = ResiliencyPolicy::builder()
        .backoff(ExponentialBackoff::new(
            Duration::from_millis(100),
            Duration::from_secs(10),
        ))
        .build();
    let bo = policy.backoff.unwrap();
    assert_eq!(bo.initial_delay, Duration::from_millis(100));
    assert_eq!(bo.max_delay, Duration::from_secs(10));
}

#[test]
fn builder_with_retryable_errors() {
    let policy = ResiliencyPolicy::builder()
        .retryable_errors(["timeout", "network_error"])
        .build();
    assert_eq!(policy.retryable_errors.len(), 2);
    assert!(policy.retryable_errors.contains(&"timeout".to_string()));
    assert!(policy
        .retryable_errors
        .contains(&"network_error".to_string()));
}

#[test]
fn builder_full_chain() {
    let policy = ResiliencyPolicy::builder()
        .retry(RetryConfig::new(3))
        .circuit_breaker(CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
        })
        .bulkhead(BulkheadConfig {
            max_concurrent_calls: 10,
            max_wait_duration: Duration::from_millis(200),
        })
        .timeout(Duration::from_secs(5))
        .backoff(ExponentialBackoff::new(
            Duration::from_millis(50),
            Duration::from_secs(5),
        ))
        .retryable_errors(["transient"])
        .build();

    assert!(policy.retry.is_some());
    assert!(policy.circuit_breaker.is_some());
    assert!(policy.bulkhead.is_some());
    assert!(policy.timeout.is_some());
    assert!(policy.backoff.is_some());
    assert_eq!(policy.retryable_errors.len(), 1);
}

// ===========================================================================
// ExponentialBackoff tests
// ===========================================================================

#[test]
fn exponential_backoff_compute_delay_attempt_zero() {
    let bo = ExponentialBackoff::new(Duration::from_millis(100), Duration::from_secs(10));
    let delay = bo.compute_delay(0, 2.0);
    assert_eq!(delay, Duration::from_millis(100));
}

#[test]
fn exponential_backoff_compute_delay_grows() {
    let bo = ExponentialBackoff::new(Duration::from_millis(100), Duration::from_secs(10));
    let d0 = bo.compute_delay(0, 2.0);
    let d1 = bo.compute_delay(1, 2.0);
    let d2 = bo.compute_delay(2, 2.0);
    assert!(d1 > d0);
    assert!(d2 > d1);
}

#[test]
fn exponential_backoff_capped_at_max() {
    let bo = ExponentialBackoff::new(Duration::from_millis(100), Duration::from_millis(500));
    let delay = bo.compute_delay(10, 2.0);
    assert!(delay <= Duration::from_millis(500));
}

// ===========================================================================
// ResiliencyPolicy::decorate
// ===========================================================================

#[tokio::test]
async fn policy_decorate_creates_decorator() {
    let policy = ResiliencyPolicy::builder().build();
    let decorator = policy.decorate();
    let val = decorator
        .execute(|| async { Ok::<_, TestError>(42) })
        .await
        .unwrap();
    assert_eq!(val, 42);
}

// ===========================================================================
// ResiliencyDecorator — simple execute
// ===========================================================================

#[tokio::test]
async fn decorator_execute_success() {
    let policy = ResiliencyPolicy::builder().build();
    let decorator = ResiliencyDecorator::new(policy);
    let result = decorator
        .execute(|| async { Ok::<_, TestError>("hello") })
        .await;
    assert_eq!(result.unwrap(), "hello");
}

#[tokio::test]
async fn decorator_execute_failure_no_retry() {
    let policy = ResiliencyPolicy::builder().build();
    let decorator = ResiliencyDecorator::new(policy);
    let result = decorator
        .execute(|| async { Err::<i32, _>(TestError("fail".into())) })
        .await;
    assert!(result.is_err());
}

// ===========================================================================
// ResiliencyDecorator — retry behavior
// ===========================================================================

#[tokio::test]
async fn decorator_retry_succeeds_after_failures() {
    let policy = ResiliencyPolicy::builder()
        .retry(RetryConfig::new(3).with_jitter(false))
        .build();
    let decorator = ResiliencyDecorator::new(policy);
    let counter = Arc::new(AtomicU32::new(0));
    let c = counter.clone();

    let result = decorator
        .execute(move || {
            let c = c.clone();
            async move {
                let n = c.fetch_add(1, Ordering::SeqCst);
                if n < 2 {
                    Err(TestError("transient".into()))
                } else {
                    Ok(100)
                }
            }
        })
        .await;

    assert_eq!(result.unwrap(), 100);
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn decorator_retry_exhausted() {
    let policy = ResiliencyPolicy::builder()
        .retry(RetryConfig::new(2).with_jitter(false))
        .build();
    let decorator = ResiliencyDecorator::new(policy);

    let result = decorator
        .execute(|| async { Err::<i32, _>(TestError("always fail".into())) })
        .await;

    assert!(matches!(
        result,
        Err(ResiliencyError::MaxRetriesExceeded { attempts: 2, .. })
    ));
}

#[tokio::test]
async fn decorator_retry_metrics_recorded() {
    let policy = ResiliencyPolicy::builder()
        .retry(RetryConfig::new(3).with_jitter(false))
        .build();
    let decorator = ResiliencyDecorator::new(policy);
    let counter = Arc::new(AtomicU32::new(0));
    let c = counter.clone();

    let _ = decorator
        .execute(move || {
            let c = c.clone();
            async move {
                let n = c.fetch_add(1, Ordering::SeqCst);
                if n < 2 {
                    Err(TestError("retry me".into()))
                } else {
                    Ok(())
                }
            }
        })
        .await;

    assert!(decorator.metrics().retry_attempts() >= 2);
}

// ===========================================================================
// ResiliencyDecorator — retry with backoff
// ===========================================================================

#[tokio::test]
async fn decorator_retry_with_backoff() {
    let policy = ResiliencyPolicy::builder()
        .retry(RetryConfig::new(3).with_jitter(false))
        .backoff(ExponentialBackoff::new(
            Duration::from_millis(1),
            Duration::from_millis(10),
        ))
        .build();
    let decorator = ResiliencyDecorator::new(policy);
    let counter = Arc::new(AtomicU32::new(0));
    let c = counter.clone();

    let result = decorator
        .execute(move || {
            let c = c.clone();
            async move {
                let n = c.fetch_add(1, Ordering::SeqCst);
                if n < 2 {
                    Err(TestError("retry".into()))
                } else {
                    Ok(42)
                }
            }
        })
        .await;

    assert_eq!(result.unwrap(), 42);
}

// ===========================================================================
// ResiliencyDecorator — retryable error filtering
// ===========================================================================

#[tokio::test]
async fn decorator_non_retryable_error_stops_immediately() {
    let policy = ResiliencyPolicy::builder()
        .retry(RetryConfig::new(5).with_jitter(false))
        .retryable_errors(["network"])
        .build();
    let decorator = ResiliencyDecorator::new(policy);
    let counter = Arc::new(AtomicU32::new(0));
    let c = counter.clone();

    let _ = decorator
        .execute(move || {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err::<i32, _>(TestError("validation error".into()))
            }
        })
        .await;

    // Should stop after first attempt since "validation error" doesn't match "network"
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn decorator_retryable_error_retries() {
    let policy = ResiliencyPolicy::builder()
        .retry(RetryConfig::new(3).with_jitter(false))
        .retryable_errors(["network"])
        .build();
    let decorator = ResiliencyDecorator::new(policy);
    let counter = Arc::new(AtomicU32::new(0));
    let c = counter.clone();

    let _ = decorator
        .execute(move || {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err::<i32, _>(TestError("network timeout".into()))
            }
        })
        .await;

    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

// ===========================================================================
// ResiliencyDecorator — timeout
// ===========================================================================

#[tokio::test]
async fn decorator_timeout_triggers() {
    let policy = ResiliencyPolicy::builder()
        .timeout(Duration::from_millis(50))
        .build();
    let decorator = ResiliencyDecorator::new(policy);

    let result = decorator
        .execute(|| async {
            tokio::time::sleep(Duration::from_secs(2)).await;
            Ok::<_, TestError>(42)
        })
        .await;

    assert!(matches!(result, Err(ResiliencyError::Timeout { .. })));
}

#[tokio::test]
async fn decorator_timeout_metrics_recorded() {
    let policy = ResiliencyPolicy::builder()
        .timeout(Duration::from_millis(10))
        .build();
    let decorator = ResiliencyDecorator::new(policy);

    let _ = decorator
        .execute(|| async {
            tokio::time::sleep(Duration::from_secs(1)).await;
            Ok::<_, TestError>(())
        })
        .await;

    assert_eq!(decorator.metrics().timeout_events(), 1);
}

#[tokio::test]
async fn decorator_no_timeout_when_fast() {
    let policy = ResiliencyPolicy::builder()
        .timeout(Duration::from_secs(10))
        .build();
    let decorator = ResiliencyDecorator::new(policy);

    let result = decorator
        .execute(|| async { Ok::<_, TestError>(42) })
        .await;

    assert_eq!(result.unwrap(), 42);
    assert_eq!(decorator.metrics().timeout_events(), 0);
}

// ===========================================================================
// ResiliencyDecorator — circuit breaker
// ===========================================================================

#[tokio::test]
async fn decorator_circuit_breaker_opens_after_threshold() {
    let policy = ResiliencyPolicy::builder()
        .circuit_breaker(CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 1,
            timeout: Duration::from_secs(60),
        })
        .retry(RetryConfig::new(1).with_jitter(false))
        .build();
    let decorator = ResiliencyDecorator::new(policy);

    // Trigger 3 failures to open the circuit
    for _ in 0..3 {
        let _ = decorator
            .execute(|| async { Err::<i32, _>(TestError("fail".into())) })
            .await;
    }

    // Next call should be rejected by circuit breaker
    let result = decorator.execute(|| async { Ok::<_, TestError>(42) }).await;
    assert!(matches!(result, Err(ResiliencyError::CircuitOpen { .. })));
}

#[tokio::test]
async fn decorator_circuit_breaker_stays_closed_on_success() {
    let policy = ResiliencyPolicy::builder()
        .circuit_breaker(CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 1,
            timeout: Duration::from_secs(60),
        })
        .build();
    let decorator = ResiliencyDecorator::new(policy);

    // Successful calls should keep circuit closed
    for _ in 0..10 {
        let result = decorator
            .execute(|| async { Ok::<_, TestError>(1) })
            .await;
        assert!(result.is_ok());
    }

    assert_eq!(decorator.metrics().circuit_state_code(), 0); // 0 = closed
}

// ===========================================================================
// ResiliencyDecorator — bulkhead
// ===========================================================================

#[tokio::test]
async fn decorator_bulkhead_limits_concurrency() {
    let policy = ResiliencyPolicy::builder()
        .bulkhead(BulkheadConfig {
            max_concurrent_calls: 1,
            max_wait_duration: Duration::from_millis(50),
        })
        .build();
    let decorator = Arc::new(ResiliencyDecorator::new(policy));

    let d1 = decorator.clone();
    let handle = tokio::spawn(async move {
        d1.execute(|| async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok::<_, TestError>(1)
        })
        .await
    });

    // Wait for the first task to acquire the permit
    tokio::time::sleep(Duration::from_millis(10)).await;

    let result = decorator
        .execute(|| async { Ok::<_, TestError>(2) })
        .await;
    assert!(matches!(result, Err(ResiliencyError::BulkheadFull { .. })));

    let _ = handle.await;
}

#[tokio::test]
async fn decorator_bulkhead_rejection_metrics() {
    let policy = ResiliencyPolicy::builder()
        .bulkhead(BulkheadConfig {
            max_concurrent_calls: 1,
            max_wait_duration: Duration::from_millis(10),
        })
        .build();
    let decorator = Arc::new(ResiliencyDecorator::new(policy));

    let d1 = decorator.clone();
    let handle = tokio::spawn(async move {
        d1.execute(|| async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok::<_, TestError>(())
        })
        .await
    });

    tokio::time::sleep(Duration::from_millis(10)).await;

    let _ = decorator
        .execute(|| async { Ok::<_, TestError>(()) })
        .await;

    assert_eq!(decorator.metrics().bulkhead_rejections(), 1);

    let _ = handle.await;
}

// ===========================================================================
// ResiliencyMetrics tests
// ===========================================================================

#[test]
fn metrics_initial_values() {
    let m = ResiliencyMetrics::new();
    assert_eq!(m.retry_attempts(), 0);
    assert_eq!(m.circuit_open_events(), 0);
    assert_eq!(m.bulkhead_rejections(), 0);
    assert_eq!(m.timeout_events(), 0);
}

#[test]
fn metrics_record_retry() {
    let m = ResiliencyMetrics::new();
    m.record_retry_attempt();
    m.record_retry_attempt();
    assert_eq!(m.retry_attempts(), 2);
}

#[test]
fn metrics_record_circuit_open() {
    let m = ResiliencyMetrics::new();
    m.record_circuit_open();
    assert_eq!(m.circuit_open_events(), 1);
}

#[test]
fn metrics_record_bulkhead_rejection() {
    let m = ResiliencyMetrics::new();
    m.record_bulkhead_rejection();
    m.record_bulkhead_rejection();
    m.record_bulkhead_rejection();
    assert_eq!(m.bulkhead_rejections(), 3);
}

#[test]
fn metrics_record_timeout() {
    let m = ResiliencyMetrics::new();
    m.record_timeout();
    assert_eq!(m.timeout_events(), 1);
}

#[test]
fn metrics_circuit_state_transitions() {
    let m = ResiliencyMetrics::new();
    m.set_circuit_closed();
    assert_eq!(m.circuit_state_code(), 0);

    m.set_circuit_open();
    assert_eq!(m.circuit_state_code(), 1);

    m.set_circuit_half_open();
    assert_eq!(m.circuit_state_code(), 2);

    m.set_circuit_closed();
    assert_eq!(m.circuit_state_code(), 0);
}

// ===========================================================================
// ResiliencyDecorator — with_metrics
// ===========================================================================

#[tokio::test]
async fn decorator_with_shared_metrics() {
    let metrics = Arc::new(ResiliencyMetrics::new());
    let policy = ResiliencyPolicy::builder()
        .timeout(Duration::from_millis(5))
        .build();
    let decorator = ResiliencyDecorator::with_metrics(policy, metrics.clone());

    let _ = decorator
        .execute(|| async {
            tokio::time::sleep(Duration::from_secs(1)).await;
            Ok::<_, TestError>(())
        })
        .await;

    assert_eq!(metrics.timeout_events(), 1);
}

// ===========================================================================
// ResiliencyError display
// ===========================================================================

#[test]
fn error_max_retries_display() {
    let err = ResiliencyError::MaxRetriesExceeded {
        attempts: 3,
        last_error: Box::new(TestError("connection refused".into())),
    };
    let msg = err.to_string();
    assert!(msg.contains("3"));
    assert!(msg.contains("connection refused"));
}

#[test]
fn error_circuit_open_display() {
    let err = ResiliencyError::CircuitOpen {
        remaining_duration: Duration::from_secs(30),
    };
    let msg = err.to_string();
    assert!(msg.contains("circuit breaker"));
}

#[test]
fn error_bulkhead_full_display() {
    let err = ResiliencyError::BulkheadFull {
        max_concurrent: 10,
    };
    let msg = err.to_string();
    assert!(msg.contains("10"));
}

#[test]
fn error_timeout_display() {
    let err = ResiliencyError::Timeout {
        after: Duration::from_secs(5),
    };
    let msg = err.to_string();
    assert!(msg.contains("timed out"));
}
