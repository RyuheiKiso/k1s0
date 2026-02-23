use std::future::Future;
use std::time::Instant;

use tokio::sync::Mutex;

use crate::config::CircuitBreakerConfig;
use crate::error::CircuitBreakerError;
use crate::metrics::CircuitBreakerMetrics;

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

struct Inner {
    state: CircuitBreakerState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
}

pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    inner: Mutex<Inner>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            inner: Mutex::new(Inner {
                state: CircuitBreakerState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: None,
            }),
        }
    }

    pub async fn state(&self) -> CircuitBreakerState {
        let mut inner = self.inner.lock().await;
        self.maybe_transition_to_half_open(&mut inner);
        inner.state.clone()
    }

    pub async fn call<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        {
            let mut inner = self.inner.lock().await;
            self.maybe_transition_to_half_open(&mut inner);

            if inner.state == CircuitBreakerState::Open {
                return Err(CircuitBreakerError::Open);
            }
        }

        match f().await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(e) => {
                self.record_failure().await;
                Err(CircuitBreakerError::Inner(e))
            }
        }
    }

    pub async fn record_success(&self) {
        let mut inner = self.inner.lock().await;
        inner.success_count += 1;

        if inner.state == CircuitBreakerState::HalfOpen
            && inner.success_count >= self.config.success_threshold
        {
            inner.state = CircuitBreakerState::Closed;
            inner.failure_count = 0;
            inner.success_count = 0;
            inner.last_failure_time = None;
        }
    }

    pub async fn record_failure(&self) {
        let mut inner = self.inner.lock().await;
        inner.failure_count += 1;
        inner.last_failure_time = Some(Instant::now());

        if inner.state == CircuitBreakerState::HalfOpen {
            inner.state = CircuitBreakerState::Open;
            inner.success_count = 0;
        } else if inner.failure_count >= self.config.failure_threshold {
            inner.state = CircuitBreakerState::Open;
            inner.success_count = 0;
        }
    }

    pub async fn metrics(&self) -> CircuitBreakerMetrics {
        let inner = self.inner.lock().await;
        CircuitBreakerMetrics {
            failure_count: inner.failure_count,
            success_count: inner.success_count,
            state: format!("{:?}", inner.state),
        }
    }

    fn maybe_transition_to_half_open(&self, inner: &mut Inner) {
        if inner.state == CircuitBreakerState::Open {
            if let Some(last_failure) = inner.last_failure_time {
                if last_failure.elapsed() >= self.config.timeout {
                    inner.state = CircuitBreakerState::HalfOpen;
                    inner.success_count = 0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn test_config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_millis(100),
        }
    }

    #[tokio::test]
    async fn test_starts_closed() {
        let cb = CircuitBreaker::new(test_config());
        assert_eq!(cb.state().await, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_opens_after_failure_threshold() {
        let cb = CircuitBreaker::new(test_config());

        for _ in 0..3 {
            cb.record_failure().await;
        }

        assert_eq!(cb.state().await, CircuitBreakerState::Open);
    }

    #[tokio::test]
    async fn test_half_open_after_timeout() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            timeout: Duration::from_millis(50),
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure().await;
        assert_eq!(cb.state().await, CircuitBreakerState::Open);

        tokio::time::sleep(Duration::from_millis(60)).await;
        assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);
    }

    #[tokio::test]
    async fn test_closes_after_success_threshold_in_half_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            timeout: Duration::from_millis(50),
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure().await;
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Should be HalfOpen now
        assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);

        cb.record_success().await;
        cb.record_success().await;

        assert_eq!(cb.state().await, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_call_success() {
        let cb = CircuitBreaker::new(test_config());

        let result: Result<i32, CircuitBreakerError<String>> =
            cb.call(|| async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_call_rejects_when_open() {
        let cb = CircuitBreaker::new(test_config());

        for _ in 0..3 {
            cb.record_failure().await;
        }

        let result: Result<i32, CircuitBreakerError<String>> =
            cb.call(|| async { Ok(42) }).await;
        assert!(matches!(result, Err(CircuitBreakerError::Open)));
    }

    #[tokio::test]
    async fn test_failure_in_half_open_reopens() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            timeout: Duration::from_millis(50),
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure().await;
        tokio::time::sleep(Duration::from_millis(60)).await;
        assert_eq!(cb.state().await, CircuitBreakerState::HalfOpen);

        cb.record_failure().await;
        assert_eq!(cb.state().await, CircuitBreakerState::Open);
    }

    #[tokio::test]
    async fn test_metrics() {
        let cb = CircuitBreaker::new(test_config());

        cb.record_success().await;
        cb.record_failure().await;

        let m = cb.metrics().await;
        assert_eq!(m.success_count, 1);
        assert_eq!(m.failure_count, 1);
        assert_eq!(m.state, "Closed");
    }
}
