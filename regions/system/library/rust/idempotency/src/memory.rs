use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{IdempotencyError, IdempotencyRecord, IdempotencyStatus};
use crate::store::IdempotencyStore;

#[derive(Clone)]
pub struct InMemoryIdempotencyStore {
    data: Arc<RwLock<HashMap<String, IdempotencyRecord>>>,
}

impl InMemoryIdempotencyStore {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 期限切れエントリを除去する
    async fn cleanup_expired(&self) {
        let mut map = self.data.write().await;
        map.retain(|_, record| !record.is_expired());
    }
}

impl Default for InMemoryIdempotencyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IdempotencyStore for InMemoryIdempotencyStore {
    async fn get(&self, key: &str) -> Result<Option<IdempotencyRecord>, IdempotencyError> {
        self.cleanup_expired().await;
        let map = self.data.read().await;
        Ok(map.get(key).cloned())
    }

    async fn insert(&self, record: IdempotencyRecord) -> Result<(), IdempotencyError> {
        self.cleanup_expired().await;
        let mut map = self.data.write().await;
        if map.contains_key(&record.key) {
            return Err(IdempotencyError::Duplicate {
                key: record.key.clone(),
            });
        }
        map.insert(record.key.clone(), record);
        Ok(())
    }

    async fn update(
        &self,
        key: &str,
        status: IdempotencyStatus,
        response_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError> {
        let mut map = self.data.write().await;
        let record = map.get_mut(key).ok_or_else(|| IdempotencyError::NotFound {
            key: key.to_string(),
        })?;
        record.status = status;
        record.response_body = response_body;
        record.response_status = response_status;
        record.completed_at = Some(chrono::Utc::now());
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool, IdempotencyError> {
        let mut map = self.data.write().await;
        Ok(map.remove(key).is_some())
    }
}
