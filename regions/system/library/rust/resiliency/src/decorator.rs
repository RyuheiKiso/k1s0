use std::future::Future;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::bulkhead::Bulkhead;
use crate::error::ResiliencyError;
use crate::metrics::ResiliencyMetrics;
use crate::policy::ResiliencyPolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct ResiliencyDecorator {
    policy: ResiliencyPolicy,
    metrics: Arc<ResiliencyMetrics>,
    bulkhead: Option<Bulkhead>,
    cb_state: Mutex<CircuitState>,
    cb_failure_count: AtomicU32,
    cb_success_count: AtomicU32,
    cb_last_failure_time: Mutex<Option<Instant>>,
}

impl ResiliencyDecorator {
    pub fn new(policy: ResiliencyPolicy) -> Self {
        let bulkhead = policy
            .bulkhead
            .as_ref()
            .map(|cfg| Bulkhead::new(cfg.max_concurrent_calls, cfg.max_wait_duration));
        let metrics = Arc::new(ResiliencyMetrics::new());
        metrics.set_circuit_closed();

        Self {
            policy,
            metrics,
            bulkhead,
            cb_state: Mutex::new(CircuitState::Closed),
            cb_failure_count: AtomicU32::new(0),
            cb_success_count: AtomicU32::new(0),
            cb_last_failure_time: Mutex::new(None),
        }
    }

    pub fn with_metrics(policy: ResiliencyPolicy, metrics: Arc<ResiliencyMetrics>) -> Self {
        let mut this = Self::new(policy);
        this.metrics = metrics;
        this
    }

    pub fn metrics(&self) -> Arc<ResiliencyMetrics> {
        self.metrics.clone()
    }

    pub async fn execute<T, E, Fut, F>(&self, f: F) -> Result<T, ResiliencyError>
    where
        E: std::error::Error + Send + Sync + 'static,
        Fut: Future<Output = Result<T, E>>,
        F: Fn() -> Fut,
    {
        self.check_circuit_breaker()?;

        let _permit = match &self.bulkhead {
            Some(bh) => match bh.acquire().await {
                Ok(permit) => Some(permit),
                Err(err) => {
                    self.metrics.record_bulkhead_rejection();
                    return Err(err);
                }
            },
            None => None,
        };

        let retry_config = self.policy.retry.clone();
        let max_attempts = retry_config.as_ref().map_or(1, |r| r.max_attempts.max(1));
        let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>> = None;

        for attempt in 0..max_attempts {
            let result = match self.policy.timeout {
                Some(timeout_dur) => match tokio::time::timeout(timeout_dur, f()).await {
                    Ok(r) => r.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
                    Err(_) => {
                        self.metrics.record_timeout();
                        return Err(ResiliencyError::Timeout { after: timeout_dur });
                    }
                },
                None => f()
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            };

            match result {
                Ok(value) => {
                    self.record_success();
                    return Ok(value);
                }
                Err(err) => {
                    self.record_failure();
                    let retryable = self.is_retryable_error(err.as_ref());
                    last_error = Some(err);

                    if !retryable {
                        break;
                    }

                    if let Err(cb_err) = self.check_circuit_breaker() {
                        self.metrics.record_circuit_open();
                        return Err(cb_err);
                    }

                    if attempt + 1 < max_attempts {
                        if let Some(ref retry_cfg) = retry_config {
                            self.metrics.record_retry_attempt();

                            let delay = self
                                .policy
                                .backoff
                                .as_ref()
                                .map(|b| b.compute_delay(attempt, retry_cfg.multiplier))
                                .unwrap_or_else(|| retry_cfg.compute_delay(attempt));
                            tokio::time::sleep(delay).await;
                        }
                    }
                }
            }
        }

        Err(ResiliencyError::MaxRetriesExceeded {
            attempts: max_attempts,
            last_error: last_error.unwrap_or_else(|| {
                Box::<dyn std::error::Error + Send + Sync>::from("unknown resiliency error")
            }),
        })
    }

    fn is_retryable_error(&self, err: &(dyn std::error::Error + Send + Sync)) -> bool {
        if self.policy.retryable_errors.is_empty() {
            return true;
        }

        let message = err.to_string();
        self.policy
            .retryable_errors
            .iter()
            .any(|pattern| message.contains(pattern))
    }

    fn check_circuit_breaker(&self) -> Result<(), ResiliencyError> {
        let cb_config = match &self.policy.circuit_breaker {
            Some(cfg) => cfg,
            None => return Ok(()),
        };

        let mut state = self.cb_state.lock().expect("circuit breaker lock poisoned");

        match *state {
            CircuitState::Closed => Ok(()),
            CircuitState::Open => {
                let last_failure = self
                    .cb_last_failure_time
                    .lock()
                    .expect("circuit breaker lock poisoned");
                if let Some(t) = *last_failure {
                    if t.elapsed() >= cb_config.timeout {
                        *state = CircuitState::HalfOpen;
                        self.cb_success_count.store(0, Ordering::SeqCst);
                        self.metrics.set_circuit_half_open();
                        Ok(())
                    } else {
                        let remaining = cb_config.timeout.saturating_sub(t.elapsed());
                        Err(ResiliencyError::CircuitOpen {
                            remaining_duration: remaining,
                        })
                    }
                } else {
                    Ok(())
                }
            }
            CircuitState::HalfOpen => Ok(()),
        }
    }

    fn record_success(&self) {
        if let Some(ref cb_config) = self.policy.circuit_breaker {
            let mut state = self.cb_state.lock().expect("circuit breaker lock poisoned");
            match *state {
                CircuitState::HalfOpen => {
                    let count = self.cb_success_count.fetch_add(1, Ordering::SeqCst) + 1;
                    if count >= cb_config.success_threshold {
                        *state = CircuitState::Closed;
                        self.cb_failure_count.store(0, Ordering::SeqCst);
                        self.metrics.set_circuit_closed();
                    }
                }
                CircuitState::Closed => {
                    self.cb_failure_count.store(0, Ordering::SeqCst);
                }
                CircuitState::Open => {}
            }
        }
    }

    fn record_failure(&self) {
        if let Some(ref cb_config) = self.policy.circuit_breaker {
            let count = self.cb_failure_count.fetch_add(1, Ordering::SeqCst) + 1;
            if count >= cb_config.failure_threshold {
                let mut state = self.cb_state.lock().expect("circuit breaker lock poisoned");
                *state = CircuitState::Open;
                let mut last = self
                    .cb_last_failure_time
                    .lock()
                    .expect("circuit breaker lock poisoned");
                *last = Some(Instant::now());
                self.metrics.set_circuit_open();
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::policy::{BulkheadConfig, CircuitBreakerConfig, ExponentialBackoff, RetryConfig};
    use std::sync::atomic::AtomicU32 as TestAtomicU32;

    #[derive(Debug)]
    struct TestError(String);
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::error::Error for TestError {}

    #[tokio::test]
    async fn test_policy_decorate() {
        let policy = ResiliencyPolicy::builder().build();
        let decorator = policy.decorate();
        let value = decorator
            .execute(|| async { Ok::<_, TestError>(42) })
            .await
            .unwrap();
        assert_eq!(value, 42);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_and_retryable_errors() {
        let policy = ResiliencyPolicy::builder()
            .retry(RetryConfig::new(3).with_jitter(false))
            .backoff(ExponentialBackoff::new(
                std::time::Duration::from_millis(1),
                std::time::Duration::from_millis(10),
            ))
            .retryable_errors(["retryable"])
            .build();
        let decorator = ResiliencyDecorator::new(policy);
        let counter = Arc::new(TestAtomicU32::new(0));
        let c = counter.clone();

        let result = decorator
            .execute(move || {
                let c = c.clone();
                async move {
                    let count = c.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(TestError("retryable error".into()))
                    } else {
                        Ok(99)
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 99);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
        assert!(decorator.metrics().retry_attempts() >= 2);
    }

    #[tokio::test]
    async fn test_non_retryable_error_stops_immediately() {
        let policy = ResiliencyPolicy::builder()
            .retry(RetryConfig::new(5).with_jitter(false))
            .retryable_errors(["network"])
            .build();
        let decorator = ResiliencyDecorator::new(policy);

        let result = decorator
            .execute(|| async { Err::<i32, _>(TestError("validation".into())) })
            .await;

        assert!(matches!(
            result,
            Err(ResiliencyError::MaxRetriesExceeded { attempts: 5, .. })
        ));
    }

    #[tokio::test]
    async fn test_timeout() {
        let policy = ResiliencyPolicy::builder()
            .timeout(std::time::Duration::from_millis(50))
            .build();
        let decorator = ResiliencyDecorator::new(policy);

        let result = decorator
            .execute(|| async {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                Ok::<_, TestError>(42)
            })
            .await;

        assert!(matches!(result, Err(ResiliencyError::Timeout { .. })));
        assert_eq!(decorator.metrics().timeout_events(), 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens() {
        let policy = ResiliencyPolicy::builder()
            .circuit_breaker(CircuitBreakerConfig {
                failure_threshold: 3,
                success_threshold: 1,
                timeout: std::time::Duration::from_secs(60),
            })
            .retry(RetryConfig::new(1).with_jitter(false))
            .build();
        let decorator = ResiliencyDecorator::new(policy);

        for _ in 0..3 {
            let _ = decorator
                .execute(|| async { Err::<i32, _>(TestError("fail".into())) })
                .await;
        }

        let result = decorator.execute(|| async { Ok::<_, TestError>(42) }).await;

        assert!(matches!(result, Err(ResiliencyError::CircuitOpen { .. })));
    }

    #[tokio::test]
    async fn test_bulkhead_limits_concurrency() {
        let policy = ResiliencyPolicy::builder()
            .bulkhead(BulkheadConfig {
                max_concurrent_calls: 1,
                max_wait_duration: std::time::Duration::from_millis(50),
            })
            .build();
        let decorator = Arc::new(ResiliencyDecorator::new(policy));

        let d1 = decorator.clone();
        let handle = tokio::spawn(async move {
            d1.execute(|| async {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                Ok::<_, TestError>(1)
            })
            .await
        });

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let result = decorator.execute(|| async { Ok::<_, TestError>(2) }).await;

        assert!(matches!(result, Err(ResiliencyError::BulkheadFull { .. })));
        assert_eq!(decorator.metrics().bulkhead_rejections(), 1);

        let _ = handle.await;
    }
}
