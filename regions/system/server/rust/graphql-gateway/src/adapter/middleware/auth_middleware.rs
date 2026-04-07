use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::infrastructure::auth::{JwksVerifier, JwtVerifyError};

/// raw JWT トークン文字列。
/// クエリ引数経由でトークンを渡すとアクセスログに記録されるリスクがあるため、
/// コンテキスト Extension として保持し下流サービスへの転送に使用する（M-3 監査対応）。
#[derive(Debug, Clone)]
pub struct BearerToken(pub String);

/// JWT Claims（async-graphql の Extension として GraphQL Context に注入）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub preferred_username: Option<String>,
    pub email: Option<String>,
    pub realm_access: Option<RealmAccess>,
    pub exp: i64,
}

impl Claims {
    pub fn roles(&self) -> Vec<String> {
        self.realm_access
            .as_ref()
            .map(|r| r.roles.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

/// Authorization ヘッダーから Bearer トークンを抽出する。
/// RFC 7235: Authorization スキーム名は大文字小文字を区別しない（RUST-HIGH-001 対応）
fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth_str = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())?;
    // "Bearer ", "bearer ", "BEARER " いずれも受け入れる
    const BEARER_PREFIX_LEN: usize = 7; // "bearer ".len()
    if auth_str.len() < BEARER_PREFIX_LEN {
        return None;
    }
    if !auth_str[..BEARER_PREFIX_LEN].eq_ignore_ascii_case("bearer ") {
        return None;
    }
    let token = &auth_str[BEARER_PREFIX_LEN..];
    if token.is_empty() {
        None
    } else {
        Some(token.to_owned())
    }
}

/// AuthMiddlewareLayer は JwksVerifier を保持し、JWT 検証ミドルウェアを提供する Tower Layer。
#[derive(Clone)]
pub struct AuthMiddlewareLayer {
    verifier: Arc<JwksVerifier>,
}

impl AuthMiddlewareLayer {
    pub fn new(verifier: Arc<JwksVerifier>) -> Self {
        Self { verifier }
    }
}

impl<S> tower::Layer<S> for AuthMiddlewareLayer {
    type Service = AuthMiddlewareService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddlewareService {
            inner,
            verifier: self.verifier.clone(),
        }
    }
}

/// AuthMiddlewareService は JWT 検証を行う Tower Service。
#[derive(Clone)]
pub struct AuthMiddlewareService<S> {
    inner: S,
    verifier: Arc<JwksVerifier>,
}

impl<S> tower::Service<Request<Body>> for AuthMiddlewareService<S>
where
    S: tower::Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let verifier = self.verifier.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let token = extract_bearer_token(req.headers());

            let token = match token {
                Some(t) => t,
                None => {
                    let request_id = uuid::Uuid::new_v4().to_string();
                    return Ok((
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({
                            "error": {
                                "code": "SYS_AUTH_TOKEN_MISSING",
                                "message": "missing Authorization header",
                                "request_id": request_id
                            }
                        })),
                    )
                        .into_response());
                }
            };

            // LOW-014 監査対応: JWT 検証エラーを種別ごとに区別し、クライアントに
            // 適切なエラーコードを返す。TokenExpired と InvalidSignature は
            // セキュリティ上の意味が異なるため、ログレベルも分けて記録する。
            let claims = match verifier.verify_token(&token).await {
                Ok(c) => c,
                Err(e) => {
                    let request_id = uuid::Uuid::new_v4().to_string();
                    let (code, message) = match &e {
                        // トークン期限切れ: 正常なセッション切れであることが多い
                        JwtVerifyError::TokenExpired => (
                            "SYS_AUTH_TOKEN_EXPIRED",
                            "Token has expired. Please log in again.",
                        ),
                        // 署名不正: 改ざん・偽造の可能性がある（セキュリティアラート対象）
                        JwtVerifyError::InvalidSignature => (
                            "SYS_AUTH_TOKEN_INVALID_SIGNATURE",
                            "Invalid JWT signature.",
                        ),
                        // issuer 不一致: 設定ミスまたは別環境のトークン
                        JwtVerifyError::InvalidIssuer => (
                            "SYS_AUTH_TOKEN_INVALID_ISSUER",
                            "Invalid JWT issuer.",
                        ),
                        // audience 不一致: 別サービス向けのトークンを使用している可能性
                        JwtVerifyError::InvalidAudience => (
                            "SYS_AUTH_TOKEN_INVALID_AUDIENCE",
                            "Invalid JWT audience.",
                        ),
                        // JWKS 取得失敗: 認証サービスの一時障害（再試行可能）
                        JwtVerifyError::JwksFetchFailed(_) => (
                            "SYS_AUTH_JWKS_UNAVAILABLE",
                            "Authentication service is temporarily unavailable. Please try again later.",
                        ),
                        // その他の不正フォーマット
                        JwtVerifyError::MalformedToken(_) => (
                            "SYS_AUTH_TOKEN_MALFORMED",
                            "Malformed JWT token.",
                        ),
                    };
                    return Ok((
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({
                            "error": {
                                "code": code,
                                "message": message,
                                "request_id": request_id
                            }
                        })),
                    )
                        .into_response());
                }
            };

            // raw トークンを BearerToken Extension として保存し、下流サービスへの転送に使用する
            req.extensions_mut().insert(BearerToken(token.clone()));
            req.extensions_mut().insert(claims);
            inner.call(req).await
        })
    }
}
