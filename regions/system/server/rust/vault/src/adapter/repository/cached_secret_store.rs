use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::entity::secret::Secret;
use crate::domain::repository::SecretStore;
use crate::infrastructure::cache::SecretCache;

pub struct CachedSecretStore {
    inner: Arc<dyn SecretStore>,
    cache: Arc<SecretCache>,
}

impl CachedSecretStore {
    pub fn new(inner: Arc<dyn SecretStore>, cache: Arc<SecretCache>) -> Self {
        Self { inner, cache }
    }
}

#[async_trait]
impl SecretStore for CachedSecretStore {
    async fn get(&self, path: &str, version: Option<i64>) -> anyhow::Result<Secret> {
        if let Some(cached) = self.cache.get(path, version).await {
            return Ok((*cached).clone());
        }

        let secret = self.inner.get(path, version).await?;
        self.cache
            .insert(path, version, Arc::new(secret.clone()))
            .await;
        Ok(secret)
    }

    async fn set(&self, path: &str, data: HashMap<String, String>) -> anyhow::Result<i64> {
        let version = self.inner.set(path, data).await?;
        self.cache.invalidate(path).await;
        Ok(version)
    }

    async fn delete(&self, path: &str, versions: Vec<i64>) -> anyhow::Result<()> {
        self.inner.delete(path, versions).await?;
        self.cache.invalidate(path).await;
        Ok(())
    }

    async fn list(&self, path_prefix: &str) -> anyhow::Result<Vec<String>> {
        self.inner.list(path_prefix).await
    }

    async fn exists(&self, path: &str) -> anyhow::Result<bool> {
        self.inner.exists(path).await
    }
}

