//! OIDC Discovery
//!
//! OpenID Connect Discovery 1.0 の実装。
//! `/.well-known/openid-configuration` からプロバイダー情報を取得する。
//!
//! # 使用例
//!
//! ```rust,ignore
//! use k1s0_auth::oidc::{OidcDiscovery, OidcConfig};
//!
//! let config = OidcConfig::new("https://auth.example.com");
//! let discovery = OidcDiscovery::new(config);
//!
//! // プロバイダー設定を取得
//! let provider = discovery.discover().await?;
//! println!("JWKS URI: {}", provider.jwks_uri);
//! ```

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::error::AuthError;

/// OIDC Discovery 設定
#[derive(Debug, Clone)]
pub struct OidcConfig {
    /// Issuer URL
    pub issuer: String,
    /// Discovery エンドポイント（オプション、デフォルトは issuer + /.well-known/openid-configuration）
    pub discovery_url: Option<String>,
    /// キャッシュ TTL（秒）
    pub cache_ttl_secs: u64,
    /// HTTP タイムアウト（ミリ秒）
    pub http_timeout_ms: u64,
}

impl OidcConfig {
    /// 新しい設定を作成
    pub fn new(issuer: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            discovery_url: None,
            cache_ttl_secs: 3600, // 1 hour
            http_timeout_ms: 10_000, // 10 seconds
        }
    }

    /// Discovery URL を設定
    pub fn with_discovery_url(mut self, url: impl Into<String>) -> Self {
        self.discovery_url = Some(url.into());
        self
    }

    /// キャッシュ TTL を設定
    pub fn with_cache_ttl_secs(mut self, ttl: u64) -> Self {
        self.cache_ttl_secs = ttl;
        self
    }

    /// HTTP タイムアウトを設定
    pub fn with_http_timeout_ms(mut self, timeout: u64) -> Self {
        self.http_timeout_ms = timeout;
        self
    }

    /// Discovery URL を取得
    pub fn get_discovery_url(&self) -> String {
        self.discovery_url.clone().unwrap_or_else(|| {
            let issuer = self.issuer.trim_end_matches('/');
            format!("{}/.well-known/openid-configuration", issuer)
        })
    }
}

/// OIDC プロバイダー設定
///
/// OpenID Connect Discovery 1.0 で定義されているプロバイダーメタデータ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcProviderConfig {
    /// Issuer
    pub issuer: String,

    /// Authorization エンドポイント
    #[serde(default)]
    pub authorization_endpoint: Option<String>,

    /// Token エンドポイント
    #[serde(default)]
    pub token_endpoint: Option<String>,

    /// UserInfo エンドポイント
    #[serde(default)]
    pub userinfo_endpoint: Option<String>,

    /// JWKS URI
    pub jwks_uri: String,

    /// Registration エンドポイント
    #[serde(default)]
    pub registration_endpoint: Option<String>,

    /// Introspection エンドポイント
    #[serde(default)]
    pub introspection_endpoint: Option<String>,

    /// Revocation エンドポイント
    #[serde(default)]
    pub revocation_endpoint: Option<String>,

    /// End Session エンドポイント
    #[serde(default)]
    pub end_session_endpoint: Option<String>,

    /// サポートするスコープ
    #[serde(default)]
    pub scopes_supported: Vec<String>,

    /// サポートするレスポンスタイプ
    #[serde(default)]
    pub response_types_supported: Vec<String>,

    /// サポートするレスポンスモード
    #[serde(default)]
    pub response_modes_supported: Vec<String>,

    /// サポートするグラントタイプ
    #[serde(default)]
    pub grant_types_supported: Vec<String>,

    /// サポートするサブジェクトタイプ
    #[serde(default)]
    pub subject_types_supported: Vec<String>,

    /// サポートする ID Token 署名アルゴリズム
    #[serde(default)]
    pub id_token_signing_alg_values_supported: Vec<String>,

    /// サポートするクレーム
    #[serde(default)]
    pub claims_supported: Vec<String>,

    /// サポートする Token エンドポイント認証方式
    #[serde(default)]
    pub token_endpoint_auth_methods_supported: Vec<String>,

    /// サポートするコードチャレンジ方式
    #[serde(default)]
    pub code_challenge_methods_supported: Vec<String>,
}

/// キャッシュエントリ
struct CachedProvider {
    config: OidcProviderConfig,
    fetched_at: Instant,
}

/// OIDC Discovery クライアント
pub struct OidcDiscovery {
    config: OidcConfig,
    client: reqwest::Client,
    cache: Arc<RwLock<Option<CachedProvider>>>,
}

impl OidcDiscovery {
    /// 新しい Discovery クライアントを作成
    pub fn new(config: OidcConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.http_timeout_ms))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// 設定を取得
    pub fn config(&self) -> &OidcConfig {
        &self.config
    }

    /// プロバイダー設定を取得（キャッシュを使用）
    pub async fn discover(&self) -> Result<OidcProviderConfig, AuthError> {
        // キャッシュをチェック
        {
            let cache = self.cache.read().await;
            if let Some(ref entry) = *cache {
                if entry.fetched_at.elapsed().as_secs() < self.config.cache_ttl_secs {
                    debug!(issuer = %self.config.issuer, "Using cached OIDC provider config");
                    return Ok(entry.config.clone());
                }
            }
        }

        // キャッシュミスまたは期限切れ - 再取得
        self.fetch_and_cache().await
    }

    /// プロバイダー設定を強制的に再取得
    pub async fn refresh(&self) -> Result<OidcProviderConfig, AuthError> {
        self.fetch_and_cache().await
    }

    /// キャッシュを無効化
    pub async fn invalidate_cache(&self) {
        let mut cache = self.cache.write().await;
        *cache = None;
    }

    /// プロバイダー設定を取得してキャッシュに保存
    async fn fetch_and_cache(&self) -> Result<OidcProviderConfig, AuthError> {
        let discovery_url = self.config.get_discovery_url();

        info!(url = %discovery_url, "Fetching OIDC provider configuration");

        let response = self
            .client
            .get(&discovery_url)
            .send()
            .await
            .map_err(|e| AuthError::discovery(format!("Failed to fetch discovery document: {}", e)))?;

        if !response.status().is_success() {
            return Err(AuthError::discovery(format!(
                "Discovery endpoint returned status {}",
                response.status()
            )));
        }

        let provider_config: OidcProviderConfig = response
            .json()
            .await
            .map_err(|e| AuthError::discovery(format!("Failed to parse discovery document: {}", e)))?;

        // Issuer の検証
        if provider_config.issuer != self.config.issuer {
            warn!(
                expected = %self.config.issuer,
                actual = %provider_config.issuer,
                "Issuer mismatch in discovery document"
            );
            return Err(AuthError::discovery(format!(
                "Issuer mismatch: expected {}, got {}",
                self.config.issuer, provider_config.issuer
            )));
        }

        // キャッシュに保存
        {
            let mut cache = self.cache.write().await;
            *cache = Some(CachedProvider {
                config: provider_config.clone(),
                fetched_at: Instant::now(),
            });
        }

        info!(issuer = %self.config.issuer, "OIDC provider configuration cached");
        Ok(provider_config)
    }

    /// JWKS URI を取得
    pub async fn get_jwks_uri(&self) -> Result<String, AuthError> {
        let config = self.discover().await?;
        Ok(config.jwks_uri)
    }

    /// Token エンドポイントを取得
    pub async fn get_token_endpoint(&self) -> Result<String, AuthError> {
        let config = self.discover().await?;
        config.token_endpoint.ok_or_else(|| {
            AuthError::discovery("Token endpoint not available in discovery document")
        })
    }

    /// UserInfo エンドポイントを取得
    pub async fn get_userinfo_endpoint(&self) -> Result<String, AuthError> {
        let config = self.discover().await?;
        config.userinfo_endpoint.ok_or_else(|| {
            AuthError::discovery("UserInfo endpoint not available in discovery document")
        })
    }

    /// Revocation エンドポイントを取得
    pub async fn get_revocation_endpoint(&self) -> Result<String, AuthError> {
        let config = self.discover().await?;
        config.revocation_endpoint.ok_or_else(|| {
            AuthError::discovery("Revocation endpoint not available in discovery document")
        })
    }
}

/// OIDC Discovery と JWT Verifier の統合
pub struct OidcJwtVerifier {
    discovery: OidcDiscovery,
    verifier: Option<crate::jwt::JwtVerifier>,
    audience: Option<String>,
}

impl OidcJwtVerifier {
    /// 新しい OIDC JWT Verifier を作成
    pub fn new(issuer: impl Into<String>) -> Self {
        let issuer = issuer.into();
        let config = OidcConfig::new(&issuer);
        let discovery = OidcDiscovery::new(config);

        Self {
            discovery,
            verifier: None,
            audience: None,
        }
    }

    /// Audience を設定
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = Some(audience.into());
        self
    }

    /// Discovery 設定を設定
    pub fn with_discovery_config(mut self, config: OidcConfig) -> Self {
        self.discovery = OidcDiscovery::new(config);
        self
    }

    /// JWT Verifier を初期化
    pub async fn initialize(&mut self) -> Result<(), AuthError> {
        let provider = self.discovery.discover().await?;

        let mut config = crate::jwt::JwtVerifierConfig::new(&provider.issuer)
            .with_jwks_uri(&provider.jwks_uri);

        if let Some(ref audience) = self.audience {
            config = config.with_audience(audience);
        }

        self.verifier = Some(crate::jwt::JwtVerifier::new(config));
        Ok(())
    }

    /// JWT を検証
    pub async fn verify(&self, token: &str) -> Result<crate::jwt::Claims, AuthError> {
        let verifier = self.verifier.as_ref().ok_or_else(|| {
            AuthError::internal("JWT Verifier not initialized. Call initialize() first.")
        })?;

        verifier.verify(token).await
    }

    /// Discovery を再実行し、JWKS を更新
    pub async fn refresh(&mut self) -> Result<(), AuthError> {
        self.discovery.invalidate_cache().await;
        self.initialize().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oidc_config() {
        let config = OidcConfig::new("https://auth.example.com");
        assert_eq!(
            config.get_discovery_url(),
            "https://auth.example.com/.well-known/openid-configuration"
        );

        let config = OidcConfig::new("https://auth.example.com/");
        assert_eq!(
            config.get_discovery_url(),
            "https://auth.example.com/.well-known/openid-configuration"
        );

        let config = OidcConfig::new("https://auth.example.com")
            .with_discovery_url("https://custom.example.com/discovery");
        assert_eq!(
            config.get_discovery_url(),
            "https://custom.example.com/discovery"
        );
    }

    #[test]
    fn test_oidc_config_builder() {
        let config = OidcConfig::new("https://auth.example.com")
            .with_cache_ttl_secs(7200)
            .with_http_timeout_ms(5000);

        assert_eq!(config.cache_ttl_secs, 7200);
        assert_eq!(config.http_timeout_ms, 5000);
    }

    #[test]
    fn test_parse_provider_config() {
        let json = r#"{
            "issuer": "https://auth.example.com",
            "jwks_uri": "https://auth.example.com/.well-known/jwks.json",
            "authorization_endpoint": "https://auth.example.com/authorize",
            "token_endpoint": "https://auth.example.com/token",
            "scopes_supported": ["openid", "profile", "email"],
            "response_types_supported": ["code"],
            "grant_types_supported": ["authorization_code", "refresh_token"]
        }"#;

        let config: OidcProviderConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.issuer, "https://auth.example.com");
        assert_eq!(config.jwks_uri, "https://auth.example.com/.well-known/jwks.json");
        assert_eq!(config.authorization_endpoint, Some("https://auth.example.com/authorize".to_string()));
        assert_eq!(config.token_endpoint, Some("https://auth.example.com/token".to_string()));
        assert!(config.scopes_supported.contains(&"openid".to_string()));
    }
}
