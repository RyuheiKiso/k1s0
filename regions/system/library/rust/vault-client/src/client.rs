use async_trait::async_trait;
use std::collections::HashMap;

use crate::config::VaultClientConfig;
use crate::error::VaultError;
use crate::secret::{Secret, SecretRotatedEvent};

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait VaultClient: Send + Sync {
    async fn get_secret(&self, path: &str) -> Result<Secret, VaultError>;
    async fn get_secret_value(&self, path: &str, key: &str) -> Result<String, VaultError>;
    async fn list_secrets(&self, path_prefix: &str) -> Result<Vec<String>, VaultError>;
    async fn watch_secret(
        &self,
        path: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<SecretRotatedEvent>, VaultError>;
}

pub struct InMemoryVaultClient {
    config: VaultClientConfig,
    // async コンテキストで await ポイントをまたいでロックを保持するため tokio::sync::Mutex を使用する
    store: tokio::sync::Mutex<HashMap<String, Secret>>,
}

/// `InMemoryVaultClient` `のデフォルト実装（new()` と同じ）。
impl Default for InMemoryVaultClient {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryVaultClient {
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(VaultClientConfig::default())
    }

    #[must_use]
    pub fn with_config(config: VaultClientConfig) -> Self {
        Self {
            config,
            // 空の HashMap で tokio::sync::Mutex を初期化する
            store: tokio::sync::Mutex::new(HashMap::new()),
        }
    }

    pub fn config(&self) -> &VaultClientConfig {
        &self.config
    }

    // async コンテキストで安全にロックを取得するため async fn として定義する
    pub async fn put_secret(&self, secret: Secret) {
        // シークレットストアへの書き込みロックを非同期で取得する
        let mut store = self.store.lock().await;
        store.insert(secret.path.clone(), secret);
    }
}

#[async_trait]
impl VaultClient for InMemoryVaultClient {
    async fn get_secret(&self, path: &str) -> Result<Secret, VaultError> {
        // シークレットストアからの読み取りロックを非同期で取得する
        let store = self.store.lock().await;
        store
            .get(path)
            .cloned()
            .ok_or_else(|| VaultError::NotFound(path.to_string()))
    }

    async fn get_secret_value(&self, path: &str, key: &str) -> Result<String, VaultError> {
        let secret = self.get_secret(path).await?;
        secret
            .data
            .get(key)
            .cloned()
            .ok_or_else(|| VaultError::NotFound(format!("{path}/{key}")))
    }

    async fn list_secrets(&self, path_prefix: &str) -> Result<Vec<String>, VaultError> {
        // シークレットストアからの読み取りロックを非同期で取得する
        let store = self.store.lock().await;
        let paths: Vec<String> = store
            .keys()
            .filter(|k| k.starts_with(path_prefix))
            .cloned()
            .collect();
        Ok(paths)
    }

    async fn watch_secret(
        &self,
        _path: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<SecretRotatedEvent>, VaultError> {
        let (_tx, rx) = tokio::sync::mpsc::channel(16);
        Ok(rx)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_config() -> VaultClientConfig {
        VaultClientConfig::new("http://localhost:8080")
    }

    fn make_secret(path: &str) -> Secret {
        let mut data = HashMap::new();
        data.insert("password".to_string(), "s3cr3t".to_string());
        data.insert("username".to_string(), "admin".to_string());
        Secret {
            path: path.to_string(),
            data,
            version: 1,
            created_at: Utc::now(),
        }
    }

    // put したシークレットが get_secret で正しく取得できることを確認する。
    #[tokio::test]
    async fn test_get_secret_found() {
        let client = InMemoryVaultClient::with_config(make_config());
        client
            .put_secret(make_secret("system/database/primary"))
            .await;
        let secret = client.get_secret("system/database/primary").await.unwrap();
        assert_eq!(secret.path, "system/database/primary");
        assert_eq!(secret.data.get("password").unwrap(), "s3cr3t");
    }

    // 未登録パスの get_secret が NotFound エラーを返すことを確認する。
    #[tokio::test]
    async fn test_get_secret_not_found() {
        let client = InMemoryVaultClient::with_config(make_config());
        let err = client.get_secret("missing/path").await.unwrap_err();
        assert!(matches!(err, VaultError::NotFound(_)));
    }

    // get_secret_value でシークレットの特定キーの値が取得できることを確認する。
    #[tokio::test]
    async fn test_get_secret_value() {
        let client = InMemoryVaultClient::with_config(make_config());
        client.put_secret(make_secret("system/db")).await;
        let value = client
            .get_secret_value("system/db", "password")
            .await
            .unwrap();
        assert_eq!(value, "s3cr3t");
    }

    // 存在しないキーへの get_secret_value が NotFound エラーを返すことを確認する。
    #[tokio::test]
    async fn test_get_secret_value_key_not_found() {
        let client = InMemoryVaultClient::with_config(make_config());
        client.put_secret(make_secret("system/db")).await;
        let err = client
            .get_secret_value("system/db", "missing_key")
            .await
            .unwrap_err();
        assert!(matches!(err, VaultError::NotFound(_)));
    }

    // プレフィックスに一致するシークレットのパス一覧が正しく返されることを確認する。
    #[tokio::test]
    async fn test_list_secrets() {
        let client = InMemoryVaultClient::with_config(make_config());
        client.put_secret(make_secret("system/db/primary")).await;
        client.put_secret(make_secret("system/db/replica")).await;
        client.put_secret(make_secret("business/api/key")).await;

        let paths = client.list_secrets("system/").await.unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.iter().all(|p| p.starts_with("system/")));
    }

    // 一致するシークレットがない場合に空リストが返されることを確認する。
    #[tokio::test]
    async fn test_list_secrets_empty() {
        let client = InMemoryVaultClient::with_config(make_config());
        let paths = client.list_secrets("nothing/").await.unwrap();
        assert!(paths.is_empty());
    }

    // watch_secret が有効な受信チャンネルを返すことを確認する。
    #[tokio::test]
    async fn test_watch_secret_returns_receiver() {
        let client = InMemoryVaultClient::with_config(make_config());
        let rx = client.watch_secret("system/db").await.unwrap();
        drop(rx);
    }

    // VaultError::NotFound バリアントが正しく生成されることを確認する。
    #[test]
    fn test_vault_error_not_found() {
        let err = VaultError::NotFound("system/missing".to_string());
        assert!(matches!(err, VaultError::NotFound(_)));
    }

    // VaultError::PermissionDenied バリアントが正しく生成されることを確認する。
    #[test]
    fn test_vault_error_permission_denied() {
        let err = VaultError::PermissionDenied("system/secret".to_string());
        assert!(matches!(err, VaultError::PermissionDenied(_)));
    }

    // VaultError::ServerError バリアントが正しく生成されることを確認する。
    #[test]
    fn test_vault_error_server_error() {
        let err = VaultError::ServerError("internal".to_string());
        assert!(matches!(err, VaultError::ServerError(_)));
    }

    // VaultError::Timeout バリアントが正しく生成されることを確認する。
    #[test]
    fn test_vault_error_timeout() {
        let err = VaultError::Timeout;
        assert!(matches!(err, VaultError::Timeout));
    }

    // VaultError::LeaseExpired バリアントが正しく生成されることを確認する。
    #[test]
    fn test_vault_error_lease_expired() {
        let err = VaultError::LeaseExpired("system/db".to_string());
        assert!(matches!(err, VaultError::LeaseExpired(_)));
    }

    // シークレットの data フィールドのキーへのアクセスと version が正しいことを確認する。
    #[test]
    fn test_secret_data_access() {
        let secret = make_secret("system/db");
        assert_eq!(secret.data.get("password").unwrap(), "s3cr3t");
        assert_eq!(secret.data.get("username").unwrap(), "admin");
        assert_eq!(secret.version, 1);
    }

    // ビルダーパターンで設定した値が正しく VaultClientConfig に反映されることを確認する。
    #[test]
    fn test_config_builder() {
        let config = VaultClientConfig::new("http://localhost:8080")
            .cache_ttl(std::time::Duration::from_secs(300))
            .cache_max_capacity(100);
        assert_eq!(config.server_url, "http://localhost:8080");
        assert_eq!(config.cache_ttl, std::time::Duration::from_secs(300));
        assert_eq!(config.cache_max_capacity, 100);
    }

    // VaultClientConfig のデフォルト値が TTL 600 秒・容量 500 であることを確認する。
    #[test]
    fn test_config_defaults() {
        let config = VaultClientConfig::new("http://vault:8080");
        assert_eq!(config.cache_ttl, std::time::Duration::from_secs(600));
        assert_eq!(config.cache_max_capacity, 500);
    }

    // SecretRotatedEvent の path と version が正しく設定されることを確認する。
    #[test]
    fn test_secret_rotated_event() {
        let event = crate::secret::SecretRotatedEvent {
            path: "system/db".to_string(),
            version: 2,
        };
        assert_eq!(event.path, "system/db");
        assert_eq!(event.version, 2);
    }
}
