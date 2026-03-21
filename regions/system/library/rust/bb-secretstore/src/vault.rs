use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::info;

use k1s0_bb_core::{Component, ComponentError, ComponentStatus};
use k1s0_vault_client::VaultClient;

use crate::SecretStoreError;
use crate::traits::{SecretStore, SecretValue};

/// VaultSecretStore は HashiCorp Vault ベースの SecretStore 実装。
/// k1s0-vault-client の VaultClient をラップする。
pub struct VaultSecretStore {
    name: String,
    client: Arc<dyn VaultClient>,
    status: RwLock<ComponentStatus>,
}

impl VaultSecretStore {
    pub fn new(name: impl Into<String>, client: Arc<dyn VaultClient>) -> Self {
        Self {
            name: name.into(),
            client,
            status: RwLock::new(ComponentStatus::Uninitialized),
        }
    }
}

#[async_trait]
impl Component for VaultSecretStore {
    fn name(&self) -> &str {
        &self.name
    }

    fn component_type(&self) -> &str {
        "secretstore"
    }

    async fn init(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Ready;
        info!(component = %self.name, "VaultSecretStore を初期化しました");
        Ok(())
    }

    async fn close(&self) -> Result<(), ComponentError> {
        let mut status = self.status.write().await;
        *status = ComponentStatus::Closed;
        info!(component = %self.name, "VaultSecretStore をクローズしました");
        Ok(())
    }

    async fn status(&self) -> ComponentStatus {
        self.status.read().await.clone()
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("backend".to_string(), "vault".to_string());
        meta
    }
}

#[async_trait]
impl SecretStore for VaultSecretStore {
    async fn get_secret(&self, key: &str) -> Result<SecretValue, SecretStoreError> {
        let secret = self
            .client
            .get_secret(key)
            .await
            .map_err(|e| SecretStoreError::Connection(e.to_string()))?;

        // Vault の Secret.data (HashMap<String, String>) の最初の値をシークレット値として使用。
        // 複数キーがある場合は JSON シリアライズする。
        let value = if secret.data.len() == 1 {
            secret.data.values().next()
                .ok_or_else(|| SecretStoreError::NotFound(format!("シークレット '{}' のデータが空です", key)))?
                .clone()
        } else {
            serde_json::to_string(&secret.data).unwrap_or_else(|_| format!("{:?}", secret.data))
        };

        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), secret.version.to_string());

        Ok(SecretValue {
            key: key.to_string(),
            value,
            metadata,
        })
    }

    async fn bulk_get(
        &self,
        keys: &[&str],
    ) -> Result<HashMap<String, SecretValue>, SecretStoreError> {
        let mut result = HashMap::new();
        for key in keys {
            match self.get_secret(key).await {
                Ok(secret) => {
                    result.insert((*key).to_string(), secret);
                }
                Err(SecretStoreError::NotFound(_)) => {
                    // キーが見つからない場合はスキップ
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(result)
    }
}
