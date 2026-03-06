use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::config::QuotaClientConfig;
use crate::error::QuotaClientError;
use crate::model::{QuotaPolicy, QuotaStatus, QuotaUsage};

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait QuotaClient: Send + Sync {
    async fn check(&self, quota_id: &str, amount: u64) -> Result<QuotaStatus, QuotaClientError>;
    async fn increment(&self, quota_id: &str, amount: u64) -> Result<QuotaUsage, QuotaClientError>;
    async fn get_usage(&self, quota_id: &str) -> Result<QuotaUsage, QuotaClientError>;
    async fn get_policy(&self, quota_id: &str) -> Result<QuotaPolicy, QuotaClientError>;
}

pub struct HttpQuotaClient {
    client: reqwest::Client,
    base_url: String,
}

impl HttpQuotaClient {
    pub fn new(config: QuotaClientConfig) -> Result<Self, QuotaClientError> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| QuotaClientError::ConnectionError(e.to_string()))?;
        Ok(Self {
            client,
            base_url: config.server_url,
        })
    }
}

#[async_trait]
impl QuotaClient for HttpQuotaClient {
    async fn check(&self, quota_id: &str, amount: u64) -> Result<QuotaStatus, QuotaClientError> {
        let url = format!("{}/api/v1/quotas/{}/check", self.base_url, quota_id);
        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "amount": amount }))
            .send()
            .await
            .map_err(|e| QuotaClientError::ConnectionError(e.to_string()))?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(QuotaClientError::NotFound(quota_id.to_string()));
        }
        resp.json::<QuotaStatus>()
            .await
            .map_err(|e| QuotaClientError::InvalidResponse(e.to_string()))
    }

    async fn increment(&self, quota_id: &str, amount: u64) -> Result<QuotaUsage, QuotaClientError> {
        let url = format!("{}/api/v1/quotas/{}/increment", self.base_url, quota_id);
        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "amount": amount }))
            .send()
            .await
            .map_err(|e| QuotaClientError::ConnectionError(e.to_string()))?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(QuotaClientError::NotFound(quota_id.to_string()));
        }
        resp.json::<QuotaUsage>()
            .await
            .map_err(|e| QuotaClientError::InvalidResponse(e.to_string()))
    }

    async fn get_usage(&self, quota_id: &str) -> Result<QuotaUsage, QuotaClientError> {
        let url = format!("{}/api/v1/quotas/{}/usage", self.base_url, quota_id);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| QuotaClientError::ConnectionError(e.to_string()))?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(QuotaClientError::NotFound(quota_id.to_string()));
        }
        resp.json::<QuotaUsage>()
            .await
            .map_err(|e| QuotaClientError::InvalidResponse(e.to_string()))
    }

    async fn get_policy(&self, quota_id: &str) -> Result<QuotaPolicy, QuotaClientError> {
        let url = format!("{}/api/v1/quotas/{}/policy", self.base_url, quota_id);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| QuotaClientError::ConnectionError(e.to_string()))?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(QuotaClientError::NotFound(quota_id.to_string()));
        }
        resp.json::<QuotaPolicy>()
            .await
            .map_err(|e| QuotaClientError::InvalidResponse(e.to_string()))
    }
}

struct CacheEntry<T> {
    value: T,
    inserted_at: Instant,
}

pub struct CachedQuotaClient<C: QuotaClient> {
    inner: C,
    policy_ttl: Duration,
    policy_cache: Mutex<HashMap<String, CacheEntry<QuotaPolicy>>>,
}

impl<C: QuotaClient> CachedQuotaClient<C> {
    pub fn new(inner: C, policy_ttl: Duration) -> Self {
        Self {
            inner,
            policy_ttl,
            policy_cache: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl<C: QuotaClient> QuotaClient for CachedQuotaClient<C> {
    async fn check(&self, quota_id: &str, amount: u64) -> Result<QuotaStatus, QuotaClientError> {
        self.inner.check(quota_id, amount).await
    }

    async fn increment(&self, quota_id: &str, amount: u64) -> Result<QuotaUsage, QuotaClientError> {
        self.inner.increment(quota_id, amount).await
    }

    async fn get_usage(&self, quota_id: &str) -> Result<QuotaUsage, QuotaClientError> {
        self.inner.get_usage(quota_id).await
    }

    async fn get_policy(&self, quota_id: &str) -> Result<QuotaPolicy, QuotaClientError> {
        {
            let cache = self.policy_cache.lock().unwrap();
            if let Some(entry) = cache.get(quota_id) {
                if entry.inserted_at.elapsed() < self.policy_ttl {
                    return Ok(entry.value.clone());
                }
            }
        }
        let policy = self.inner.get_policy(quota_id).await?;
        {
            let mut cache = self.policy_cache.lock().unwrap();
            cache.insert(
                quota_id.to_string(),
                CacheEntry {
                    value: policy.clone(),
                    inserted_at: Instant::now(),
                },
            );
        }
        Ok(policy)
    }
}

#[cfg(any(test, feature = "test-utils"))]
#[derive(Default)]
struct InMemoryState {
    usages: HashMap<String, QuotaUsage>,
    policies: HashMap<String, QuotaPolicy>,
}

#[cfg(any(test, feature = "test-utils"))]
pub struct InMemoryQuotaClient {
    state: Mutex<InMemoryState>,
}

#[cfg(any(test, feature = "test-utils"))]
impl InMemoryQuotaClient {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(InMemoryState::default()),
        }
    }

    pub fn set_policy(&self, quota_id: impl Into<String>, policy: QuotaPolicy) {
        let mut state = self.state.lock().unwrap();
        state.policies.insert(quota_id.into(), policy);
    }

    fn default_policy(quota_id: &str) -> QuotaPolicy {
        QuotaPolicy {
            quota_id: quota_id.to_string(),
            limit: 1000,
            period: crate::model::QuotaPeriod::Daily,
            reset_strategy: "fixed".to_string(),
        }
    }

    fn get_policy_internal(state: &InMemoryState, quota_id: &str) -> QuotaPolicy {
        state
            .policies
            .get(quota_id)
            .cloned()
            .unwrap_or_else(|| Self::default_policy(quota_id))
    }

    fn get_or_create_usage(state: &mut InMemoryState, quota_id: &str) -> QuotaUsage {
        if let Some(usage) = state.usages.get(quota_id) {
            return usage.clone();
        }
        let policy = Self::get_policy_internal(state, quota_id);
        let usage = QuotaUsage {
            quota_id: quota_id.to_string(),
            used: 0,
            limit: policy.limit,
            period: policy.period,
            reset_at: chrono::Utc::now() + chrono::Duration::days(1),
        };
        state.usages.insert(quota_id.to_string(), usage.clone());
        usage
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl Default for InMemoryQuotaClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(any(test, feature = "test-utils"))]
#[async_trait]
impl QuotaClient for InMemoryQuotaClient {
    async fn check(&self, quota_id: &str, amount: u64) -> Result<QuotaStatus, QuotaClientError> {
        let mut state = self.state.lock().unwrap();
        let usage = Self::get_or_create_usage(&mut state, quota_id);
        let remaining = usage.limit.saturating_sub(usage.used);
        Ok(QuotaStatus {
            allowed: amount <= remaining,
            remaining,
            limit: usage.limit,
            reset_at: usage.reset_at,
        })
    }

    async fn increment(&self, quota_id: &str, amount: u64) -> Result<QuotaUsage, QuotaClientError> {
        let mut state = self.state.lock().unwrap();
        let mut usage = Self::get_or_create_usage(&mut state, quota_id);
        usage.used = usage.used.saturating_add(amount);
        state.usages.insert(quota_id.to_string(), usage.clone());
        Ok(usage)
    }

    async fn get_usage(&self, quota_id: &str) -> Result<QuotaUsage, QuotaClientError> {
        let mut state = self.state.lock().unwrap();
        Ok(Self::get_or_create_usage(&mut state, quota_id))
    }

    async fn get_policy(&self, quota_id: &str) -> Result<QuotaPolicy, QuotaClientError> {
        let state = self.state.lock().unwrap();
        Ok(Self::get_policy_internal(&state, quota_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    use crate::model::QuotaPeriod;

    struct StubQuotaClient {
        check_result: QuotaStatus,
        usage_result: QuotaUsage,
        policy_result: QuotaPolicy,
        call_count: Mutex<u32>,
    }

    impl StubQuotaClient {
        fn new() -> Self {
            let now = Utc::now();
            Self {
                check_result: QuotaStatus {
                    allowed: true,
                    remaining: 900,
                    limit: 1000,
                    reset_at: now,
                },
                usage_result: QuotaUsage {
                    quota_id: "test-quota".to_string(),
                    used: 100,
                    limit: 1000,
                    period: QuotaPeriod::Daily,
                    reset_at: now,
                },
                policy_result: QuotaPolicy {
                    quota_id: "test-quota".to_string(),
                    limit: 1000,
                    period: QuotaPeriod::Daily,
                    reset_strategy: "fixed".to_string(),
                },
                call_count: Mutex::new(0),
            }
        }
    }

    #[async_trait]
    impl QuotaClient for StubQuotaClient {
        async fn check(
            &self,
            _quota_id: &str,
            _amount: u64,
        ) -> Result<QuotaStatus, QuotaClientError> {
            Ok(self.check_result.clone())
        }

        async fn increment(
            &self,
            _quota_id: &str,
            _amount: u64,
        ) -> Result<QuotaUsage, QuotaClientError> {
            Ok(self.usage_result.clone())
        }

        async fn get_usage(&self, _quota_id: &str) -> Result<QuotaUsage, QuotaClientError> {
            Ok(self.usage_result.clone())
        }

        async fn get_policy(&self, _quota_id: &str) -> Result<QuotaPolicy, QuotaClientError> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;
            Ok(self.policy_result.clone())
        }
    }

    #[tokio::test]
    async fn test_check_allowed() {
        let stub = StubQuotaClient::new();
        let status = stub.check("test-quota", 100).await.unwrap();
        assert!(status.allowed);
        assert_eq!(status.remaining, 900);
        assert_eq!(status.limit, 1000);
    }

    #[tokio::test]
    async fn test_increment() {
        let stub = StubQuotaClient::new();
        let usage = stub.increment("test-quota", 100).await.unwrap();
        assert_eq!(usage.quota_id, "test-quota");
        assert_eq!(usage.used, 100);
        assert_eq!(usage.limit, 1000);
    }

    #[tokio::test]
    async fn test_get_usage() {
        let stub = StubQuotaClient::new();
        let usage = stub.get_usage("test-quota").await.unwrap();
        assert_eq!(usage.period, QuotaPeriod::Daily);
    }

    #[tokio::test]
    async fn test_get_policy() {
        let stub = StubQuotaClient::new();
        let policy = stub.get_policy("test-quota").await.unwrap();
        assert_eq!(policy.limit, 1000);
        assert_eq!(policy.reset_strategy, "fixed");
    }

    #[tokio::test]
    async fn test_cached_client_caches_policy() {
        let stub = StubQuotaClient::new();
        let cached = CachedQuotaClient::new(stub, Duration::from_secs(60));

        let p1 = cached.get_policy("test-quota").await.unwrap();
        let p2 = cached.get_policy("test-quota").await.unwrap();
        assert_eq!(p1, p2);

        let count = *cached.inner.call_count.lock().unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_cached_client_delegates_check() {
        let stub = StubQuotaClient::new();
        let cached = CachedQuotaClient::new(stub, Duration::from_secs(60));
        let status = cached.check("test-quota", 50).await.unwrap();
        assert!(status.allowed);
    }

    #[tokio::test]
    async fn test_cached_client_delegates_increment() {
        let stub = StubQuotaClient::new();
        let cached = CachedQuotaClient::new(stub, Duration::from_secs(60));
        let usage = cached.increment("test-quota", 50).await.unwrap();
        assert_eq!(usage.used, 100);
    }

    #[tokio::test]
    async fn test_cached_client_delegates_get_usage() {
        let stub = StubQuotaClient::new();
        let cached = CachedQuotaClient::new(stub, Duration::from_secs(60));
        let usage = cached.get_usage("test-quota").await.unwrap();
        assert_eq!(usage.quota_id, "test-quota");
    }

    #[tokio::test]
    async fn test_cached_client_expired_policy() {
        let stub = StubQuotaClient::new();
        let cached = CachedQuotaClient::new(stub, Duration::from_millis(1));

        let _p1 = cached.get_policy("test-quota").await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        let _p2 = cached.get_policy("test-quota").await.unwrap();

        let count = *cached.inner.call_count.lock().unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_inmemory_quota_client_basic_flow() {
        let client = InMemoryQuotaClient::new();

        let status = client.check("quota-a", 10).await.unwrap();
        assert!(status.allowed);
        assert_eq!(status.limit, 1000);

        let usage = client.increment("quota-a", 200).await.unwrap();
        assert_eq!(usage.used, 200);

        let status_after = client.check("quota-a", 900).await.unwrap();
        assert!(!status_after.allowed);
    }
}
