use std::future::Future;
use std::time::Instant;

use tokio::sync::Mutex;

use crate::config::CircuitBreakerConfig;
use crate::error::CircuitBreakerError;
use crate::metrics::{CircuitBreakerMetrics, CircuitBreakerMetricsRecorder};

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
    metrics: CircuitBreakerMetricsRecorder,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        let metrics = CircuitBreakerMetricsRecorder::new();
        metrics.set_state(CircuitBreakerState::Closed);

        Self {
            config,
            inner: Mutex::new(Inner {
                state: CircuitBreakerState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: None,
            }),
            metrics,
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
        self.metrics.record_success();
        let mut inner = self.inner.lock().await;
        inner.success_count += 1;

        match inner.state {
            // Closed 状態では成功時に失敗カウントをリセットする（Go/TS/Dart と統一）
            CircuitBreakerState::Closed => {
                inner.failure_count = 0;
            }
            // HalfOpen 状態では成功閾値に達したら Closed へ遷移する
            CircuitBreakerState::HalfOpen => {
                if inner.success_count >= self.config.success_threshold {
                    inner.state = CircuitBreakerState::Closed;
                    inner.failure_count = 0;
                    inner.success_count = 0;
                    inner.last_failure_time = None;
                    self.metrics.set_state(CircuitBreakerState::Closed);
                }
            }
            _ => {}
        }
    }

    pub async fn record_failure(&self) {
        self.metrics.record_failure();
        let mut inner = self.inner.lock().await;
        inner.failure_count += 1;
        inner.last_failure_time = Some(Instant::now());

        // HalfOpen状態または失敗回数が閾値を超えた場合はOpen状態に遷移する
        if inner.state == CircuitBreakerState::HalfOpen
            || inner.failure_count >= self.config.failure_threshold
        {
            inner.state = CircuitBreakerState::Open;
            inner.success_count = 0;
            self.metrics.set_state(CircuitBreakerState::Open);
        }
    }

    pub async fn metrics(&self) -> CircuitBreakerMetrics {
        self.metrics.snapshot()
    }

    fn maybe_transition_to_half_open(&self, inner: &mut Inner) {
        if inner.state == CircuitBreakerState::Open {
            if let Some(last_failure) = inner.last_failure_time {
                if last_failure.elapsed() >= self.config.timeout {
                    inner.state = CircuitBreakerState::HalfOpen;
                    inner.success_count = 0;
                    self.metrics.set_state(CircuitBreakerState::HalfOpen);
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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

    // サーキットブレーカーの初期状態が Closed であることを確認する。
    #[tokio::test]
    async fn test_starts_closed() {
        let cb = CircuitBreaker::new(test_config());
        assert_eq!(cb.state().await, CircuitBreakerState::Closed);
    }

    // 失敗回数が閾値に達した後に Open へ遷移することを確認する。
    #[tokio::test]
    async fn test_opens_after_failure_threshold() {
        let cb = CircuitBreaker::new(test_config());

        for _ in 0..3 {
            cb.record_failure().await;
        }

        assert_eq!(cb.state().await, CircuitBreakerState::Open);
    }

    // タイムアウト経過後に Open から HalfOpen へ遷移することを確認する。
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

    // HalfOpen 状態で成功回数が閾値に達した後に Closed へ遷移することを確認する。
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

    // call が成功した場合に値が正しく返されることを確認する。
    #[tokio::test]
    async fn test_call_success() {
        let cb = CircuitBreaker::new(test_config());

        let result: Result<i32, CircuitBreakerError<String>> = cb.call(|| async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    // Open 状態のとき call がリジェクトされることを確認する。
    #[tokio::test]
    async fn test_call_rejects_when_open() {
        let cb = CircuitBreaker::new(test_config());

        for _ in 0..3 {
            cb.record_failure().await;
        }

        let result: Result<i32, CircuitBreakerError<String>> = cb.call(|| async { Ok(42) }).await;
        assert!(matches!(result, Err(CircuitBreakerError::Open)));
    }

    // HalfOpen 状態で失敗が発生した場合に Open へ再遷移することを確認する。
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

    // メトリクスが成功数・失敗数・状態を正しく記録することを確認する。
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

    // Closed 状態で成功を記録すると失敗カウントがリセットされ、
    // その後の失敗で Open にならないことを確認する（Go/TS/Dart と統一）。
    #[tokio::test]
    async fn test_success_resets_failure_count_in_closed() {
        let cb = CircuitBreaker::new(test_config());

        // 閾値(3)未満の失敗を記録した後、成功で失敗カウントをリセットする
        cb.record_failure().await;
        cb.record_failure().await;
        cb.record_success().await;

        // リセット後の1回の失敗では Open にならないことを確認する
        cb.record_failure().await;
        assert_eq!(cb.state().await, CircuitBreakerState::Closed);
    }
}
