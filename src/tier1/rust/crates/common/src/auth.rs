// 本ファイルは Rust 共通の JWT 認証 interceptor / 検証ロジック。
//
// 設計正典:
//   docs/03_要件定義/00_共通規約.md §「認証認可」
//   docs/03_要件定義/30_非機能要件/E-AC-*（認証認可 NFR）
//
// 役割（Go 側 src/tier1/go/internal/common/auth.go と等価）:
//   3 Pod の gRPC server に挿す UnaryInterceptor で、`authorization: Bearer <jwt>`
//   header から JWT を抽出し、`TIER1_AUTH_MODE` 環境変数の値に応じて 3 通りに振る舞う:
//     - `off`:   認証 skip（dev 限定）。`AuthClaims` には dev 既定値を埋める。
//     - `hmac`:  共通秘密 `TIER1_AUTH_HMAC_SECRET` で HS256 を verify する。
//     - `jwks`:  `TIER1_AUTH_JWKS_URL` から JWKS を fetch しキャッシュ、RS256 を verify する。
//
// AuthClaims が gRPC の MetadataMap（`x-tenant-id` / `x-subject`）に
// 上書き伝搬され、後段の interceptor / handler が信頼できる。

// 標準同期型。
use std::sync::Arc;
// 非同期 RwLock。
use tokio::sync::RwLock;
// JWT デコード / 検証。
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};

/// JWT 検証モード。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthMode {
    /// dev 限定。認証を完全 skip する。
    Off,
    /// HMAC 共通鍵による HS256 検証。
    Hmac,
    /// JWKS endpoint から取得した RSA 鍵による RS256 検証。
    Jwks,
}

impl AuthMode {
    /// 環境変数 `TIER1_AUTH_MODE` を解釈する。未設定 / 空文字は `Off`。
    pub fn from_env() -> Self {
        match std::env::var("TIER1_AUTH_MODE").unwrap_or_default().to_lowercase().as_str() {
            "hmac" => AuthMode::Hmac,
            "jwks" => AuthMode::Jwks,
            _ => AuthMode::Off,
        }
    }
}

/// 検証済 JWT の最小情報。
/// Keycloak `realm_access.roles` を `roles` に展開し、handler / interceptor が
/// RBAC 判定（NFR-E-AC-002）で参照する。
#[derive(Debug, Clone, Default)]
pub struct AuthClaims {
    /// `tenant_id` クレーム（テナント境界の主キー）。
    pub tenant_id: String,
    /// `sub` クレーム（操作主体）。
    pub subject: String,
    /// Keycloak Realm Role 集合（NFR-E-AC-002 RBAC）。`realm_access.roles` を展開済。
    pub roles: Vec<String>,
}

impl AuthClaims {
    /// 指定 role を含むかを判定する（線形探索、roles は通常数件のため十分）。
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

/// 認証検証器。
///
/// 各モードに対応する内部状態を持ち、`verify` で `AuthClaims` を返す。
#[derive(Clone)]
pub struct Authenticator {
    /// 設定済モード。
    mode: AuthMode,
    /// HMAC 共通鍵（mode=hmac のみ使用）。
    hmac_secret: Option<Vec<u8>>,
    /// JWKS endpoint URL（mode=jwks のみ使用）。
    jwks_url: Option<String>,
    /// JWKS cache（key id → DecodingKey）。
    jwks_cache: Arc<RwLock<JwksCache>>,
}

/// JWKS cache 1 entry。
struct JwksCache {
    /// kid → 公開鍵。
    keys: std::collections::HashMap<String, DecodingKey>,
    /// 最終 fetch 時刻。
    fetched_at: std::time::Instant,
    /// TTL。
    ttl: std::time::Duration,
}

impl JwksCache {
    fn new() -> Self {
        Self {
            keys: std::collections::HashMap::new(),
            // 過去のため初回必ず fetch。
            fetched_at: std::time::Instant::now() - std::time::Duration::from_secs(3600),
            ttl: std::time::Duration::from_secs(600),
        }
    }
    fn is_stale(&self) -> bool {
        self.fetched_at.elapsed() > self.ttl
    }
}

impl Authenticator {
    /// 環境変数からビルドする。エラーは Pod 起動時に panic して fail-fast にする。
    pub fn from_env() -> Self {
        let mode = AuthMode::from_env();
        let hmac_secret = std::env::var("TIER1_AUTH_HMAC_SECRET")
            .ok()
            .filter(|s| !s.is_empty())
            .map(|s| s.into_bytes());
        let jwks_url = std::env::var("TIER1_AUTH_JWKS_URL").ok().filter(|s| !s.is_empty());
        Self {
            mode,
            hmac_secret,
            jwks_url,
            jwks_cache: Arc::new(RwLock::new(JwksCache::new())),
        }
    }

    /// テスト用に直接モードと鍵を指定する。
    pub fn new_hmac(secret: Vec<u8>) -> Self {
        Self {
            mode: AuthMode::Hmac,
            hmac_secret: Some(secret),
            jwks_url: None,
            jwks_cache: Arc::new(RwLock::new(JwksCache::new())),
        }
    }

    /// dev 用 off モードで作る。
    pub fn off() -> Self {
        Self {
            mode: AuthMode::Off,
            hmac_secret: None,
            jwks_url: None,
            jwks_cache: Arc::new(RwLock::new(JwksCache::new())),
        }
    }

    /// 現在のモードを返す（テスト / log で利用）。
    pub fn mode(&self) -> &AuthMode {
        &self.mode
    }

    /// `Authorization: Bearer <jwt>` を verify して `AuthClaims` を返す。
    ///
    /// `Off` モードでは header が無くても dev 既定 claims を返す。
    pub async fn verify_bearer(&self, header_value: Option<&str>) -> Result<AuthClaims, tonic::Status> {
        match self.mode {
            AuthMode::Off => Ok(AuthClaims {
                // dev 既定（Go 側 `off mode` と同じ "demo-tenant"）。
                tenant_id: "demo-tenant".to_string(),
                subject: "dev".to_string(),
                // dev では role 検査の対象外。空 Vec で「全 role 不在」を表現する。
                roles: Vec::new(),
            }),
            AuthMode::Hmac => {
                let token = extract_bearer(header_value)?;
                let secret = self.hmac_secret.as_ref().ok_or_else(|| {
                    tonic::Status::failed_precondition(
                        "tier1/auth: TIER1_AUTH_HMAC_SECRET is required when TIER1_AUTH_MODE=hmac",
                    )
                })?;
                let key = DecodingKey::from_secret(secret);
                // HS256 / HS384 / HS512 を許容する（Go 側挙動と一致）。
                let mut v = Validation::new(Algorithm::HS256);
                v.algorithms = vec![Algorithm::HS256, Algorithm::HS384, Algorithm::HS512];
                v.validate_exp = true;
                let data = decode::<JwtClaims>(token, &key, &v)
                    .map_err(|e| tonic::Status::unauthenticated(format!("tier1/auth: {}", e)))?;
                claims_to_auth(data.claims)
            }
            AuthMode::Jwks => {
                let token = extract_bearer(header_value)?;
                let url = self.jwks_url.as_ref().ok_or_else(|| {
                    tonic::Status::failed_precondition(
                        "tier1/auth: TIER1_AUTH_JWKS_URL is required when TIER1_AUTH_MODE=jwks",
                    )
                })?;
                self.refresh_jwks_if_stale(url).await?;
                let header = decode_header(token)
                    .map_err(|e| tonic::Status::unauthenticated(format!("tier1/auth: {}", e)))?;
                let kid = header.kid.ok_or_else(|| {
                    tonic::Status::unauthenticated("tier1/auth: jwt header lacks kid")
                })?;
                let key = {
                    let g = self.jwks_cache.read().await;
                    g.keys.get(&kid).cloned().ok_or_else(|| {
                        tonic::Status::unauthenticated(format!(
                            "tier1/auth: kid {} not in JWKS",
                            kid
                        ))
                    })?
                };
                let mut v = Validation::new(Algorithm::RS256);
                v.algorithms = vec![Algorithm::RS256, Algorithm::RS384, Algorithm::RS512];
                v.validate_exp = true;
                let data = decode::<JwtClaims>(token, &key, &v)
                    .map_err(|e| tonic::Status::unauthenticated(format!("tier1/auth: {}", e)))?;
                claims_to_auth(data.claims)
            }
        }
    }

    /// JWKS cache が stale なら fetch し直す。
    async fn refresh_jwks_if_stale(&self, url: &str) -> Result<(), tonic::Status> {
        // fast path: cache が新しければ何もしない。
        {
            let g = self.jwks_cache.read().await;
            if !g.is_stale() {
                return Ok(());
            }
        }
        let body: serde_json::Value = reqwest::get(url)
            .await
            .map_err(|e| tonic::Status::unavailable(format!("tier1/auth: jwks fetch: {}", e)))?
            .json()
            .await
            .map_err(|e| tonic::Status::unavailable(format!("tier1/auth: jwks parse: {}", e)))?;
        let mut new_keys = std::collections::HashMap::new();
        if let Some(arr) = body.get("keys").and_then(|v| v.as_array()) {
            for jwk in arr {
                let kid = jwk.get("kid").and_then(|v| v.as_str()).unwrap_or_default();
                let n = jwk.get("n").and_then(|v| v.as_str()).unwrap_or_default();
                let e = jwk.get("e").and_then(|v| v.as_str()).unwrap_or_default();
                if kid.is_empty() || n.is_empty() || e.is_empty() {
                    continue;
                }
                if let Ok(k) = DecodingKey::from_rsa_components(n, e) {
                    new_keys.insert(kid.to_string(), k);
                }
            }
        }
        if new_keys.is_empty() {
            return Err(tonic::Status::failed_precondition(
                "tier1/auth: JWKS endpoint returned no usable keys",
            ));
        }
        let mut w = self.jwks_cache.write().await;
        w.keys = new_keys;
        w.fetched_at = std::time::Instant::now();
        Ok(())
    }
}

/// `Authorization: Bearer <jwt>` から jwt 部分を抽出する。
fn extract_bearer(h: Option<&str>) -> Result<&str, tonic::Status> {
    let h = h.ok_or_else(|| tonic::Status::unauthenticated("tier1/auth: missing authorization"))?;
    let prefix = "Bearer ";
    if let Some(rest) = h.strip_prefix(prefix) {
        if !rest.is_empty() {
            return Ok(rest.trim());
        }
    }
    Err(tonic::Status::unauthenticated(
        "tier1/auth: invalid authorization header (expect 'Bearer <jwt>')",
    ))
}

/// JWT claims の最小デシリアライズ用。
#[derive(Debug, serde::Deserialize)]
struct JwtClaims {
    /// 主体。
    #[serde(default)]
    sub: String,
    /// テナント ID。docs §共通規約 では `tenant_id` クレーム必須。
    #[serde(default)]
    tenant_id: String,
    /// Keycloak realm_access（roles 配列を含む）。`realm_access` クレーム不在時は空。
    #[serde(default)]
    realm_access: RealmAccessClaim,
}

/// Keycloak `realm_access` クレーム。
#[derive(Debug, Default, serde::Deserialize)]
struct RealmAccessClaim {
    /// Realm Role 一覧。
    #[serde(default)]
    roles: Vec<String>,
}

fn claims_to_auth(c: JwtClaims) -> Result<AuthClaims, tonic::Status> {
    if c.tenant_id.is_empty() {
        return Err(tonic::Status::permission_denied(
            "tier1/auth: jwt missing tenant_id claim",
        ));
    }
    Ok(AuthClaims {
        tenant_id: c.tenant_id,
        subject: c.sub,
        roles: c.realm_access.roles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{EncodingKey, Header, encode};

    #[derive(serde::Serialize)]
    struct Encode<'a> {
        sub: &'a str,
        tenant_id: &'a str,
        exp: i64,
    }

    fn now_plus_60() -> i64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 60
    }

    #[tokio::test]
    async fn off_mode_returns_dev_claims() {
        let a = Authenticator::off();
        let c = a.verify_bearer(None).await.unwrap();
        assert_eq!(c.tenant_id, "demo-tenant");
    }

    #[tokio::test]
    async fn hmac_mode_verifies_valid_token() {
        let secret = b"unit-test-secret-32bytes-long-aaa".to_vec();
        let a = Authenticator::new_hmac(secret.clone());
        let token = encode(
            &Header::new(Algorithm::HS256),
            &Encode {
                sub: "alice",
                tenant_id: "T1",
                exp: now_plus_60(),
            },
            &EncodingKey::from_secret(&secret),
        )
        .unwrap();
        let header = format!("Bearer {}", token);
        let c = a.verify_bearer(Some(&header)).await.unwrap();
        assert_eq!(c.tenant_id, "T1");
        assert_eq!(c.subject, "alice");
    }

    #[tokio::test]
    async fn hmac_mode_rejects_missing_tenant_id() {
        let secret = b"unit-test-secret-32bytes-long-aaa".to_vec();
        let a = Authenticator::new_hmac(secret.clone());
        let token = encode(
            &Header::new(Algorithm::HS256),
            &Encode {
                sub: "alice",
                tenant_id: "",
                exp: now_plus_60(),
            },
            &EncodingKey::from_secret(&secret),
        )
        .unwrap();
        let header = format!("Bearer {}", token);
        let r = a.verify_bearer(Some(&header)).await;
        assert!(r.is_err());
        assert_eq!(r.err().unwrap().code(), tonic::Code::PermissionDenied);
    }

    #[tokio::test]
    async fn hmac_mode_rejects_invalid_signature() {
        let a = Authenticator::new_hmac(b"correct-secret-32bytes-long-aaaaa".to_vec());
        let token = encode(
            &Header::new(Algorithm::HS256),
            &Encode {
                sub: "alice",
                tenant_id: "T1",
                exp: now_plus_60(),
            },
            &EncodingKey::from_secret(b"wrong-secret-32bytes-long-aaaaaaaa"),
        )
        .unwrap();
        let header = format!("Bearer {}", token);
        let r = a.verify_bearer(Some(&header)).await;
        assert!(r.is_err());
        assert_eq!(r.err().unwrap().code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn extract_bearer_rejects_other_schemes() {
        assert!(extract_bearer(Some("Basic dXNlcjpwYXNz")).is_err());
        assert!(extract_bearer(None).is_err());
        assert!(extract_bearer(Some("Bearer ")).is_err());
        assert!(extract_bearer(Some("Bearer abc")).is_ok());
    }
}
