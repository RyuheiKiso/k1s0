use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use k1s0_bb_core::{Component, ComponentError, ComponentStatus};

use crate::StateStoreError;
use crate::traits::{StateEntry, StateStore};

struct Entry {
    value: Vec<u8>,
    etag: String,
}

/// InMemoryStateStore はテスト・開発用のインメモリ StateStore 実装。
pub struct InMemoryStateStore {
    name: String,
    status: RwLock<ComponentStatus>,
    store: RwLock<HashMap<String, Entry>>,
}

impl InMemoryStateStore {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: RwLock::new(ComponentStatus::Uninitialized),
            store: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Component for InMemoryStateStore {
    fn name(&self) -> &str {
        &self.name
    }

    fn component_type(&self) -> &str {
        "statestore"
    }

    async fn init(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Ready;
        info!(component = %self.name, "InMemoryStateStore を初期化しました");
        Ok(())
    }

    async fn close(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Closed;
        let mut store = self.store.write().await;
        store.clear();
        info!(component = %self.name, "InMemoryStateStore をクローズしました");
        Ok(())
    }

    async fn status(&self) -> ComponentStatus {
        self.status.read().await.clone()
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("backend".to_string(), "memory".to_string());
        meta
    }
}

#[async_trait]
impl StateStore for InMemoryStateStore {
    async fn get(&self, key: &str) -> Result<Option<StateEntry>, StateStoreError> {
        let store = self.store.read().await;
        Ok(store.get(key).map(|entry| StateEntry {
            key: key.to_string(),
            value: entry.value.clone(),
            etag: entry.etag.clone(),
        }))
    }

    async fn set(
        &self,
        key: &str,
        value: &[u8],
        etag: Option<&str>,
    ) -> Result<String, StateStoreError> {
        let mut store = self.store.write().await;
        // ETag が指定されている場合、既存エントリの ETag と一致するか検証する。
        if let Some(expected_etag) = etag
            && let Some(existing) = store.get(key)
            && existing.etag != expected_etag
        {
            return Err(StateStoreError::ETagMismatch {
                expected: expected_etag.to_string(),
                actual: existing.etag.clone(),
            });
        }
        let new_etag = Uuid::new_v4().to_string();
        store.insert(
            key.to_string(),
            Entry {
                value: value.to_vec(),
                etag: new_etag.clone(),
            },
        );
        Ok(new_etag)
    }

    async fn delete(&self, key: &str, etag: Option<&str>) -> Result<(), StateStoreError> {
        let mut store = self.store.write().await;
        // ETag が指定されている場合、既存エントリの ETag と一致するか検証する。
        if let Some(expected_etag) = etag
            && let Some(existing) = store.get(key)
            && existing.etag != expected_etag
        {
            return Err(StateStoreError::ETagMismatch {
                expected: expected_etag.to_string(),
                actual: existing.etag.clone(),
            });
        }
        store.remove(key);
        Ok(())
    }

    async fn bulk_get(&self, keys: &[&str]) -> Result<Vec<StateEntry>, StateStoreError> {
        let store = self.store.read().await;
        let entries = keys
            .iter()
            .filter_map(|key| {
                store.get(*key).map(|entry| StateEntry {
                    key: (*key).to_string(),
                    value: entry.value.clone(),
                    etag: entry.etag.clone(),
                })
            })
            .collect();
        Ok(entries)
    }

    async fn bulk_set(&self, entries: &[(&str, &[u8])]) -> Result<Vec<String>, StateStoreError> {
        let mut store = self.store.write().await;
        let mut etags = Vec::with_capacity(entries.len());
        for (key, value) in entries {
            let new_etag = Uuid::new_v4().to_string();
            store.insert(
                (*key).to_string(),
                Entry {
                    value: value.to_vec(),
                    etag: new_etag.clone(),
                },
            );
            etags.push(new_etag);
        }
        Ok(etags)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // InMemoryStateStore の初期化後にステータスが Ready になることを確認する。
    #[tokio::test]
    async fn test_init_and_status() {
        let store = InMemoryStateStore::new("test-store");
        assert_eq!(store.status().await, ComponentStatus::Uninitialized);
        store.init().await.unwrap();
        assert_eq!(store.status().await, ComponentStatus::Ready);
    }

    // 値をセットした後に正しく取得でき、ETag が返されることを確認する。
    #[tokio::test]
    async fn test_set_and_get() {
        let store = InMemoryStateStore::new("test-store");
        store.init().await.unwrap();

        let etag = store.set("key1", b"value1", None).await.unwrap();
        assert!(!etag.is_empty());

        let entry = store.get("key1").await.unwrap().unwrap();
        assert_eq!(entry.key, "key1");
        assert_eq!(entry.value, b"value1");
        assert_eq!(entry.etag, etag);
    }

    // 存在しないキーを取得すると None が返されることを確認する。
    #[tokio::test]
    async fn test_get_not_found() {
        let store = InMemoryStateStore::new("test-store");
        let result = store.get("missing").await.unwrap();
        assert!(result.is_none());
    }

    // 正しい ETag を指定して値を更新できることを確認する。
    #[tokio::test]
    async fn test_set_with_etag() {
        let store = InMemoryStateStore::new("test-store");
        let etag = store.set("key1", b"value1", None).await.unwrap();
        let new_etag = store.set("key1", b"value2", Some(&etag)).await.unwrap();
        assert_ne!(etag, new_etag);

        let entry = store.get("key1").await.unwrap().unwrap();
        assert_eq!(entry.value, b"value2");
    }

    // 不正な ETag を指定してセットすると ETagMismatch エラーになることを確認する。
    #[tokio::test]
    async fn test_set_etag_mismatch() {
        let store = InMemoryStateStore::new("test-store");
        store.set("key1", b"value1", None).await.unwrap();
        let result = store.set("key1", b"value2", Some("wrong-etag")).await;
        assert!(matches!(result, Err(StateStoreError::ETagMismatch { .. })));
    }

    // キーを削除後に取得すると None が返されることを確認する。
    #[tokio::test]
    async fn test_delete() {
        let store = InMemoryStateStore::new("test-store");
        store.set("key1", b"value1", None).await.unwrap();
        store.delete("key1", None).await.unwrap();
        assert!(store.get("key1").await.unwrap().is_none());
    }

    // 不正な ETag を指定して削除すると ETagMismatch エラーになることを確認する。
    #[tokio::test]
    async fn test_delete_with_etag_mismatch() {
        let store = InMemoryStateStore::new("test-store");
        store.set("key1", b"value1", None).await.unwrap();
        let result = store.delete("key1", Some("wrong-etag")).await;
        assert!(matches!(result, Err(StateStoreError::ETagMismatch { .. })));
    }

    // 複数キーを一括取得し、存在するキーのエントリのみ返されることを確認する。
    #[tokio::test]
    async fn test_bulk_get() {
        let store = InMemoryStateStore::new("test-store");
        store.set("a", b"1", None).await.unwrap();
        store.set("b", b"2", None).await.unwrap();
        store.set("c", b"3", None).await.unwrap();

        let entries = store.bulk_get(&["a", "c", "missing"]).await.unwrap();
        assert_eq!(entries.len(), 2);
    }

    // 複数エントリを一括セットし、それぞれ ETag が返されることを確認する。
    #[tokio::test]
    async fn test_bulk_set() {
        let store = InMemoryStateStore::new("test-store");
        let etags = store.bulk_set(&[("x", b"10"), ("y", b"20")]).await.unwrap();
        assert_eq!(etags.len(), 2);

        let entry = store.get("x").await.unwrap().unwrap();
        assert_eq!(entry.value, b"10");
    }

    // クローズ後にストアがクリアされステータスが Closed になることを確認する。
    #[tokio::test]
    async fn test_close_clears_store() {
        let store = InMemoryStateStore::new("test-store");
        store.set("key1", b"value1", None).await.unwrap();
        store.close().await.unwrap();
        assert_eq!(store.status().await, ComponentStatus::Closed);
    }

    // メタデータにバックエンドが "memory" として設定されていることを確認する。
    #[tokio::test]
    async fn test_metadata() {
        let store = InMemoryStateStore::new("test-store");
        let meta = store.metadata();
        assert_eq!(meta.get("backend").unwrap(), "memory");
    }
}
