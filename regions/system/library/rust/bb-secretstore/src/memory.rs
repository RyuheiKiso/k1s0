use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::info;

use k1s0_bb_core::{Component, ComponentError, ComponentStatus};

use crate::SecretStoreError;
use crate::traits::{SecretStore, SecretValue};

/// InMemorySecretStore はテスト・開発用のインメモリ SecretStore 実装。
pub struct InMemorySecretStore {
    name: String,
    status: RwLock<ComponentStatus>,
    store: RwLock<HashMap<String, SecretValue>>,
}

impl InMemorySecretStore {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: RwLock::new(ComponentStatus::Uninitialized),
            store: RwLock::new(HashMap::new()),
        }
    }

    /// テスト用にシークレットを追加する。
    pub async fn put_secret(&self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let mut store = self.store.write().await;
        store.insert(
            key.clone(),
            SecretValue {
                key,
                value: value.into(),
                metadata: HashMap::new(),
            },
        );
    }

    /// テスト用にメタデータ付きシークレットを追加する。
    pub async fn put_secret_with_metadata(
        &self,
        key: impl Into<String>,
        value: impl Into<String>,
        metadata: HashMap<String, String>,
    ) {
        let key = key.into();
        let mut store = self.store.write().await;
        store.insert(
            key.clone(),
            SecretValue {
                key,
                value: value.into(),
                metadata,
            },
        );
    }
}

#[async_trait]
impl Component for InMemorySecretStore {
    fn name(&self) -> &str {
        &self.name
    }

    fn component_type(&self) -> &str {
        "secretstore"
    }

    async fn init(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Ready;
        info!(component = %self.name, "InMemorySecretStore を初期化しました");
        Ok(())
    }

    async fn close(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Closed;
        let mut store = self.store.write().await;
        store.clear();
        info!(component = %self.name, "InMemorySecretStore をクローズしました");
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
impl SecretStore for InMemorySecretStore {
    async fn get_secret(&self, key: &str) -> Result<SecretValue, SecretStoreError> {
        let store = self.store.read().await;
        store
            .get(key)
            .cloned()
            .ok_or_else(|| SecretStoreError::NotFound(key.to_string()))
    }

    async fn bulk_get(
        &self,
        keys: &[&str],
    ) -> Result<HashMap<String, SecretValue>, SecretStoreError> {
        let store = self.store.read().await;
        let mut result = HashMap::new();
        for key in keys {
            if let Some(secret) = store.get(*key) {
                result.insert((*key).to_string(), secret.clone());
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // InMemorySecretStore の初期化後にステータスが Ready になることを確認する。
    #[tokio::test]
    async fn test_init_and_status() {
        let store = InMemorySecretStore::new("test-secrets");
        assert_eq!(store.status().await, ComponentStatus::Uninitialized);
        store.init().await.unwrap();
        assert_eq!(store.status().await, ComponentStatus::Ready);
    }

    // シークレットを追加して正しく取得できることを確認する。
    #[tokio::test]
    async fn test_put_and_get_secret() {
        let store = InMemorySecretStore::new("test-secrets");
        store.put_secret("db/password", "s3cr3t").await;

        let secret = store.get_secret("db/password").await.unwrap();
        assert_eq!(secret.key, "db/password");
        assert_eq!(secret.value, "s3cr3t");
        assert!(secret.metadata.is_empty());
    }

    // 存在しないキーを取得しようとすると NotFound エラーになることを確認する。
    #[tokio::test]
    async fn test_get_secret_not_found() {
        let store = InMemorySecretStore::new("test-secrets");
        let result = store.get_secret("missing").await;
        assert!(matches!(result, Err(SecretStoreError::NotFound(_))));
    }

    // 複数のシークレットを一括取得し、存在するキーのみ返されることを確認する。
    #[tokio::test]
    async fn test_bulk_get() {
        let store = InMemorySecretStore::new("test-secrets");
        store.put_secret("db/password", "pass1").await;
        store.put_secret("db/username", "admin").await;
        store.put_secret("api/key", "key123").await;

        let secrets = store
            .bulk_get(&["db/password", "api/key", "missing"])
            .await
            .unwrap();
        assert_eq!(secrets.len(), 2);
        assert_eq!(secrets.get("db/password").unwrap().value, "pass1");
        assert_eq!(secrets.get("api/key").unwrap().value, "key123");
    }

    // メタデータ付きシークレットを追加して正しく取得できることを確認する。
    #[tokio::test]
    async fn test_put_secret_with_metadata() {
        let store = InMemorySecretStore::new("test-secrets");
        let mut meta = HashMap::new();
        meta.insert("version".to_string(), "2".to_string());
        store
            .put_secret_with_metadata("db/password", "s3cr3t", meta)
            .await;

        let secret = store.get_secret("db/password").await.unwrap();
        assert_eq!(secret.metadata.get("version").unwrap(), "2");
    }

    // クローズ後にストアがクリアされステータスが Closed になることを確認する。
    #[tokio::test]
    async fn test_close_clears_store() {
        let store = InMemorySecretStore::new("test-secrets");
        store.put_secret("db/password", "s3cr3t").await;
        store.close().await.unwrap();
        assert_eq!(store.status().await, ComponentStatus::Closed);
    }

    // メタデータにバックエンドが "memory" として設定されていることを確認する。
    #[tokio::test]
    async fn test_metadata() {
        let store = InMemorySecretStore::new("test-secrets");
        let meta = store.metadata();
        assert_eq!(meta.get("backend").unwrap(), "memory");
    }

    // コンポーネントタイプが "secretstore" であることを確認する。
    #[tokio::test]
    async fn test_component_type() {
        let store = InMemorySecretStore::new("test-secrets");
        assert_eq!(store.component_type(), "secretstore");
    }
}
