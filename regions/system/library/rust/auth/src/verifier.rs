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
/// タイムアウト付きの reqwest::Client を保持し、JWKS エンドポイントへの接続遅延を防ぐ。
pub struct DefaultJwksFetcher {
    /// タイムアウト設定済みの HTTP クライアント
    client: reqwest::Client,
}

impl DefaultJwksFetcher {
    /// 新しい DefaultJwksFetcher を生成する。
    /// HTTP クライアントにタイムアウトを設定し、JWKS 取得時の無限待ちを防止する。
    pub fn new() -> Result<Self, AuthError> {
        // 全体タイムアウト: 10秒。JWKS レスポンスは通常小さいため、
        // 10秒以内に完了しない場合はネットワーク障害と判断する。
        // 接続タイムアウト: 5秒。DNS 解決やTCPハンドシェイクが
        // 5秒以上かかる場合はエンドポイント到達不能と判断する。
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| AuthError::JwksFetchFailed(e.to_string()))?;

        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl JwksFetcher for DefaultJwksFetcher {
    /// JWKS エンドポイントから公開鍵一覧を取得する。
    /// タイムアウト付きクライアントを使用して HTTP リクエストを送信する。
    async fn fetch_keys(&self, jwks_url: &str) -> Result<Vec<JwkKey>, AuthError> {
        let resp: JwksResponse = self
            .client
            .get(jwks_url)
            .send()
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

/// JWKS キャッシュ。Vec<JwkKey> を Arc で包み、クローン時のコピーコストを削減する。
struct JwksCache {
    keys: Arc<Vec<JwkKey>>,
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
    /// DefaultJwksFetcher の HTTP クライアント構築に失敗した場合はエラーを返す。
    pub fn new(
        jwks_url: &str,
        issuer: &str,
        audience: &str,
        cache_ttl: Duration,
    ) -> Result<Self, AuthError> {
        Ok(Self {
            jwks_url: jwks_url.to_string(),
            issuer: issuer.to_string(),
            audience: audience.to_string(),
            cache_ttl,
            cache: Arc::new(RwLock::new(None)),
            fetcher: Arc::new(DefaultJwksFetcher::new()?),
        })
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
    /// 返り値を Arc<Vec<JwkKey>> にしてクローン時の Vec コピーコストを排除する。
    async fn get_keys(&self) -> Result<Arc<Vec<JwkKey>>, AuthError> {
        // Read lock でキャッシュを確認
        {
            let cache = self.cache.read().await;
            if let Some(ref c) = *cache {
                if c.fetched_at.elapsed() < self.cache_ttl {
                    // Arc クローンは参照カウントのインクリメントのみ
                    return Ok(Arc::clone(&c.keys));
                }
            }
        }

        // Write lock で再取得
        let mut cache = self.cache.write().await;

        // ダブルチェック: 他のスレッドが既に更新済みの場合はキャッシュを返す
        if let Some(ref c) = *cache {
            if c.fetched_at.elapsed() < self.cache_ttl {
                return Ok(Arc::clone(&c.keys));
            }
        }

        match self.fetcher.fetch_keys(&self.jwks_url).await {
            Ok(keys) => {
                // Arc で包んでキャッシュに格納する
                let keys_arc = Arc::new(keys);
                *cache = Some(JwksCache {
                    keys: Arc::clone(&keys_arc),
                    fetched_at: Instant::now(),
                });
                Ok(keys_arc)
            }
            Err(err) => {
                // fetch 失敗時、stale キャッシュがあればそれを返す（Go 実装と同等）
                if let Some(ref c) = *cache {
                    return Ok(Arc::clone(&c.keys));
                }
                Err(err)
            }
        }
    }

    /// キャッシュを無効化する。鍵ローテーション時に使用。
    pub async fn invalidate_cache(&self) {
        let mut cache = self.cache.write().await;
        *cache = None;
    }
}
