//! サービス間認証の設定構造体。

use serde::{Deserialize, Serialize};

/// refresh_before_secs のデフォルト値（120 秒）。
fn default_refresh_before_secs() -> u64 {
    120
}

/// timeout_secs のデフォルト値（10 秒）。
fn default_timeout_secs() -> u64 {
    10
}

/// ServiceAuthConfig はサービス間認証クライアントの設定を表す。
///
/// Keycloak の Client Credentials フローで使用する設定値を保持する。
/// YAML または環境変数から serde でデシリアライズ可能。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAuthConfig {
    /// Keycloak のトークンエンドポイント URL。
    /// 例: `https://auth.k1s0.internal.example.com/realms/k1s0/protocol/openid-connect/token`
    pub token_endpoint: String,

    /// OAuth2 クライアント ID（サービス名）。
    pub client_id: String,

    /// OAuth2 クライアントシークレット（Vault から取得）。
    pub client_secret: String,

    /// トークン検証に使用する JWKS URI。
    /// 省略した場合はトークン検証機能を無効にする。
    pub jwks_uri: Option<String>,

    /// トークン有効期限の何秒前にリフレッシュするか（デフォルト: 120 秒）。
    #[serde(default = "default_refresh_before_secs")]
    pub refresh_before_secs: u64,

    /// HTTP タイムアウト秒数（デフォルト: 10 秒）。
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

impl ServiceAuthConfig {
    /// 最小限の設定で ServiceAuthConfig を生成する。
    ///
    /// `refresh_before_secs` と `timeout_secs` はデフォルト値が使用される。
    pub fn new(token_endpoint: &str, client_id: &str, client_secret: &str) -> Self {
        Self {
            token_endpoint: token_endpoint.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            jwks_uri: None,
            refresh_before_secs: default_refresh_before_secs(),
            timeout_secs: default_timeout_secs(),
        }
    }

    /// JWKS URI を設定する。
    pub fn with_jwks_uri(mut self, jwks_uri: &str) -> Self {
        self.jwks_uri = Some(jwks_uri.to_string());
        self
    }

    /// リフレッシュ秒数を設定する。
    pub fn with_refresh_before_secs(mut self, secs: u64) -> Self {
        self.refresh_before_secs = secs;
        self
    }

    /// タイムアウト秒数を設定する。
    pub fn with_timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sets_required_fields() {
        let config = ServiceAuthConfig::new(
            "https://auth.example.com/token",
            "my-service",
            "secret-value",
        );

        assert_eq!(config.token_endpoint, "https://auth.example.com/token");
        assert_eq!(config.client_id, "my-service");
        assert_eq!(config.client_secret, "secret-value");
        assert!(config.jwks_uri.is_none());
    }

    #[test]
    fn test_default_refresh_before_secs() {
        let config = ServiceAuthConfig::new("https://auth.example.com/token", "svc", "sec");
        assert_eq!(config.refresh_before_secs, 120);
    }

    #[test]
    fn test_default_timeout_secs() {
        let config = ServiceAuthConfig::new("https://auth.example.com/token", "svc", "sec");
        assert_eq!(config.timeout_secs, 10);
    }

    #[test]
    fn test_with_jwks_uri() {
        let config = ServiceAuthConfig::new("https://auth.example.com/token", "svc", "sec")
            .with_jwks_uri("https://auth.example.com/certs");
        assert_eq!(
            config.jwks_uri.as_deref(),
            Some("https://auth.example.com/certs")
        );
    }

    #[test]
    fn test_with_refresh_before_secs() {
        let config = ServiceAuthConfig::new("https://auth.example.com/token", "svc", "sec")
            .with_refresh_before_secs(60);
        assert_eq!(config.refresh_before_secs, 60);
    }

    #[test]
    fn test_with_timeout_secs() {
        let config = ServiceAuthConfig::new("https://auth.example.com/token", "svc", "sec")
            .with_timeout_secs(30);
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_serde_defaults_applied() {
        // JSON に refresh_before_secs と timeout_secs が含まれない場合でもデフォルトが使われる
        let json = r#"{
            "token_endpoint": "https://auth.example.com/token",
            "client_id": "svc",
            "client_secret": "sec"
        }"#;

        let config: ServiceAuthConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.refresh_before_secs, 120);
        assert_eq!(config.timeout_secs, 10);
        assert!(config.jwks_uri.is_none());
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = ServiceAuthConfig::new("https://auth.example.com/token", "svc", "sec")
            .with_jwks_uri("https://auth.example.com/certs")
            .with_refresh_before_secs(90)
            .with_timeout_secs(15);

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ServiceAuthConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.token_endpoint, original.token_endpoint);
        assert_eq!(deserialized.client_id, original.client_id);
        assert_eq!(deserialized.client_secret, original.client_secret);
        assert_eq!(deserialized.jwks_uri, original.jwks_uri);
        assert_eq!(deserialized.refresh_before_secs, original.refresh_before_secs);
        assert_eq!(deserialized.timeout_secs, original.timeout_secs);
    }
}
