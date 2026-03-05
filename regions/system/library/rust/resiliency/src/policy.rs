use std::time::Duration;

pub use k1s0_circuit_breaker::CircuitBreakerConfig;
pub use k1s0_retry::RetryConfig;

use crate::decorator::ResiliencyDecorator;

#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    pub initial_delay: Duration,
    pub max_delay: Duration,
}

impl ExponentialBackoff {
    pub fn new(initial_delay: Duration, max_delay: Duration) -> Self {
        Self {
            initial_delay,
            max_delay,
        }
    }

    pub fn compute_delay(&self, attempt: u32, multiplier: f64) -> Duration {
        let base = self.initial_delay.as_millis() as f64 * multiplier.powi(attempt as i32);
        let capped = base.min(self.max_delay.as_millis() as f64);
        Duration::from_millis(capped as u64)
    }
}

#[derive(Debug, Clone)]
pub struct BulkheadConfig {
    pub max_concurrent_calls: usize,
    pub max_wait_duration: Duration,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self {
            max_concurrent_calls: 20,
            max_wait_duration: Duration::from_millis(500),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResiliencyPolicy {
    pub retry: Option<RetryConfig>,
    pub circuit_breaker: Option<CircuitBreakerConfig>,
    pub bulkhead: Option<BulkheadConfig>,
    pub timeout: Option<Duration>,
    pub backoff: Option<ExponentialBackoff>,
    pub retryable_errors: Vec<String>,
}

impl ResiliencyPolicy {
    pub fn builder() -> ResiliencyPolicyBuilder {
        ResiliencyPolicyBuilder::default()
    }

    pub fn decorate(self) -> ResiliencyDecorator {
        ResiliencyDecorator::new(self)
    }
}

#[derive(Debug, Default)]
pub struct ResiliencyPolicyBuilder {
    retry: Option<RetryConfig>,
    circuit_breaker: Option<CircuitBreakerConfig>,
    bulkhead: Option<BulkheadConfig>,
    timeout: Option<Duration>,
    backoff: Option<ExponentialBackoff>,
    retryable_errors: Vec<String>,
}

impl ResiliencyPolicyBuilder {
    pub fn retry(mut self, config: RetryConfig) -> Self {
        self.retry = Some(config);
        self
    }

    pub fn circuit_breaker(mut self, config: CircuitBreakerConfig) -> Self {
        self.circuit_breaker = Some(config);
        self
    }

    pub fn bulkhead(mut self, config: BulkheadConfig) -> Self {
        self.bulkhead = Some(config);
        self
    }

    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    pub fn backoff(mut self, backoff: ExponentialBackoff) -> Self {
        self.backoff = Some(backoff);
        self
    }

    pub fn retryable_errors<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.retryable_errors = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn build(self) -> ResiliencyPolicy {
        ResiliencyPolicy {
            retry: self.retry,
            circuit_breaker: self.circuit_breaker,
            bulkhead: self.bulkhead,
            timeout: self.timeout,
            backoff: self.backoff,
            retryable_errors: self.retryable_errors,
        }
    }
}
