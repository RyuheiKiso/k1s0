use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(30),
        }
    }
}

struct CircuitBreakerInner {
    state: CircuitBreakerState,
    failure_count: u32,
    success_count: u32,
}

pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<Mutex<CircuitBreakerInner>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(CircuitBreakerInner {
                state: CircuitBreakerState::Closed,
                failure_count: 0,
                success_count: 0,
            })),
        }
    }

    pub async fn is_open(&self) -> bool {
        let mut inner = self.state.lock().await;
        if let CircuitBreakerState::Open { opened_at } = inner.state {
            if opened_at.elapsed() >= self.config.timeout {
                inner.state = CircuitBreakerState::HalfOpen;
                inner.success_count = 0;
                return false;
            }
            return true;
        }
        false
    }

    pub async fn record_success(&self) {
        let mut inner = self.state.lock().await;
        match inner.state {
            CircuitBreakerState::HalfOpen => {
                inner.success_count += 1;
                if inner.success_count >= self.config.success_threshold {
                    inner.state = CircuitBreakerState::Closed;
                    inner.failure_count = 0;
                    inner.success_count = 0;
                }
            }
            CircuitBreakerState::Closed => {
                inner.failure_count = 0;
            }
            _ => {}
        }
    }

    pub async fn record_failure(&self) {
        let mut inner = self.state.lock().await;
        inner.failure_count += 1;
        if inner.failure_count >= self.config.failure_threshold {
            inner.state = CircuitBreakerState::Open {
                opened_at: Instant::now(),
            };
            inner.failure_count = 0;
        }
    }

    pub async fn get_state(&self) -> CircuitBreakerState {
        self.state.lock().await.state.clone()
    }
}
