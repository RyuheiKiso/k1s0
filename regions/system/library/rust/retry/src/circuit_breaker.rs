pub use k1s0_circuit_breaker::CircuitBreakerConfig;
pub use k1s0_circuit_breaker::CircuitBreakerState;

/// Compatibility wrapper that delegates implementation to `k1s0-circuit-breaker`.
pub struct CircuitBreaker {
    inner: k1s0_circuit_breaker::CircuitBreaker,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            inner: k1s0_circuit_breaker::CircuitBreaker::new(config),
        }
    }

    pub async fn is_open(&self) -> bool {
        matches!(self.inner.state().await, CircuitBreakerState::Open)
    }

    pub async fn record_success(&self) {
        self.inner.record_success().await;
    }

    pub async fn record_failure(&self) {
        self.inner.record_failure().await;
    }

    pub async fn get_state(&self) -> CircuitBreakerState {
        self.inner.state().await
    }
}
