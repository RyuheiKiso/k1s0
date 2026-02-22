//! axum 用の認証ミドルウェア。

use crate::claims::Claims;
use crate::rbac;
use crate::verifier::{AuthError, JwksVerifier};
use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;

/// ミドルウェアファクトリの戻り値型。
type AuthMiddlewareFuture = std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<Response, AuthErrorResponse>> + Send>,
>;

/// AuthState はミドルウェアが使用する共有状態。
#[derive(Clone)]
pub struct AuthState {
    pub verifier: Arc<JwksVerifier>,
}

/// auth_middleware は JWT 認証ミドルウェア。
/// Authorization ヘッダーから Bearer トークンを取得し、JWKS 検証を行う。
/// 検証成功時は Claims をリクエストエクステンションに格納する。
pub async fn auth_middleware(
    State(state): State<AuthState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AuthErrorResponse> {
    let token = extract_bearer_token(&req)?;

    let claims = state
        .verifier
        .verify_token(&token)
        .await
        .map_err(AuthErrorResponse::from_auth_error)?;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}

/// require_role は指定ロールを必須とするミドルウェアファクトリ。
/// auth_middleware の後に使用すること。
pub fn require_role(
    role: &'static str,
) -> impl Fn(Request<Body>, Next) -> AuthMiddlewareFuture + Clone {
    move |req: Request<Body>, next: Next| {
        Box::pin(async move {
            let claims = req
                .extensions()
                .get::<Claims>()
                .ok_or_else(AuthErrorResponse::unauthenticated)?;

            if !rbac::has_role(claims, role) {
                return Err(AuthErrorResponse::forbidden(
                    "この操作を実行する権限がありません",
                ));
            }

            Ok(next.run(req).await)
        })
    }
}

/// require_permission は指定リソース・アクションの権限を必須とするミドルウェアファクトリ。
/// auth_middleware の後に使用すること。
pub fn require_permission(
    resource: &'static str,
    action: &'static str,
) -> impl Fn(Request<Body>, Next) -> AuthMiddlewareFuture + Clone {
    move |req: Request<Body>, next: Next| {
        Box::pin(async move {
            let claims = req
                .extensions()
                .get::<Claims>()
                .ok_or_else(AuthErrorResponse::unauthenticated)?;

            if !rbac::has_permission(claims, resource, action) {
                return Err(AuthErrorResponse::forbidden(
                    "この操作を実行する権限がありません",
                ));
            }

            Ok(next.run(req).await)
        })
    }
}

/// require_tier_access は指定 Tier へのアクセスを必須とするミドルウェアファクトリ。
/// auth_middleware の後に使用すること。
pub fn require_tier_access(
    tier: &'static str,
) -> impl Fn(Request<Body>, Next) -> AuthMiddlewareFuture + Clone {
    move |req: Request<Body>, next: Next| {
        Box::pin(async move {
            let claims = req
                .extensions()
                .get::<Claims>()
                .ok_or_else(AuthErrorResponse::unauthenticated)?;

            if !rbac::has_tier_access(claims, tier) {
                return Err(AuthErrorResponse::tier_forbidden());
            }

            Ok(next.run(req).await)
        })
    }
}

/// リクエストエクステンションから Claims を取得する。
pub fn get_claims(req: &Request<Body>) -> Option<&Claims> {
    req.extensions().get::<Claims>()
}

/// Bearer トークンを Authorization ヘッダーから取得する。
fn extract_bearer_token(req: &Request<Body>) -> Result<String, AuthErrorResponse> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AuthErrorResponse::from_auth_error(AuthError::MissingToken))?;

    let parts: Vec<&str> = auth_header.splitn(2, ' ').collect();
    if parts.len() != 2 || !parts[0].eq_ignore_ascii_case("Bearer") {
        return Err(AuthErrorResponse::from_auth_error(
            AuthError::InvalidAuthHeader,
        ));
    }

    let token = parts[1].trim();
    if token.is_empty() {
        return Err(AuthErrorResponse::from_auth_error(AuthError::MissingToken));
    }

    Ok(token.to_string())
}

/// AuthErrorResponse は認証エラーの HTTP レスポンス。
#[derive(Debug)]
pub struct AuthErrorResponse {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
}

impl AuthErrorResponse {
    fn from_auth_error(err: AuthError) -> Self {
        match err {
            AuthError::MissingToken | AuthError::InvalidAuthHeader => Self {
                status: StatusCode::UNAUTHORIZED,
                code: "SYS_AUTH_UNAUTHENTICATED".into(),
                message: "認証が必要です".into(),
            },
            AuthError::TokenExpired => Self {
                status: StatusCode::UNAUTHORIZED,
                code: "SYS_AUTH_TOKEN_EXPIRED".into(),
                message: "トークンの有効期限が切れています".into(),
            },
            AuthError::InvalidToken(_) => Self {
                status: StatusCode::UNAUTHORIZED,
                code: "SYS_AUTH_INVALID_TOKEN".into(),
                message: "トークンが無効です".into(),
            },
            AuthError::JwksFetchFailed(_) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "SYS_AUTH_JWKS_ERROR".into(),
                message: "認証サービスへの接続に失敗しました".into(),
            },
            AuthError::PermissionDenied => Self::forbidden("この操作を実行する権限がありません"),
            AuthError::TierAccessDenied => Self::tier_forbidden(),
        }
    }

    fn unauthenticated() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "SYS_AUTH_UNAUTHENTICATED".into(),
            message: "認証が必要です".into(),
        }
    }

    fn forbidden(message: &str) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: "SYS_AUTH_FORBIDDEN".into(),
            message: message.into(),
        }
    }

    fn tier_forbidden() -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: "SYS_AUTH_TIER_FORBIDDEN".into(),
            message: "このTierへのアクセス権がありません".into(),
        }
    }
}

impl IntoResponse for AuthErrorResponse {
    fn into_response(self) -> Response {
        let body = json!({
            "error": self.code,
            "message": self.message,
        });

        (self.status, Json(body)).into_response()
    }
}
