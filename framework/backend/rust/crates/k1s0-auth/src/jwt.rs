//! JWT検証モジュール
//!
//! JWT/OIDCトークンの検証、公開鍵ローテーション対応

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use jsonwebtoken::{
    decode, decode_header, Algorithm, DecodingKey, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::error::AuthError;

/// JWTクレーム
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// サブジェクト（ユーザーID）
    pub sub: String,
    /// 発行者
    pub iss: String,
    /// 対象者（audience）
    pub aud: Option<AudienceClaim>,
    /// 有効期限（Unix timestamp）
    pub exp: i64,
    /// 発行日時（Unix timestamp）
    pub iat: i64,
    /// Not Before（Unix timestamp）
    #[serde(default)]
    pub nbf: Option<i64>,
    /// JWT ID
    #[serde(default)]
    pub jti: Option<String>,
    /// ロール（カスタムクレーム）
    #[serde(default)]
    pub roles: Vec<String>,
    /// パーミッション（カスタムクレーム）
    #[serde(default)]
    pub permissions: Vec<String>,
    /// テナントID（カスタムクレーム）
    #[serde(default)]
    pub tenant_id: Option<String>,
    /// メールアドレス（OIDCクレーム）
    #[serde(default)]
    pub email: Option<String>,
    /// メール確認済み（OIDCクレーム）
    #[serde(default)]
    pub email_verified: Option<bool>,
    /// 名前（OIDCクレーム）
    #[serde(default)]
    pub name: Option<String>,
}

/// Audience claim（単一または複数）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AudienceClaim {
    /// 単一
    Single(String),
    /// 複数
    Multiple(Vec<String>),
}

impl AudienceClaim {
    /// 指定されたaudienceを含むかチェック
    pub fn contains(&self, target: &str) -> bool {
        match self {
            Self::Single(s) => s == target,
            Self::Multiple(v) => v.iter().any(|s| s == target),
        }
    }
}

/// JWKS（JSON Web Key Set）
#[derive(Debug, Clone, Deserialize)]
pub struct Jwks {
    /// 鍵の配列
    pub keys: Vec<Jwk>,
}

/// JWK（JSON Web Key）
#[derive(Debug, Clone, Deserialize)]
pub struct Jwk {
    /// 鍵タイプ
    pub kty: String,
    /// 使用目的
    #[serde(rename = "use")]
    pub use_: Option<String>,
    /// 鍵ID
    pub kid: Option<String>,
    /// アルゴリズム
    pub alg: Option<String>,
    /// RSA modulus（Base64URL）
    pub n: Option<String>,
    /// RSA exponent（Base64URL）
    pub e: Option<String>,
    /// EC curve
    pub crv: Option<String>,
    /// EC x coordinate
    pub x: Option<String>,
    /// EC y coordinate
    pub y: Option<String>,
}

impl Jwk {
    /// DecodingKeyに変換
    pub fn to_decoding_key(&self) -> Result<DecodingKey, AuthError> {
        match self.kty.as_str() {
            "RSA" => {
                let n = self.n.as_ref().ok_or_else(|| {
                    AuthError::InvalidKey("RSA key missing 'n' component".to_string())
                })?;
                let e = self.e.as_ref().ok_or_else(|| {
                    AuthError::InvalidKey("RSA key missing 'e' component".to_string())
                })?;
                DecodingKey::from_rsa_components(n, e).map_err(|e| {
                    AuthError::InvalidKey(format!("Failed to create RSA key: {}", e))
                })
            }
            "EC" => {
                let x = self.x.as_ref().ok_or_else(|| {
                    AuthError::InvalidKey("EC key missing 'x' component".to_string())
                })?;
                let y = self.y.as_ref().ok_or_else(|| {
                    AuthError::InvalidKey("EC key missing 'y' component".to_string())
                })?;
                DecodingKey::from_ec_components(x, y).map_err(|e| {
                    AuthError::InvalidKey(format!("Failed to create EC key: {}", e))
                })
            }
            other => Err(AuthError::InvalidKey(format!(
                "Unsupported key type: {}",
                other
            ))),
        }
    }

    /// アルゴリズムを取得
    pub fn algorithm(&self) -> Option<Algorithm> {
        self.alg.as_ref().and_then(|alg| match alg.as_str() {
            "RS256" => Some(Algorithm::RS256),
            "RS384" => Some(Algorithm::RS384),
            "RS512" => Some(Algorithm::RS512),
            "ES256" => Some(Algorithm::ES256),
            "ES384" => Some(Algorithm::ES384),
            "PS256" => Some(Algorithm::PS256),
            "PS384" => Some(Algorithm::PS384),
            "PS512" => Some(Algorithm::PS512),
            _ => None,
        })
    }
}

/// キャッシュされた鍵
struct CachedKey {
    key: DecodingKey,
    algorithm: Algorithm,
    fetched_at: Instant,
}

/// JWT検証設定
#[derive(Debug, Clone)]
pub struct JwtVerifierConfig {
    /// JWKS URI
    pub jwks_uri: Option<String>,
    /// 発行者（issuer）
    pub issuer: String,
    /// 対象者（audience）
    pub audience: Option<String>,
    /// 許可するアルゴリズム
    pub algorithms: Vec<Algorithm>,
    /// 鍵のリフレッシュ間隔
    pub key_refresh_interval: Duration,
    /// 静的な秘密鍵（開発用）
    pub static_secret: Option<String>,
    /// クロック許容誤差（秒）
    pub leeway: u64,
}

impl Default for JwtVerifierConfig {
    fn default() -> Self {
        Self {
            jwks_uri: None,
            issuer: String::new(),
            audience: None,
            algorithms: vec![Algorithm::RS256],
            key_refresh_interval: Duration::from_secs(3600),
            static_secret: None,
            leeway: 60,
        }
    }
}

impl JwtVerifierConfig {
    /// 新しい設定を作成
    pub fn new(issuer: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            ..Default::default()
        }
    }

    /// JWKS URIを設定
    pub fn with_jwks_uri(mut self, uri: impl Into<String>) -> Self {
        self.jwks_uri = Some(uri.into());
        self
    }

    /// Audienceを設定
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = Some(audience.into());
        self
    }

    /// 静的シークレットを設定（開発用）
    pub fn with_static_secret(mut self, secret: impl Into<String>) -> Self {
        self.static_secret = Some(secret.into());
        self
    }

    /// アルゴリズムを設定
    pub fn with_algorithms(mut self, algorithms: Vec<Algorithm>) -> Self {
        self.algorithms = algorithms;
        self
    }

    /// リフレッシュ間隔を設定
    pub fn with_key_refresh_interval(mut self, interval: Duration) -> Self {
        self.key_refresh_interval = interval;
        self
    }
}

/// JWT検証器
pub struct JwtVerifier {
    config: JwtVerifierConfig,
    /// kid -> CachedKey
    key_cache: Arc<RwLock<HashMap<String, CachedKey>>>,
    /// HTTPクライアント
    http_client: reqwest::Client,
}

impl JwtVerifier {
    /// 新しい検証器を作成
    pub fn new(config: JwtVerifierConfig) -> Self {
        Self {
            config,
            key_cache: Arc::new(RwLock::new(HashMap::new())),
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// トークンを検証してクレームを取得
    pub async fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        // ヘッダーを解析
        let header = decode_header(token).map_err(|e| {
            AuthError::InvalidToken(format!("Failed to decode header: {}", e))
        })?;

        // 検証設定を作成
        let mut validation = Validation::new(header.alg);
        validation.set_issuer(&[&self.config.issuer]);
        validation.leeway = self.config.leeway;

        if let Some(ref aud) = self.config.audience {
            validation.set_audience(&[aud]);
        } else {
            validation.validate_aud = false;
        }

        // 鍵を取得
        let decoding_key = self.get_decoding_key(&header.kid, header.alg).await?;

        // トークンを検証
        let token_data: TokenData<Claims> = decode(token, &decoding_key, &validation)
            .map_err(|e| AuthError::InvalidToken(format!("Token validation failed: {}", e)))?;

        debug!(
            sub = %token_data.claims.sub,
            iss = %token_data.claims.iss,
            "JWT verified successfully"
        );

        Ok(token_data.claims)
    }

    /// 検証鍵を取得
    async fn get_decoding_key(
        &self,
        kid: &Option<String>,
        alg: Algorithm,
    ) -> Result<DecodingKey, AuthError> {
        // 静的シークレットがあれば使用
        if let Some(ref secret) = self.config.static_secret {
            return Ok(DecodingKey::from_secret(secret.as_bytes()));
        }

        // JWKSから取得
        let jwks_uri = self.config.jwks_uri.as_ref().ok_or_else(|| {
            AuthError::Configuration("JWKS URI not configured".to_string())
        })?;

        // キャッシュをチェック
        if let Some(kid) = kid {
            let cache = self.key_cache.read().await;
            if let Some(cached) = cache.get(kid) {
                if cached.fetched_at.elapsed() < self.config.key_refresh_interval {
                    return Ok(cached.key.clone());
                }
            }
        }

        // JWKSを取得
        let jwks = self.fetch_jwks(jwks_uri).await?;

        // キャッシュを更新
        let mut cache = self.key_cache.write().await;
        for jwk in &jwks.keys {
            if let Some(ref jwk_kid) = jwk.kid {
                if let Ok(key) = jwk.to_decoding_key() {
                    let algorithm = jwk.algorithm().unwrap_or(Algorithm::RS256);
                    cache.insert(
                        jwk_kid.clone(),
                        CachedKey {
                            key,
                            algorithm,
                            fetched_at: Instant::now(),
                        },
                    );
                }
            }
        }

        // 要求された鍵を返す
        if let Some(kid) = kid {
            cache
                .get(kid)
                .map(|c| c.key.clone())
                .ok_or_else(|| AuthError::InvalidKey(format!("Key not found: {}", kid)))
        } else {
            // kidがない場合は最初の一致する鍵を使用
            jwks.keys
                .iter()
                .find(|k| k.algorithm().map(|a| a == alg).unwrap_or(true))
                .and_then(|k| k.to_decoding_key().ok())
                .ok_or_else(|| AuthError::InvalidKey("No suitable key found".to_string()))
        }
    }

    /// JWKSを取得
    async fn fetch_jwks(&self, uri: &str) -> Result<Jwks, AuthError> {
        info!(uri = %uri, "Fetching JWKS");

        let response = self
            .http_client
            .get(uri)
            .send()
            .await
            .map_err(|e| AuthError::NetworkError(format!("Failed to fetch JWKS: {}", e)))?;

        if !response.status().is_success() {
            return Err(AuthError::NetworkError(format!(
                "JWKS fetch failed with status: {}",
                response.status()
            )));
        }

        let jwks: Jwks = response
            .json()
            .await
            .map_err(|e| AuthError::InvalidKey(format!("Failed to parse JWKS: {}", e)))?;

        info!(key_count = jwks.keys.len(), "JWKS fetched successfully");

        Ok(jwks)
    }

    /// キャッシュをクリア
    pub async fn clear_cache(&self) {
        let mut cache = self.key_cache.write().await;
        cache.clear();
        info!("JWT key cache cleared");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audience_claim_single() {
        let aud = AudienceClaim::Single("my-api".to_string());
        assert!(aud.contains("my-api"));
        assert!(!aud.contains("other-api"));
    }

    #[test]
    fn test_audience_claim_multiple() {
        let aud = AudienceClaim::Multiple(vec!["api1".to_string(), "api2".to_string()]);
        assert!(aud.contains("api1"));
        assert!(aud.contains("api2"));
        assert!(!aud.contains("api3"));
    }

    #[test]
    fn test_config_builder() {
        let config = JwtVerifierConfig::new("https://auth.example.com")
            .with_jwks_uri("https://auth.example.com/.well-known/jwks.json")
            .with_audience("my-api")
            .with_algorithms(vec![Algorithm::RS256, Algorithm::RS384]);

        assert_eq!(config.issuer, "https://auth.example.com");
        assert_eq!(
            config.jwks_uri,
            Some("https://auth.example.com/.well-known/jwks.json".to_string())
        );
        assert_eq!(config.audience, Some("my-api".to_string()));
        assert_eq!(config.algorithms.len(), 2);
    }

    #[tokio::test]
    async fn test_verify_with_static_secret() {
        use jsonwebtoken::{encode, EncodingKey, Header};

        let secret = "test-secret-key-for-testing-only";
        let config = JwtVerifierConfig::new("test-issuer")
            .with_static_secret(secret)
            .with_algorithms(vec![Algorithm::HS256]);

        let verifier = JwtVerifier::new(config);

        // テスト用のトークンを作成
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: "user123".to_string(),
            iss: "test-issuer".to_string(),
            aud: None,
            exp: now + 3600,
            iat: now,
            nbf: None,
            jti: None,
            roles: vec!["admin".to_string()],
            permissions: vec![],
            tenant_id: None,
            email: Some("user@example.com".to_string()),
            email_verified: Some(true),
            name: Some("Test User".to_string()),
        };

        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let result = verifier.verify(&token).await;
        assert!(result.is_ok());

        let verified_claims = result.unwrap();
        assert_eq!(verified_claims.sub, "user123");
        assert_eq!(verified_claims.roles, vec!["admin"]);
    }

    #[tokio::test]
    async fn test_verify_expired_token() {
        use jsonwebtoken::{encode, EncodingKey, Header};

        let secret = "test-secret";
        let config = JwtVerifierConfig::new("test-issuer")
            .with_static_secret(secret);

        let verifier = JwtVerifier::new(config);

        // 期限切れトークンを作成
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: "user123".to_string(),
            iss: "test-issuer".to_string(),
            aud: None,
            exp: now - 3600, // 1時間前に期限切れ
            iat: now - 7200,
            nbf: None,
            jti: None,
            roles: vec![],
            permissions: vec![],
            tenant_id: None,
            email: None,
            email_verified: None,
            name: None,
        };

        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let result = verifier.verify(&token).await;
        assert!(result.is_err());
    }
}
