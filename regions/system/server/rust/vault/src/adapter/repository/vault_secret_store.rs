use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::adapter::gateway::VaultKvClient;
use crate::domain::entity::secret::{Secret, SecretValue, SecretVersion};
use crate::domain::repository::SecretStore;

/// HashiCorp Vault KV v2 をバックエンドとする SecretStore 実装。
pub struct VaultSecretStore {
    client: Arc<VaultKvClient>,
}

impl VaultSecretStore {
    pub fn new(client: Arc<VaultKvClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl SecretStore for VaultSecretStore {
    async fn get(&self, path: &str, _version: Option<i64>) -> anyhow::Result<Secret> {
        let data = self.client.read_secret(path).await?;
        let now = chrono::Utc::now();
        Ok(Secret {
            path: path.to_string(),
            current_version: 1,
            versions: vec![SecretVersion {
                version: 1,
                value: SecretValue { data },
                created_at: now,
                destroyed: false,
            }],
            created_at: now,
            updated_at: now,
        })
    }

    async fn set(&self, path: &str, data: HashMap<String, String>) -> anyhow::Result<i64> {
        self.client.write_secret(path, &data).await?;
        Ok(1)
    }

    async fn delete(&self, path: &str, _versions: Vec<i64>) -> anyhow::Result<()> {
        self.client.delete_secret(path).await
    }

    async fn list(&self, path_prefix: &str) -> anyhow::Result<Vec<String>> {
        self.client.list_secrets(path_prefix).await
    }

    async fn exists(&self, path: &str) -> anyhow::Result<bool> {
        match self.client.read_secret(path).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用のモック VaultKvClient 相当のストア。
    /// 実際の VaultKvClient は外部依存のため、SecretStore trait レベルでテストする。
    struct MockVaultSecretStore {
        data: tokio::sync::RwLock<HashMap<String, HashMap<String, String>>>,
    }

    impl MockVaultSecretStore {
        fn new() -> Self {
            Self {
                data: tokio::sync::RwLock::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl SecretStore for MockVaultSecretStore {
        async fn get(&self, path: &str, _version: Option<i64>) -> anyhow::Result<Secret> {
            let store = self.data.read().await;
            let data = store
                .get(path)
                .ok_or_else(|| anyhow::anyhow!("secret not found: {}", path))?
                .clone();
            let now = chrono::Utc::now();
            Ok(Secret {
                path: path.to_string(),
                current_version: 1,
                versions: vec![SecretVersion {
                    version: 1,
                    value: SecretValue { data },
                    created_at: now,
                    destroyed: false,
                }],
                created_at: now,
                updated_at: now,
            })
        }

        async fn set(&self, path: &str, data: HashMap<String, String>) -> anyhow::Result<i64> {
            self.data.write().await.insert(path.to_string(), data);
            Ok(1)
        }

        async fn delete(&self, path: &str, _versions: Vec<i64>) -> anyhow::Result<()> {
            self.data.write().await.remove(path);
            Ok(())
        }

        async fn list(&self, path_prefix: &str) -> anyhow::Result<Vec<String>> {
            let store = self.data.read().await;
            Ok(store
                .keys()
                .filter(|k| k.starts_with(path_prefix))
                .cloned()
                .collect())
        }

        async fn exists(&self, path: &str) -> anyhow::Result<bool> {
            Ok(self.data.read().await.contains_key(path))
        }
    }

    #[tokio::test]
    async fn test_vault_secret_store_set_and_get() {
        let store = MockVaultSecretStore::new();
        let data = HashMap::from([("password".to_string(), "s3cret".to_string())]);

        let version = store.set("app/db", data.clone()).await.unwrap();
        assert_eq!(version, 1);

        let secret = store.get("app/db", None).await.unwrap();
        assert_eq!(secret.path, "app/db");
        assert_eq!(secret.versions[0].value.data["password"], "s3cret");
    }

    #[tokio::test]
    async fn test_vault_secret_store_delete() {
        let store = MockVaultSecretStore::new();
        let data = HashMap::from([("key".to_string(), "val".to_string())]);

        store.set("app/key", data).await.unwrap();
        assert!(store.exists("app/key").await.unwrap());

        store.delete("app/key", vec![]).await.unwrap();
        assert!(!store.exists("app/key").await.unwrap());
    }

    #[tokio::test]
    async fn test_vault_secret_store_list() {
        let store = MockVaultSecretStore::new();
        store
            .set(
                "app/a",
                HashMap::from([("k".to_string(), "1".to_string())]),
            )
            .await
            .unwrap();
        store
            .set(
                "app/b",
                HashMap::from([("k".to_string(), "2".to_string())]),
            )
            .await
            .unwrap();
        store
            .set(
                "other/c",
                HashMap::from([("k".to_string(), "3".to_string())]),
            )
            .await
            .unwrap();

        let mut keys = store.list("app/").await.unwrap();
        keys.sort();
        assert_eq!(keys, vec!["app/a", "app/b"]);
    }

    #[tokio::test]
    async fn test_vault_secret_store_exists() {
        let store = MockVaultSecretStore::new();
        assert!(!store.exists("missing").await.unwrap());

        store
            .set(
                "missing",
                HashMap::from([("k".to_string(), "v".to_string())]),
            )
            .await
            .unwrap();
        assert!(store.exists("missing").await.unwrap());
    }

    #[tokio::test]
    async fn test_vault_secret_store_get_not_found() {
        let store = MockVaultSecretStore::new();
        let result = store.get("nonexistent", None).await;
        assert!(result.is_err());
    }
}
