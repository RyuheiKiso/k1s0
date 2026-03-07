use std::time::Duration;

pub use k1s0_bulkhead::BulkheadConfig;
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

// --- Feature flag integration ---

#[cfg(feature = "hot-reload")]
mod featureflag_support {
    use super::*;
    use crate::error::ResiliencyError;
    use serde::Deserialize;

    /// JSON-serializable representation of a resiliency policy for feature flag storage.
    #[derive(Debug, Deserialize)]
    struct PolicyJson {
        retry: Option<RetryJson>,
        circuit_breaker: Option<CircuitBreakerJson>,
        bulkhead: Option<BulkheadJson>,
        #[serde(default)]
        timeout_ms: Option<u64>,
        backoff: Option<BackoffJson>,
        #[serde(default)]
        retryable_errors: Vec<String>,
    }

    #[derive(Debug, Deserialize)]
    struct RetryJson {
        max_attempts: u32,
        #[serde(default = "default_initial_delay_ms")]
        initial_delay_ms: u64,
        #[serde(default = "default_max_delay_ms")]
        max_delay_ms: u64,
        #[serde(default = "default_multiplier")]
        multiplier: f64,
        #[serde(default = "default_jitter")]
        jitter: bool,
    }

    fn default_initial_delay_ms() -> u64 { 100 }
    fn default_max_delay_ms() -> u64 { 30_000 }
    fn default_multiplier() -> f64 { 2.0 }
    fn default_jitter() -> bool { true }

    #[derive(Debug, Deserialize)]
    struct CircuitBreakerJson {
        failure_threshold: u32,
        #[serde(default = "default_success_threshold")]
        success_threshold: u32,
        #[serde(default = "default_timeout_ms")]
        timeout_ms: u64,
    }

    fn default_success_threshold() -> u32 { 3 }
    fn default_timeout_ms() -> u64 { 30_000 }

    #[derive(Debug, Deserialize)]
    struct BulkheadJson {
        max_concurrent_calls: usize,
        #[serde(default = "default_bulkhead_wait_ms")]
        max_wait_duration_ms: u64,
    }

    fn default_bulkhead_wait_ms() -> u64 { 500 }

    #[derive(Debug, Deserialize)]
    struct BackoffJson {
        initial_delay_ms: u64,
        max_delay_ms: u64,
    }

    impl From<PolicyJson> for ResiliencyPolicy {
        fn from(json: PolicyJson) -> Self {
            let mut builder = ResiliencyPolicy::builder();

            if let Some(r) = json.retry {
                builder = builder.retry(
                    RetryConfig::new(r.max_attempts)
                        .with_initial_delay(Duration::from_millis(r.initial_delay_ms))
                        .with_max_delay(Duration::from_millis(r.max_delay_ms))
                        .with_multiplier(r.multiplier)
                        .with_jitter(r.jitter),
                );
            }

            if let Some(cb) = json.circuit_breaker {
                builder = builder.circuit_breaker(CircuitBreakerConfig {
                    failure_threshold: cb.failure_threshold,
                    success_threshold: cb.success_threshold,
                    timeout: Duration::from_millis(cb.timeout_ms),
                });
            }

            if let Some(bh) = json.bulkhead {
                builder = builder.bulkhead(BulkheadConfig {
                    max_concurrent_calls: bh.max_concurrent_calls,
                    max_wait_duration: Duration::from_millis(bh.max_wait_duration_ms),
                });
            }

            if let Some(timeout_ms) = json.timeout_ms {
                builder = builder.timeout(Duration::from_millis(timeout_ms));
            }

            if let Some(bo) = json.backoff {
                builder = builder.backoff(ExponentialBackoff::new(
                    Duration::from_millis(bo.initial_delay_ms),
                    Duration::from_millis(bo.max_delay_ms),
                ));
            }

            if !json.retryable_errors.is_empty() {
                builder = builder.retryable_errors(json.retryable_errors);
            }

            builder.build()
        }
    }

    impl ResiliencyPolicy {
        /// Fetch a resiliency policy from a feature flag.
        ///
        /// The feature flag's first variant value is expected to be a JSON string
        /// representing the policy configuration. Returns an error if the flag
        /// cannot be fetched or the JSON is invalid.
        pub async fn from_featureflag(
            key: &str,
            client: &dyn k1s0_featureflag::FeatureFlagClient,
        ) -> Result<Self, ResiliencyError> {
            let flag = client.get_flag(key).await.map_err(|e| {
                ResiliencyError::Config {
                    message: format!("failed to fetch feature flag '{}': {}", key, e),
                }
            })?;

            let variant_value = flag
                .variants
                .first()
                .map(|v| v.value.as_str())
                .unwrap_or("{}");

            let policy_json: PolicyJson =
                serde_json::from_str(variant_value).map_err(|e| ResiliencyError::Config {
                    message: format!(
                        "failed to deserialize feature flag '{}' as ResiliencyPolicy: {}",
                        key, e
                    ),
                })?;

            Ok(policy_json.into())
        }
    }
}

#[cfg(all(test, feature = "hot-reload"))]
mod featureflag_tests {
    use super::*;
    use k1s0_featureflag::{FeatureFlag, FlagVariant, InMemoryFeatureFlagClient};
    use std::time::Duration;

    fn make_flag(key: &str, json_value: &str) -> FeatureFlag {
        FeatureFlag {
            id: "test-id".to_string(),
            flag_key: key.to_string(),
            description: "test flag".to_string(),
            enabled: true,
            variants: vec![FlagVariant {
                name: "default".to_string(),
                value: json_value.to_string(),
                weight: 100,
            }],
        }
    }

    #[tokio::test]
    async fn test_from_featureflag_full_policy() {
        let client = InMemoryFeatureFlagClient::new();
        let json = r#"{
            "retry": { "max_attempts": 5, "initial_delay_ms": 200, "max_delay_ms": 10000, "multiplier": 3.0, "jitter": false },
            "circuit_breaker": { "failure_threshold": 10, "success_threshold": 2, "timeout_ms": 60000 },
            "bulkhead": { "max_concurrent_calls": 15, "max_wait_duration_ms": 2000 },
            "timeout_ms": 8000,
            "backoff": { "initial_delay_ms": 50, "max_delay_ms": 5000 },
            "retryable_errors": ["network_error", "timeout"]
        }"#;
        client.set_flag(make_flag("svc-policy", json)).await;

        let policy = ResiliencyPolicy::from_featureflag("svc-policy", &client)
            .await
            .unwrap();

        let retry = policy.retry.unwrap();
        assert_eq!(retry.max_attempts, 5);
        assert_eq!(retry.multiplier, 3.0);
        assert!(!retry.jitter);

        let cb = policy.circuit_breaker.unwrap();
        assert_eq!(cb.failure_threshold, 10);
        assert_eq!(cb.success_threshold, 2);
        assert_eq!(cb.timeout, Duration::from_millis(60000));

        let bh = policy.bulkhead.unwrap();
        assert_eq!(bh.max_concurrent_calls, 15);
        assert_eq!(bh.max_wait_duration, Duration::from_millis(2000));

        assert_eq!(policy.timeout, Some(Duration::from_millis(8000)));

        let bo = policy.backoff.unwrap();
        assert_eq!(bo.initial_delay, Duration::from_millis(50));
        assert_eq!(bo.max_delay, Duration::from_millis(5000));

        assert_eq!(policy.retryable_errors, vec!["network_error", "timeout"]);
    }

    #[tokio::test]
    async fn test_from_featureflag_minimal_policy() {
        let client = InMemoryFeatureFlagClient::new();
        let json = r#"{ "retry": { "max_attempts": 2 } }"#;
        client.set_flag(make_flag("minimal", json)).await;

        let policy = ResiliencyPolicy::from_featureflag("minimal", &client)
            .await
            .unwrap();

        let retry = policy.retry.unwrap();
        assert_eq!(retry.max_attempts, 2);
        // defaults applied
        assert_eq!(retry.multiplier, 2.0);
        assert!(retry.jitter);

        assert!(policy.circuit_breaker.is_none());
        assert!(policy.bulkhead.is_none());
        assert!(policy.timeout.is_none());
        assert!(policy.backoff.is_none());
        assert!(policy.retryable_errors.is_empty());
    }

    #[tokio::test]
    async fn test_from_featureflag_empty_json_returns_default() {
        let client = InMemoryFeatureFlagClient::new();
        client.set_flag(make_flag("empty", "{}")).await;

        let policy = ResiliencyPolicy::from_featureflag("empty", &client)
            .await
            .unwrap();

        assert!(policy.retry.is_none());
        assert!(policy.circuit_breaker.is_none());
        assert!(policy.bulkhead.is_none());
        assert!(policy.timeout.is_none());
    }

    #[tokio::test]
    async fn test_from_featureflag_flag_not_found() {
        let client = InMemoryFeatureFlagClient::new();

        let result = ResiliencyPolicy::from_featureflag("nonexistent", &client).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, crate::error::ResiliencyError::Config { .. }),
            "expected Config error, got: {:?}",
            err
        );
    }

    #[tokio::test]
    async fn test_from_featureflag_invalid_json() {
        let client = InMemoryFeatureFlagClient::new();
        client
            .set_flag(make_flag("bad-json", "not valid json"))
            .await;

        let result = ResiliencyPolicy::from_featureflag("bad-json", &client).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, crate::error::ResiliencyError::Config { .. }),
            "expected Config error, got: {:?}",
            err
        );
    }

    #[tokio::test]
    async fn test_from_featureflag_empty_variants_returns_default() {
        let client = InMemoryFeatureFlagClient::new();
        let flag = FeatureFlag {
            id: "test-id".to_string(),
            flag_key: "no-variants".to_string(),
            description: "flag with no variants".to_string(),
            enabled: true,
            variants: vec![],
        };
        client.set_flag(flag).await;

        let policy = ResiliencyPolicy::from_featureflag("no-variants", &client)
            .await
            .unwrap();

        // Empty variants -> falls back to "{}" -> default policy
        assert!(policy.retry.is_none());
        assert!(policy.circuit_breaker.is_none());
    }
}
