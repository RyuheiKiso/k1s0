use std::sync::Arc;
use std::time::{Duration, Instant};

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::RwLock;
use tracing::{debug, instrument};

use crate::adapter::middleware::auth_middleware::Claims;

/// JwksVerifier は JWKS エンドポイントから公開鍵を取得し、JWT の署名を検証する。
/// 公開鍵は内部にキャッシュし、TTL 経過後に再取得する。
/// issuer/audience が設定されている場合は JWT のクレームを検証し、不一致時はエラーを返す。
pub struct JwksVerifier {
    jwks_url: String,
    http_client: Client,
    cache: Arc<RwLock<Option<CachedJwks>>>,
    cache_ttl: Duration,
    // JWT issuer 検証用（None の場合は検証をスキップ）
    issuer: Option<String>,
    // JWT audience 検証用（None の場合は検証をスキップ）
    audience: Option<String>,
}

struct CachedJwks {
    keys: Vec<Jwk>,
    fetched_at: Instant,
}

#[derive(Debug, Clone, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

#[derive(Debug, Clone, Deserialize)]
struct Jwk {
    kid: Option<String>,
    kty: String,
    #[allow(dead_code)]
    alg: Option<String>,
    n: Option<String>,
    e: Option<String>,
}

impl JwksVerifier {
    /// 新しい JwksVerifier を生成する。
    /// issuer/audience は AuthConfig から取得し、設定値がある場合のみ JWT クレームを検証する。
    /// TLS バックエンドの初期化に失敗した場合は Err を返す。
    pub fn new(jwks_url: String) -> anyhow::Result<Self> {
        // HTTPクライアントを構築する（タイムアウト10秒）
        // TLS バックエンドが利用不可の場合はエラーとして伝播する
        let http_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| anyhow::anyhow!("HTTPクライアントの構築に失敗: {}", e))?;
        Ok(Self {
            jwks_url,
            http_client,
            cache: Arc::new(RwLock::new(None)),
            cache_ttl: Duration::from_secs(600), // 10分
            issuer: None,
            audience: None,
        })
    }

    /// issuer/audience 検証を設定する。
    /// AuthConfig の値を渡すことで JWT クレームの厳密な検証が有効になる。
    pub fn with_issuer_audience(
        mut self,
        issuer: Option<String>,
        audience: Option<String>,
    ) -> Self {
        self.issuer = issuer;
        self.audience = audience;
        self
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn verify_token(&self, token: &str) -> anyhow::Result<Claims> {
        let keys = self.get_jwks().await?;

        let header =
            decode_header(token).map_err(|e| anyhow::anyhow!("invalid JWT header: {}", e))?;

        // kid でマッチする鍵を選択。kid が無い場合は最初の RSA 鍵を使用
        let jwk = match &header.kid {
            Some(kid) => keys.iter().find(|k| k.kid.as_deref() == Some(kid.as_str())),
            None => keys.iter().find(|k| k.kty == "RSA"),
        }
        .ok_or_else(|| anyhow::anyhow!("no matching JWK found"))?;

        let n = jwk
            .n
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("JWK missing 'n'"))?;
        let e = jwk
            .e
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("JWK missing 'e'"))?;

        let decoding_key = DecodingKey::from_rsa_components(n, e)
            .map_err(|e| anyhow::anyhow!("invalid RSA key: {}", e))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;

        // issuer が設定されている場合は JWT の iss クレームを検証する。
        // None のままにすれば jsonwebtoken は iss を検証しない（後方互換性）。
        if let Some(ref iss) = self.issuer {
            validation.set_issuer(&[iss]);
        }

        // audience が設定されている場合は JWT の aud クレームを検証する。
        // None の場合は validate_aud を false にしてスキップする（後方互換性）。
        if let Some(ref aud) = self.audience {
            validation.set_audience(&[aud]);
        } else {
            // audience 未設定時は検証をスキップする（後方互換性）
            validation.validate_aud = false;
        }

        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| anyhow::anyhow!("JWT verification failed: {}", e))?;

        Ok(token_data.claims)
    }

    async fn get_jwks(&self) -> anyhow::Result<Vec<Jwk>> {
        // キャッシュが有効であれば返す
        {
            let cache = self.cache.read().await;
            if let Some(ref c) = *cache {
                if c.fetched_at.elapsed() < self.cache_ttl {
                    debug!("JWKS cache hit");
                    return Ok(c.keys.clone());
                }
            }
        }

        // キャッシュ期限切れ: 再取得
        debug!("fetching JWKS from {}", self.jwks_url);
        let resp: JwksResponse = self
            .http_client
            .get(&self.jwks_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let mut cache = self.cache.write().await;
        *cache = Some(CachedJwks {
            keys: resp.keys.clone(),
            fetched_at: Instant::now(),
        });

        Ok(resp.keys)
    }
}
