use std::sync::Arc;
use std::time::{Duration, Instant};

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::RwLock;
use tracing::{debug, instrument};

use crate::adapter::middleware::auth_middleware::Claims;

/// LOW-014 監査対応: JWT 検証エラーを種別ごとに区別する型付きエラー。
/// `auth_middleware.rs` がこの型でマッチし、クライアントに適切なエラーコードを返す。
/// - `TokenExpired`: 期限切れ → `SYS_AUTH_TOKEN_EXPIRED`
/// - `InvalidSignature`: 署名不正 → `SYS_AUTH_TOKEN_INVALID_SIGNATURE`
/// - InvalidIssuer/Audience: クレーム不一致 → `SYS_AUTH_TOKEN_CLAIMS_INVALID`
/// - `JwksFetchFailed`: JWKS 取得失敗 → `SYS_AUTH_JWKS_UNAVAILABLE`
/// - `MalformedToken`: その他の不正フォーマット → `SYS_AUTH_TOKEN_MALFORMED`
#[derive(Debug, thiserror::Error)]
pub enum JwtVerifyError {
    /// トークンの有効期限が切れている（`exp` クレームが現在時刻より過去）
    #[error("Token has expired")]
    TokenExpired,

    /// RSA 署名が JWKS の公開鍵と一致しない（改ざん・偽造の可能性）
    #[error("Invalid JWT signature")]
    InvalidSignature,

    /// `iss` クレームが設定値と不一致
    #[error("Invalid JWT issuer")]
    InvalidIssuer,

    /// `aud` クレームが設定値と不一致
    #[error("Invalid JWT audience")]
    InvalidAudience,

    /// JWKS エンドポイントへの接続・取得に失敗（認証サービスの一時障害等）
    #[error("JWKS fetch failed: {0}")]
    JwksFetchFailed(String),

    /// ヘッダー/クレームのデコード失敗・鍵不一致等、上記以外のトークン不正
    #[error("Malformed JWT: {0}")]
    MalformedToken(String),
}

/// `JwksVerifier` は JWKS エンドポイントから公開鍵を取得し、JWT の署名を検証する。
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
    /// 新しい `JwksVerifier` を生成する。
    /// issuer/audience は `AuthConfig` から取得し、設定値がある場合のみ JWT クレームを検証する。
    /// TLS バックエンドの初期化に失敗した場合は Err を返す。
    pub fn new(jwks_url: String) -> anyhow::Result<Self> {
        // HTTPクライアントを構築する（タイムアウト10秒）
        // TLS バックエンドが利用不可の場合はエラーとして伝播する
        let http_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| anyhow::anyhow!("HTTPクライアントの構築に失敗: {e}"))?;
        Ok(Self {
            jwks_url,
            http_client,
            cache: Arc::new(RwLock::new(None)),
            cache_ttl: Duration::from_secs(600), // 10分
            issuer: None,
            audience: None,
        })
    }

    /// JWKS キャッシュの TTL を設定する。
    /// config の `cache_ttl_secs` を渡すことで、設定ファイルの値をキャッシュ有効期限に反映する。
    /// デフォルト値は 600 秒（10 分）。
    #[must_use] 
    pub fn with_cache_ttl(mut self, ttl_secs: u64) -> Self {
        self.cache_ttl = Duration::from_secs(ttl_secs);
        self
    }

    /// issuer/audience 検証を設定する。
    /// `AuthConfig` の値を渡すことで JWT クレームの厳密な検証が有効になる。
    #[must_use] 
    pub fn with_issuer_audience(
        mut self,
        issuer: Option<String>,
        audience: Option<String>,
    ) -> Self {
        self.issuer = issuer;
        self.audience = audience;
        self
    }

    /// JWT を検証し、成功時にクレームを返す。失敗時は種別付きの `JwtVerifyError` を返す。
    ///
    /// LOW-014 監査対応: 旧実装は全エラーを `anyhow::Error` に統合し、
    /// 呼び出し元で種別判定不可能だった。本実装では `JwtVerifyError` で種別を明示し、
    /// ミドルウェアがクライアントに適切なエラーコードを返せるようにする。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn verify_token(&self, token: &str) -> Result<Claims, JwtVerifyError> {
        let keys = self.get_jwks().await?;

        let header = decode_header(token)
            .map_err(|e| JwtVerifyError::MalformedToken(format!("invalid JWT header: {e}")))?;

        // kid でマッチする鍵を選択。kid が無い場合は最初の RSA 鍵を使用
        let jwk = match &header.kid {
            Some(kid) => keys.iter().find(|k| k.kid.as_deref() == Some(kid.as_str())),
            None => keys.iter().find(|k| k.kty == "RSA"),
        }
        .ok_or_else(|| JwtVerifyError::MalformedToken("no matching JWK found".to_string()))?;

        let n = jwk
            .n
            .as_deref()
            .ok_or_else(|| JwtVerifyError::MalformedToken("JWK missing 'n'".to_string()))?;
        let e = jwk
            .e
            .as_deref()
            .ok_or_else(|| JwtVerifyError::MalformedToken("JWK missing 'e'".to_string()))?;

        let decoding_key = DecodingKey::from_rsa_components(n, e)
            .map_err(|e| JwtVerifyError::MalformedToken(format!("invalid RSA key: {e}")))?;

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

        // LOW-014 監査対応: decode エラーを jsonwebtoken の ErrorKind で種別判定し、
        // JwtVerifyError の適切なバリアントに変換する。
        let token_data = decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
            use jsonwebtoken::errors::ErrorKind;
            match e.kind() {
                // exp クレームが現在時刻より過去 → 期限切れ
                ErrorKind::ExpiredSignature => JwtVerifyError::TokenExpired,
                // 署名検証失敗 → 改ざん・偽造の可能性
                ErrorKind::InvalidSignature => JwtVerifyError::InvalidSignature,
                // iss クレーム不一致
                ErrorKind::InvalidIssuer => JwtVerifyError::InvalidIssuer,
                // aud クレーム不一致
                ErrorKind::InvalidAudience => JwtVerifyError::InvalidAudience,
                // nbf（Not Before）クレームが未到達
                ErrorKind::ImmatureSignature => {
                    JwtVerifyError::MalformedToken("JWT not yet valid (nbf)".to_string())
                }
                // その他（デコードエラー、鍵長不正等）
                _ => JwtVerifyError::MalformedToken(format!("JWT verification failed: {e}")),
            }
        })?;

        Ok(token_data.claims)
    }

    async fn get_jwks(&self) -> Result<Vec<Jwk>, JwtVerifyError> {
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

        // キャッシュ期限切れ: 再取得。ネットワークエラーは JwksFetchFailed に変換する。
        debug!("fetching JWKS from {}", self.jwks_url);
        let resp: JwksResponse = self
            .http_client
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|e| JwtVerifyError::JwksFetchFailed(e.to_string()))?
            .error_for_status()
            .map_err(|e| JwtVerifyError::JwksFetchFailed(e.to_string()))?
            .json()
            .await
            .map_err(|e| JwtVerifyError::JwksFetchFailed(e.to_string()))?;

        let mut cache = self.cache.write().await;
        *cache = Some(CachedJwks {
            keys: resp.keys.clone(),
            fetched_at: Instant::now(),
        });

        Ok(resp.keys)
    }
}
