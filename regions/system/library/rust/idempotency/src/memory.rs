use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::store::IdempotencyStore;
use crate::{IdempotencyError, IdempotencyRecord, IdempotencyStatus};

#[derive(Clone)]
pub struct InMemoryIdempotencyStore {
    data: Arc<RwLock<HashMap<String, IdempotencyRecord>>>,
}

impl InMemoryIdempotencyStore {
    #[must_use] 
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

    async fn set(&self, record: IdempotencyRecord) -> Result<(), IdempotencyError> {
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

    async fn mark_completed(
        &self,
        key: &str,
        response_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError> {
        let mut map = self.data.write().await;
        let record = map.get_mut(key).ok_or_else(|| IdempotencyError::NotFound {
            key: key.to_string(),
        })?;
        record.status = IdempotencyStatus::Completed;
        record.response_body = response_body;
        record.response_status = response_status;
        record.completed_at = Some(chrono::Utc::now());
        Ok(())
    }

    async fn mark_failed(
        &self,
        key: &str,
        error_body: Option<String>,
        response_status: Option<u16>,
    ) -> Result<(), IdempotencyError> {
        let mut map = self.data.write().await;
        let record = map.get_mut(key).ok_or_else(|| IdempotencyError::NotFound {
            key: key.to_string(),
        })?;
        record.status = IdempotencyStatus::Failed;
        record.response_body = error_body;
        record.response_status = response_status;
        record.completed_at = Some(chrono::Utc::now());
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool, IdempotencyError> {
        let mut map = self.data.write().await;
        Ok(map.remove(key).is_some())
    }
}

// テストコードでは unwrap() を許可する（unwrap_used = "deny" はプロダクションコード向け）
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::store::IdempotencyStore;

    fn make_record(key: &str) -> IdempotencyRecord {
        IdempotencyRecord::new(key.to_string(), None)
    }

    /// set してから get で取得できる
    #[tokio::test]
    async fn set_and_get_returns_record() {
        let store = InMemoryIdempotencyStore::new();
        let record = make_record("key-1");
        store.set(record).await.unwrap();
        let got = store.get("key-1").await.unwrap();
        assert!(got.is_some());
        assert_eq!(got.unwrap().key, "key-1");
    }

    /// 存在しないキーの get は None を返す
    #[tokio::test]
    async fn get_missing_key_returns_none() {
        let store = InMemoryIdempotencyStore::new();
        let result = store.get("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    /// 同じキーで二回 set すると Duplicate エラーが返る
    #[tokio::test]
    async fn set_duplicate_key_returns_error() {
        let store = InMemoryIdempotencyStore::new();
        store.set(make_record("dup-key")).await.unwrap();
        let err = store.set(make_record("dup-key")).await.unwrap_err();
        assert!(matches!(err, IdempotencyError::Duplicate { .. }));
    }

    /// mark_completed でステータスが Completed になる
    #[tokio::test]
    async fn mark_completed_sets_status() {
        let store = InMemoryIdempotencyStore::new();
        store.set(make_record("comp-key")).await.unwrap();
        store
            .mark_completed("comp-key", Some(r#"{"ok":true}"#.to_string()), Some(200))
            .await
            .unwrap();
        let record = store.get("comp-key").await.unwrap().unwrap();
        assert_eq!(record.status, IdempotencyStatus::Completed);
        assert_eq!(record.response_status, Some(200));
    }

    /// mark_failed でステータスが Failed になる
    #[tokio::test]
    async fn mark_failed_sets_status() {
        let store = InMemoryIdempotencyStore::new();
        store.set(make_record("fail-key")).await.unwrap();
        store
            .mark_failed("fail-key", Some("error message".to_string()), Some(500))
            .await
            .unwrap();
        let record = store.get("fail-key").await.unwrap().unwrap();
        assert_eq!(record.status, IdempotencyStatus::Failed);
        assert_eq!(record.response_body, Some("error message".to_string()));
    }

    /// 存在しないキーの mark_completed は NotFound エラーを返す
    #[tokio::test]
    async fn mark_completed_missing_key_returns_not_found() {
        let store = InMemoryIdempotencyStore::new();
        let err = store.mark_completed("ghost", None, None).await.unwrap_err();
        assert!(matches!(err, IdempotencyError::NotFound { .. }));
    }

    /// delete で削除でき、その後 get は None を返す
    #[tokio::test]
    async fn delete_removes_record() {
        let store = InMemoryIdempotencyStore::new();
        store.set(make_record("del-key")).await.unwrap();
        let deleted = store.delete("del-key").await.unwrap();
        assert!(deleted);
        assert!(store.get("del-key").await.unwrap().is_none());
    }

    /// 存在しないキーの delete は false を返す
    #[tokio::test]
    async fn delete_nonexistent_returns_false() {
        let store = InMemoryIdempotencyStore::new();
        let deleted = store.delete("no-such").await.unwrap();
        assert!(!deleted);
    }

    /// IdempotencyRecord::new がデフォルトで Pending ステータスを持つ
    #[test]
    fn new_record_has_pending_status() {
        let record = IdempotencyRecord::new("test-key".to_string(), None);
        assert_eq!(record.status, IdempotencyStatus::Pending);
        assert!(record.expires_at.is_none());
        assert!(record.completed_at.is_none());
    }

    /// IdempotencyRecord::new が TTL を設定する
    #[test]
    fn new_record_with_ttl_sets_expires_at() {
        let record = IdempotencyRecord::new("ttl-key".to_string(), Some(3600));
        assert!(record.expires_at.is_some());
    }
}
