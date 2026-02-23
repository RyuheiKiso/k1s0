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
    async fn increment(
        &self,
        quota_id: &str,
        amount: u64,
    ) -> Result<QuotaUsage, QuotaClientError>;
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

    async fn increment(
        &self,
        quota_id: &str,
        amount: u64,
    ) -> Result<QuotaUsage, QuotaClientError> {
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

    async fn increment(
        &self,
        quota_id: &str,
        amount: u64,
    ) -> Result<QuotaUsage, QuotaClientError> {
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
}
