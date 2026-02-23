use std::future::Future;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::bulkhead::Bulkhead;
use crate::error::ResiliencyError;
use crate::policy::ResiliencyPolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct ResiliencyDecorator {
    policy: ResiliencyPolicy,
    bulkhead: Option<Bulkhead>,
    cb_state: Mutex<CircuitState>,
    cb_failure_count: AtomicU32,
    cb_success_count: AtomicU32,
    cb_last_failure_time: Mutex<Option<Instant>>,
}

impl ResiliencyDecorator {
    pub fn new(policy: ResiliencyPolicy) -> Self {
        let bulkhead = policy.bulkhead.as_ref().map(|cfg| {
            Bulkhead::new(cfg.max_concurrent_calls, cfg.max_wait_duration)
        });
        Self {
            policy,
            bulkhead,
            cb_state: Mutex::new(CircuitState::Closed),
            cb_failure_count: AtomicU32::new(0),
            cb_success_count: AtomicU32::new(0),
            cb_last_failure_time: Mutex::new(None),
        }
    }

    pub async fn execute<T, E, Fut, F>(&self, f: F) -> Result<T, ResiliencyError>
    where
        E: std::error::Error + Send + Sync + 'static,
        Fut: Future<Output = Result<T, E>>,
        F: Fn() -> Fut,
    {
        // Check circuit breaker before starting
        self.check_circuit_breaker()?;

        // Acquire bulkhead permit if configured
        let _permit = match &self.bulkhead {
            Some(bh) => Some(bh.acquire().await?),
            None => None,
        };

        // Execute with retry logic
        let retry_config = self.policy.retry.clone();
        let max_attempts = retry_config.as_ref().map_or(1, |r| r.max_attempts);

        let mut last_error: Option<Box<dyn std::error::Error + Send + Sync>> = None;

        for attempt in 0..max_attempts {
            // Apply timeout if configured
            let result = match self.policy.timeout {
                Some(timeout_dur) => {
                    match tokio::time::timeout(timeout_dur, f()).await {
                        Ok(r) => r.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
                        Err(_) => return Err(ResiliencyError::Timeout { after: timeout_dur }),
                    }
                }
                None => f().await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
            };

            match result {
                Ok(value) => {
                    self.record_success();
                    return Ok(value);
                }
                Err(e) => {
                    self.record_failure();
                    last_error = Some(e);

                    // Check if circuit breaker tripped
                    if let Err(cb_err) = self.check_circuit_breaker() {
                        return Err(cb_err);
                    }

                    // Apply backoff delay before next retry
                    if attempt + 1 < max_attempts {
                        if let Some(ref retry_cfg) = retry_config {
                            let delay = calculate_backoff(attempt, retry_cfg.base_delay, retry_cfg.max_delay);
                            tokio::time::sleep(delay).await;
                        }
                    }
                }
            }
        }

        Err(ResiliencyError::MaxRetriesExceeded {
            attempts: max_attempts,
            last_error: last_error.unwrap(),
        })
    }

    fn check_circuit_breaker(&self) -> Result<(), ResiliencyError> {
        let cb_config = match &self.policy.circuit_breaker {
            Some(cfg) => cfg,
            None => return Ok(()),
        };

        let mut state = self.cb_state.lock().unwrap();

        match *state {
            CircuitState::Closed => Ok(()),
            CircuitState::Open => {
                let last_failure = self.cb_last_failure_time.lock().unwrap();
                if let Some(t) = *last_failure {
                    if t.elapsed() >= cb_config.recovery_timeout {
                        *state = CircuitState::HalfOpen;
                        self.cb_success_count.store(0, Ordering::SeqCst);
                        Ok(())
                    } else {
                        let remaining = cb_config.recovery_timeout - t.elapsed();
                        Err(ResiliencyError::CircuitBreakerOpen {
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
            let mut state = self.cb_state.lock().unwrap();
            match *state {
                CircuitState::HalfOpen => {
                    let count = self.cb_success_count.fetch_add(1, Ordering::SeqCst) + 1;
                    if count >= cb_config.half_open_max_calls {
                        *state = CircuitState::Closed;
                        self.cb_failure_count.store(0, Ordering::SeqCst);
                    }
                }
                CircuitState::Closed => {
                    self.cb_failure_count.store(0, Ordering::SeqCst);
                }
                _ => {}
            }
        }
    }

    fn record_failure(&self) {
        if let Some(ref cb_config) = self.policy.circuit_breaker {
            let count = self.cb_failure_count.fetch_add(1, Ordering::SeqCst) + 1;
            if count >= cb_config.failure_threshold {
                let mut state = self.cb_state.lock().unwrap();
                *state = CircuitState::Open;
                let mut last = self.cb_last_failure_time.lock().unwrap();
                *last = Some(Instant::now());
            }
        }
    }
}

fn calculate_backoff(attempt: u32, base_delay: Duration, max_delay: Duration) -> Duration {
    let delay = base_delay.saturating_mul(2u32.saturating_pow(attempt));
    std::cmp::min(delay, max_delay)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::{BulkheadConfig, CircuitBreakerConfig, RetryConfig};
    use std::sync::atomic::AtomicU32 as TestAtomicU32;
    use std::sync::Arc;

    #[derive(Debug)]
    struct TestError(String);
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::error::Error for TestError {}

    #[tokio::test]
    async fn test_successful_execution() {
        let policy = ResiliencyPolicy::builder().build();
        let decorator = ResiliencyDecorator::new(policy);

        let result = decorator
            .execute(|| async { Ok::<_, TestError>(42) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_on_failure() {
        let policy = ResiliencyPolicy::builder()
            .retry(RetryConfig {
                max_attempts: 3,
                base_delay: Duration::from_millis(10),
                max_delay: Duration::from_millis(100),
                jitter: false,
            })
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
                        Err(TestError("fail".into()))
                    } else {
                        Ok(99)
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 99);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_max_retries_exceeded() {
        let policy = ResiliencyPolicy::builder()
            .retry(RetryConfig {
                max_attempts: 2,
                base_delay: Duration::from_millis(1),
                max_delay: Duration::from_millis(10),
                jitter: false,
            })
            .build();
        let decorator = ResiliencyDecorator::new(policy);

        let result = decorator
            .execute(|| async { Err::<i32, _>(TestError("always fail".into())) })
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ResiliencyError::MaxRetriesExceeded { attempts, .. } => {
                assert_eq!(attempts, 2);
            }
            _ => panic!("expected MaxRetriesExceeded"),
        }
    }

    #[tokio::test]
    async fn test_timeout() {
        let policy = ResiliencyPolicy::builder()
            .timeout(Duration::from_millis(50))
            .build();
        let decorator = ResiliencyDecorator::new(policy);

        let result = decorator
            .execute(|| async {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok::<_, TestError>(42)
            })
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ResiliencyError::Timeout { .. } => {}
            _ => panic!("expected Timeout"),
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens() {
        let policy = ResiliencyPolicy::builder()
            .circuit_breaker(CircuitBreakerConfig {
                failure_threshold: 3,
                recovery_timeout: Duration::from_secs(60),
                half_open_max_calls: 1,
            })
            .build();
        let decorator = ResiliencyDecorator::new(policy);

        // Trip the circuit breaker
        for _ in 0..3 {
            let _ = decorator
                .execute(|| async { Err::<i32, _>(TestError("fail".into())) })
                .await;
        }

        // Next call should fail with CircuitBreakerOpen
        let result = decorator
            .execute(|| async { Ok::<_, TestError>(42) })
            .await;

        match result.unwrap_err() {
            ResiliencyError::CircuitBreakerOpen { .. } => {}
            other => panic!("expected CircuitBreakerOpen, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_bulkhead_limits_concurrency() {
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

        // Give time for first task to acquire permit
        tokio::time::sleep(Duration::from_millis(10)).await;

        let result = decorator
            .execute(|| async { Ok::<_, TestError>(2) })
            .await;

        match result.unwrap_err() {
            ResiliencyError::BulkheadFull { max_concurrent } => {
                assert_eq!(max_concurrent, 1);
            }
            other => panic!("expected BulkheadFull, got {:?}", other),
        }

        let _ = handle.await;
    }

    #[test]
    fn test_backoff_calculation() {
        let base = Duration::from_millis(100);
        let max = Duration::from_secs(5);

        assert_eq!(calculate_backoff(0, base, max), Duration::from_millis(100));
        assert_eq!(calculate_backoff(1, base, max), Duration::from_millis(200));
        assert_eq!(calculate_backoff(2, base, max), Duration::from_millis(400));
        assert_eq!(calculate_backoff(10, base, max), max);
    }
}
