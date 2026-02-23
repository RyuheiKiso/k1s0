use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            jitter: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            half_open_max_calls: 2,
        }
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
}

impl ResiliencyPolicy {
    pub fn builder() -> ResiliencyPolicyBuilder {
        ResiliencyPolicyBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct ResiliencyPolicyBuilder {
    retry: Option<RetryConfig>,
    circuit_breaker: Option<CircuitBreakerConfig>,
    bulkhead: Option<BulkheadConfig>,
    timeout: Option<Duration>,
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

    pub fn build(self) -> ResiliencyPolicy {
        ResiliencyPolicy {
            retry: self.retry,
            circuit_breaker: self.circuit_breaker,
            bulkhead: self.bulkhead,
            timeout: self.timeout,
        }
    }
}
