use anyhow::Context;
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct JwksProvider {
    inner: Arc<Inner>,
}

struct Inner {
    url: String,
    ttl: Duration,
    client: reqwest::Client,
    cache: RwLock<Option<CachedJwks>>,
}

struct CachedJwks {
    fetched_at: Instant,
    value: Value,
}

impl JwksProvider {
    pub fn new(url: String, ttl: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap_or_default();
        Self {
            inner: Arc::new(Inner {
                url,
                ttl,
                client,
                cache: RwLock::new(None),
            }),
        }
    }

    pub async fn get(&self) -> anyhow::Result<Value> {
        {
            let cache = self.inner.cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.fetched_at.elapsed() < self.inner.ttl {
                    return Ok(cached.value.clone());
                }
            }
        }

        let value = self.fetch().await?;
        let mut cache = self.inner.cache.write().await;
        *cache = Some(CachedJwks {
            fetched_at: Instant::now(),
            value: value.clone(),
        });
        Ok(value)
    }

    async fn fetch(&self) -> anyhow::Result<Value> {
        let response = self
            .inner
            .client
            .get(&self.inner.url)
            .send()
            .await
            .with_context(|| format!("failed to fetch JWKS from {}", self.inner.url))?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("JWKS endpoint returned non-success status: {status}");
        }

        response
            .json::<Value>()
            .await
            .with_context(|| format!("failed to parse JWKS response as JSON from {}", self.inner.url))
    }
}
