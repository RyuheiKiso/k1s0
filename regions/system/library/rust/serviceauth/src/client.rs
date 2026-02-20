//! サービス間認証クライアントのトレイトと HTTP 実装。

use crate::config::ServiceAuthConfig;
use crate::error::ServiceAuthError;
use crate::token::{ServiceToken, SpiffeId};
use async_trait::async_trait;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error};

#[cfg(feature = "mock")]
use mockall::automock;

/// ServiceClaims はサービストークンの JWT Claims を表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceClaims {
    /// サービスアカウントの識別子（client_id と同一のケースが多い）。
    pub sub: String,

    /// Keycloak の client_id（オプション）。
    pub client_id: Option<String>,

    /// 付与されたスコープ（オプション）。
    pub scope: Option<String>,

    /// トークン発行者 URL。
    pub iss: String,

    /// 有効期限（Unix タイムスタンプ）。
    pub exp: i64,

    /// 発行時刻（Unix タイムスタンプ）。
    pub iat: i64,
}

/// ServiceAuthClient は Client Credentials フローによるサービス間認証を提供するトレイト。
///
/// `HttpServiceAuthClient` がデフォルト実装。テスト時は `MockServiceAuthClient` が使用可能。
#[async_trait]
#[cfg_attr(feature = "mock", automock)]
pub trait ServiceAuthClient: Send + Sync {
    /// クライアントクレデンシャルフローでトークンを取得する。
    ///
    /// 毎回トークンエンドポイントに問い合わせる。
    /// キャッシュを使う場合は `get_cached_token` を使用すること。
    async fn get_token(&self) -> Result<ServiceToken, ServiceAuthError>;

    /// キャッシュ済みトークンを取得する（期限切れなら自動更新）。
    ///
    /// リフレッシュ閾値（`config.refresh_before_secs`）以内に有効期限が迫っている場合は
    /// 自動的に `get_token` を呼び出してキャッシュを更新する。
    async fn get_cached_token(&self) -> Result<String, ServiceAuthError>;

    /// サービストークンを検証して Claims を返す。
    ///
    /// JWKS URI が設定されていない場合はエラーを返す。
    async fn verify_token(&self, token: &str) -> Result<ServiceClaims, ServiceAuthError>;

    /// SPIFFE ID が指定ネームスペースの信頼済みサービスか検証する。
    ///
    /// SPIFFE URI を解析し、期待するネームスペースと一致するかを確認する。
    fn validate_spiffe_id(
        &self,
        spiffe_id: &str,
        expected_namespace: &str,
    ) -> Result<SpiffeId, ServiceAuthError>;
}

/// トークンエンドポイントのレスポンス（OAuth2 標準形式）。
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

/// HttpServiceAuthClient は reqwest を使った ServiceAuthClient の HTTP 実装。
///
/// トークンをメモリ内にキャッシュし、有効期限前に自動リフレッシュする。
pub struct HttpServiceAuthClient {
    config: ServiceAuthConfig,
    http_client: reqwest::Client,
    token_cache: Arc<RwLock<Option<ServiceToken>>>,
}

impl HttpServiceAuthClient {
    /// 新しい HttpServiceAuthClient を生成する。
    ///
    /// `config.timeout_secs` で指定したタイムアウトを持つ HTTP クライアントを内部で生成する。
    pub fn new(config: ServiceAuthConfig) -> Result<Self, ServiceAuthError> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| ServiceAuthError::Http(e.to_string()))?;

        Ok(Self {
            config,
            http_client,
            token_cache: Arc::new(RwLock::new(None)),
        })
    }
}

#[async_trait]
impl ServiceAuthClient for HttpServiceAuthClient {
    async fn get_token(&self) -> Result<ServiceToken, ServiceAuthError> {
        debug!(
            client_id = %self.config.client_id,
            token_endpoint = %self.config.token_endpoint,
            "Client Credentials フローでトークンを取得します"
        );

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        let response = self
            .http_client
            .post(&self.config.token_endpoint)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "トークンエンドポイントへの HTTP リクエストに失敗しました");
                ServiceAuthError::Http(e.to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!(
                status = %status,
                body = %body,
                "トークン取得に失敗しました"
            );
            return Err(ServiceAuthError::TokenAcquisition(format!(
                "HTTP {} - {}",
                status, body
            )));
        }

        let token_resp: TokenResponse = response.json().await.map_err(|e| {
            error!(error = %e, "トークンレスポンスの解析に失敗しました");
            ServiceAuthError::TokenAcquisition(e.to_string())
        })?;

        debug!(
            client_id = %self.config.client_id,
            expires_in = token_resp.expires_in,
            "トークンを取得しました"
        );

        Ok(ServiceToken::new(
            token_resp.access_token,
            token_resp.token_type,
            token_resp.expires_in,
        ))
    }

    async fn get_cached_token(&self) -> Result<String, ServiceAuthError> {
        // まず Read ロックでキャッシュを確認する
        {
            let cache = self.token_cache.read().await;
            if let Some(ref token) = *cache {
                if !token.should_refresh(self.config.refresh_before_secs) {
                    debug!("キャッシュ済みトークンを返します");
                    return Ok(token.bearer_header());
                }
            }
        }

        // キャッシュが存在しないかリフレッシュが必要なので Write ロックを取得する
        let mut cache = self.token_cache.write().await;

        // ダブルチェック: 別スレッドがすでにリフレッシュを完了しているかもしれない
        if let Some(ref token) = *cache {
            if !token.should_refresh(self.config.refresh_before_secs) {
                debug!("ダブルチェック: キャッシュ済みトークンを返します");
                return Ok(token.bearer_header());
            }
        }

        debug!("トークンをリフレッシュします");
        let new_token = self.get_token().await?;
        let bearer = new_token.bearer_header();
        *cache = Some(new_token);

        Ok(bearer)
    }

    async fn verify_token(&self, token: &str) -> Result<ServiceClaims, ServiceAuthError> {
        let jwks_uri = self.config.jwks_uri.as_deref().ok_or_else(|| {
            ServiceAuthError::InvalidToken("JWKS URI が設定されていません".to_string())
        })?;

        debug!(jwks_uri = %jwks_uri, "JWKS からトークンを検証します");

        // JWKS エンドポイントから公開鍵を取得する
        let jwks_resp: serde_json::Value = self
            .http_client
            .get(jwks_uri)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "JWKS エンドポイントへのアクセスに失敗しました");
                ServiceAuthError::Http(e.to_string())
            })?
            .json()
            .await
            .map_err(|e| {
                error!(error = %e, "JWKS レスポンスの解析に失敗しました");
                ServiceAuthError::InvalidToken(e.to_string())
            })?;

        let header = jsonwebtoken::decode_header(token).map_err(|e| {
            error!(error = %e, "JWT ヘッダーの解析に失敗しました");
            ServiceAuthError::InvalidToken(e.to_string())
        })?;

        let kid = header.kid.ok_or_else(|| {
            ServiceAuthError::InvalidToken("JWT ヘッダーに kid が含まれていません".to_string())
        })?;

        // 対応する鍵を JWKS から検索する
        let keys = jwks_resp["keys"]
            .as_array()
            .ok_or_else(|| ServiceAuthError::InvalidToken("JWKS に keys フィールドがありません".to_string()))?;

        let jwk = keys
            .iter()
            .find(|k| k["kid"].as_str() == Some(&kid))
            .ok_or_else(|| {
                ServiceAuthError::InvalidToken(format!("JWKS に kid '{}' が見つかりません", kid))
            })?;

        let n = jwk["n"].as_str().ok_or_else(|| {
            ServiceAuthError::InvalidToken("JWK に n フィールドがありません".to_string())
        })?;
        let e = jwk["e"].as_str().ok_or_else(|| {
            ServiceAuthError::InvalidToken("JWK に e フィールドがありません".to_string())
        })?;

        let decoding_key = DecodingKey::from_rsa_components(n, e).map_err(|e| {
            ServiceAuthError::InvalidToken(format!("RSA 公開鍵の構築に失敗しました: {}", e))
        })?;

        let mut validation = Validation::new(Algorithm::RS256);
        // サービストークンはオーディエンス検証を緩める（issuer のみ確認）
        validation.validate_aud = false;

        let token_data = decode::<ServiceClaims>(token, &decoding_key, &validation).map_err(
            |e| {
                error!(error = %e, "JWT 検証に失敗しました");
                ServiceAuthError::InvalidToken(e.to_string())
            },
        )?;

        debug!(sub = %token_data.claims.sub, "トークン検証に成功しました");

        Ok(token_data.claims)
    }

    fn validate_spiffe_id(
        &self,
        spiffe_id: &str,
        expected_namespace: &str,
    ) -> Result<SpiffeId, ServiceAuthError> {
        debug!(
            spiffe_id = %spiffe_id,
            expected_namespace = %expected_namespace,
            "SPIFFE ID を検証します"
        );

        let parsed = SpiffeId::parse(spiffe_id)?;

        if parsed.namespace != expected_namespace {
            error!(
                namespace = %parsed.namespace,
                expected = %expected_namespace,
                "SPIFFE ID のネームスペースが一致しません"
            );
            return Err(ServiceAuthError::SpiffeValidationFailed(format!(
                "ネームスペースが一致しません: 期待='{}', 実際='{}'",
                expected_namespace, parsed.namespace
            )));
        }

        debug!(
            trust_domain = %parsed.trust_domain,
            namespace = %parsed.namespace,
            service_account = %parsed.service_account,
            "SPIFFE ID 検証に成功しました"
        );

        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_client(token_endpoint: &str) -> HttpServiceAuthClient {
        let config = ServiceAuthConfig::new(token_endpoint, "test-svc", "test-secret");
        HttpServiceAuthClient::new(config).unwrap()
    }

    // --- validate_spiffe_id テスト ---

    #[test]
    fn test_validate_spiffe_id_success() {
        let client = make_client("https://auth.example.com/token");
        let result = client
            .validate_spiffe_id("spiffe://k1s0.internal/ns/system/sa/auth-service", "system");
        assert!(result.is_ok());
        let spiffe = result.unwrap();
        assert_eq!(spiffe.namespace, "system");
        assert_eq!(spiffe.service_account, "auth-service");
    }

    #[test]
    fn test_validate_spiffe_id_namespace_mismatch() {
        let client = make_client("https://auth.example.com/token");
        let result = client.validate_spiffe_id(
            "spiffe://k1s0.internal/ns/system/sa/auth-service",
            "business",
        );
        assert!(matches!(
            result,
            Err(ServiceAuthError::SpiffeValidationFailed(_))
        ));
        if let Err(ServiceAuthError::SpiffeValidationFailed(msg)) = result {
            assert!(msg.contains("business"));
            assert!(msg.contains("system"));
        }
    }

    #[test]
    fn test_validate_spiffe_id_invalid_uri() {
        let client = make_client("https://auth.example.com/token");
        let result = client.validate_spiffe_id("not-a-spiffe-id", "system");
        assert!(matches!(
            result,
            Err(ServiceAuthError::SpiffeValidationFailed(_))
        ));
    }

    #[test]
    fn test_validate_spiffe_id_business_namespace() {
        let client = make_client("https://auth.example.com/token");
        let result = client.validate_spiffe_id(
            "spiffe://k1s0.internal/ns/business/sa/order-service",
            "business",
        );
        assert!(result.is_ok());
    }

    // --- HttpServiceAuthClient 生成テスト ---

    #[test]
    fn test_new_client_success() {
        let config = ServiceAuthConfig::new(
            "https://auth.example.com/token",
            "test-service",
            "test-secret",
        );
        let client = HttpServiceAuthClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_new_client_with_jwks_uri() {
        let config = ServiceAuthConfig::new(
            "https://auth.example.com/token",
            "test-service",
            "test-secret",
        )
        .with_jwks_uri("https://auth.example.com/certs");
        let client = HttpServiceAuthClient::new(config);
        assert!(client.is_ok());
        let c = client.unwrap();
        assert!(c.config.jwks_uri.is_some());
    }

    // --- verify_token エラーケース（JWKS URI なし）---

    #[tokio::test]
    async fn test_verify_token_no_jwks_uri() {
        let config = ServiceAuthConfig::new(
            "https://auth.example.com/token",
            "test-service",
            "test-secret",
        );
        let client = HttpServiceAuthClient::new(config).unwrap();
        let result = client.verify_token("dummy-token").await;
        assert!(matches!(result, Err(ServiceAuthError::InvalidToken(_))));
    }
}
