use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use k1s0_bb_core::{Component, ComponentError, ComponentStatus};
use k1s0_cache::CacheClient;

use crate::traits::{StateEntry, StateStore};
use crate::StateStoreError;

/// RedisStateStore は Redis ベースの StateStore 実装。
/// k1s0-cache の CacheClient をラップする。
pub struct RedisStateStore {
    name: String,
    client: Arc<dyn CacheClient>,
    status: RwLock<ComponentStatus>,
}

impl RedisStateStore {
    pub fn new(name: impl Into<String>, client: Arc<dyn CacheClient>) -> Self {
        Self {
            name: name.into(),
            client,
            status: RwLock::new(ComponentStatus::Uninitialized),
        }
    }

    fn etag_key(key: &str) -> String {
        format!("{key}:__etag")
    }
}

#[async_trait]
impl Component for RedisStateStore {
    fn name(&self) -> &str {
        &self.name
    }

    fn component_type(&self) -> &str {
        "statestore"
    }

    async fn init(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Ready;
        info!(component = %self.name, "RedisStateStore を初期化しました");
        Ok(())
    }

    async fn close(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Closed;
        info!(component = %self.name, "RedisStateStore をクローズしました");
        Ok(())
    }

    async fn status(&self) -> ComponentStatus {
        self.status.read().await.clone()
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("backend".to_string(), "redis".to_string());
        meta
    }
}

#[async_trait]
impl StateStore for RedisStateStore {
    async fn get(&self, key: &str) -> Result<Option<StateEntry>, StateStoreError> {
        let value = self
            .client
            .get(key)
            .await
            .map_err(|e| StateStoreError::Connection(e.to_string()))?;

        match value {
            Some(v) => {
                let etag = self
                    .client
                    .get(&Self::etag_key(key))
                    .await
                    .map_err(|e| StateStoreError::Connection(e.to_string()))?
                    .unwrap_or_default();
                Ok(Some(StateEntry {
                    key: key.to_string(),
                    value: v.into_bytes(),
                    etag,
                }))
            }
            None => Ok(None),
        }
    }

    async fn set(
        &self,
        key: &str,
        value: &[u8],
        etag: Option<&str>,
    ) -> Result<String, StateStoreError> {
        if let Some(expected_etag) = etag {
            let current_etag = self
                .client
                .get(&Self::etag_key(key))
                .await
                .map_err(|e| StateStoreError::Connection(e.to_string()))?;
            if let Some(ref current) = current_etag {
                if current != expected_etag {
                    return Err(StateStoreError::ETagMismatch {
                        expected: expected_etag.to_string(),
                        actual: current.clone(),
                    });
                }
            }
        }

        let value_str = String::from_utf8(value.to_vec())
            .map_err(|e| StateStoreError::Serialization(e.to_string()))?;
        self.client
            .set(key, &value_str, None)
            .await
            .map_err(|e| StateStoreError::Connection(e.to_string()))?;

        let new_etag = Uuid::new_v4().to_string();
        self.client
            .set(&Self::etag_key(key), &new_etag, None)
            .await
            .map_err(|e| StateStoreError::Connection(e.to_string()))?;

        Ok(new_etag)
    }

    async fn delete(&self, key: &str, etag: Option<&str>) -> Result<(), StateStoreError> {
        if let Some(expected_etag) = etag {
            let current_etag = self
                .client
                .get(&Self::etag_key(key))
                .await
                .map_err(|e| StateStoreError::Connection(e.to_string()))?;
            if let Some(ref current) = current_etag {
                if current != expected_etag {
                    return Err(StateStoreError::ETagMismatch {
                        expected: expected_etag.to_string(),
                        actual: current.clone(),
                    });
                }
            }
        }

        self.client
            .delete(key)
            .await
            .map_err(|e| StateStoreError::Connection(e.to_string()))?;
        self.client
            .delete(&Self::etag_key(key))
            .await
            .map_err(|e| StateStoreError::Connection(e.to_string()))?;
        Ok(())
    }

    async fn bulk_get(&self, keys: &[&str]) -> Result<Vec<StateEntry>, StateStoreError> {
        let mut entries = Vec::new();
        for key in keys {
            if let Some(entry) = self.get(key).await? {
                entries.push(entry);
            }
        }
        Ok(entries)
    }

    async fn bulk_set(&self, entries: &[(&str, &[u8])]) -> Result<Vec<String>, StateStoreError> {
        let mut etags = Vec::with_capacity(entries.len());
        for (key, value) in entries {
            let etag = self.set(key, value, None).await?;
            etags.push(etag);
        }
        Ok(etags)
    }
}
