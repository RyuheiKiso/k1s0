use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{CacheClient, CacheError};

struct Entry {
    value: String,
    expires_at: Option<Instant>,
}

impl Entry {
    fn is_expired(&self) -> bool {
        self.expires_at
            .map_or(false, |exp| exp <= Instant::now())
    }
}

#[derive(Clone)]
pub struct InMemoryCacheClient {
    store: Arc<RwLock<HashMap<String, Entry>>>,
}

impl InMemoryCacheClient {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryCacheClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CacheClient for InMemoryCacheClient {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        let store = self.store.read().await;
        match store.get(key) {
            Some(entry) if !entry.is_expired() => Ok(Some(entry.value.clone())),
            Some(_) => Ok(None),
            None => Ok(None),
        }
    }

    async fn set(&self, key: &str, value: &str, ttl: Option<Duration>) -> Result<(), CacheError> {
        let mut store = self.store.write().await;
        store.insert(
            key.to_string(),
            Entry {
                value: value.to_string(),
                expires_at: ttl.map(|d| Instant::now() + d),
            },
        );
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool, CacheError> {
        let mut store = self.store.write().await;
        Ok(store.remove(key).is_some())
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let store = self.store.read().await;
        Ok(store.get(key).map_or(false, |e| !e.is_expired()))
    }

    async fn set_nx(&self, key: &str, value: &str, ttl: Duration) -> Result<bool, CacheError> {
        let mut store = self.store.write().await;
        if store.get(key).map_or(true, |e| e.is_expired()) {
            store.insert(
                key.to_string(),
                Entry {
                    value: value.to_string(),
                    expires_at: Some(Instant::now() + ttl),
                },
            );
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn expire(&self, key: &str, ttl: Duration) -> Result<bool, CacheError> {
        let mut store = self.store.write().await;
        if let Some(entry) = store.get_mut(key) {
            if !entry.is_expired() {
                entry.expires_at = Some(Instant::now() + ttl);
                return Ok(true);
            }
        }
        Ok(false)
    }
}
