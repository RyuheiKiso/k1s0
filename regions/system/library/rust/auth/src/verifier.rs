//! JWKS 検証器: HTTP で公開鍵を取得しキャッシュ、JWT トークンを検証する。

use crate::claims::Claims;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// AuthError は認証・認可エラーを表す。
#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("token expired")]
    TokenExpired,

    #[error("invalid token: {0}")]
    InvalidToken(String),

    #[error("JWKS fetch failed: {0}")]
    JwksFetchFailed(String),

    #[error("missing Authorization header")]
    MissingToken,

    #[error("invalid Authorization header format")]
    InvalidAuthHeader,

    #[error("permission denied")]
    PermissionDenied,

    #[error("tier access denied")]
    TierAccessDenied,
}

/// JWKS レスポンスの構造体。
#[derive(Debug, Clone, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

/// 個々の JWK 鍵。
#[derive(Debug, Clone, Deserialize)]
struct Jwk {
    kid: String,
    #[allow(dead_code)]
    kty: String,
    n: String,
    e: String,
}

/// JwksFetcher は JWKS エンドポイントからの鍵取得を抽象化するトレイト。
#[async_trait::async_trait]
pub trait JwksFetcher: Send + Sync {
    async fn fetch_keys(&self, jwks_url: &str) -> Result<Vec<JwkKey>, AuthError>;
}

/// JwkKey は取得した JWK 鍵の公開情報。
#[derive(Debug, Clone)]
pub struct JwkKey {
    pub kid: String,
    pub n: String,
    pub e: String,
}

/// DefaultJwksFetcher は HTTP 経由で JWKS を取得するデフォルト実装。
pub struct DefaultJwksFetcher;

#[async_trait::async_trait]
impl JwksFetcher for DefaultJwksFetcher {
    async fn fetch_keys(&self, jwks_url: &str) -> Result<Vec<JwkKey>, AuthError> {
        let resp: JwksResponse = reqwest::get(jwks_url)
            .await
            .map_err(|e| AuthError::JwksFetchFailed(e.to_string()))?
            .json()
            .await
            .map_err(|e| AuthError::JwksFetchFailed(e.to_string()))?;

        Ok(resp
            .keys
            .into_iter()
            .map(|k| JwkKey {
                kid: k.kid,
                n: k.n,
                e: k.e,
            })
            .collect())
    }
}

/// JWKS キャッシュ。
struct JwksCache {
    keys: Vec<JwkKey>,
    fetched_at: Instant,
}

/// JwksVerifier は JWKS エンドポイントから公開鍵を取得し、JWT トークンを検証する。
pub struct JwksVerifier {
    jwks_url: String,
    issuer: String,
    audience: String,
    cache_ttl: Duration,
    cache: Arc<RwLock<Option<JwksCache>>>,
    fetcher: Arc<dyn JwksFetcher>,
}

impl JwksVerifier {
    /// 新しい JwksVerifier を生成する。
    pub fn new(jwks_url: &str, issuer: &str, audience: &str, cache_ttl: Duration) -> Self {
        Self {
            jwks_url: jwks_url.to_string(),
            issuer: issuer.to_string(),
            audience: audience.to_string(),
            cache_ttl,
            cache: Arc::new(RwLock::new(None)),
            fetcher: Arc::new(DefaultJwksFetcher),
        }
    }

    /// カスタムフェッチャーを使う JwksVerifier を生成する（テスト用）。
    pub fn with_fetcher(
        jwks_url: &str,
        issuer: &str,
        audience: &str,
        cache_ttl: Duration,
        fetcher: Arc<dyn JwksFetcher>,
    ) -> Self {
        Self {
            jwks_url: jwks_url.to_string(),
            issuer: issuer.to_string(),
            audience: audience.to_string(),
            cache_ttl,
            cache: Arc::new(RwLock::new(None)),
            fetcher,
        }
    }

    /// JWT トークン文字列を検証し、Claims を返す。
    pub async fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let keys = self.get_keys().await?;

        let header = jsonwebtoken::decode_header(token)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        let kid = header
            .kid
            .ok_or_else(|| AuthError::InvalidToken("missing kid in header".into()))?;

        let jwk = keys
            .iter()
            .find(|k| k.kid == kid)
            .ok_or_else(|| AuthError::InvalidToken(format!("unknown kid: {}", kid)))?;

        let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);

        let data = decode::<Claims>(token, &key, &validation)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

        Ok(data.claims)
    }

    /// キャッシュから鍵を取得する。TTL を超えている場合は再取得する。
    async fn get_keys(&self) -> Result<Vec<JwkKey>, AuthError> {
        // Read lock でキャッシュを確認
        {
            let cache = self.cache.read().await;
            if let Some(ref c) = *cache {
                if c.fetched_at.elapsed() < self.cache_ttl {
                    return Ok(c.keys.clone());
                }
            }
        }

        // Write lock で再取得
        let mut cache = self.cache.write().await;

        // ダブルチェック
        if let Some(ref c) = *cache {
            if c.fetched_at.elapsed() < self.cache_ttl {
                return Ok(c.keys.clone());
            }
        }

        let keys = self.fetcher.fetch_keys(&self.jwks_url).await?;

        *cache = Some(JwksCache {
            keys: keys.clone(),
            fetched_at: Instant::now(),
        });

        Ok(keys)
    }

    /// キャッシュを無効化する。鍵ローテーション時に使用。
    pub async fn invalidate_cache(&self) {
        let mut cache = self.cache.write().await;
        *cache = None;
    }
}

