// 認証ミドルウェア
// Bearer トークンを検証し、Claims をリクエストエクステンションに格納する

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;

use k1s0_auth::JwksVerifier;

/// AgentAuthState は ai-agent-server の認証ミドルウェアが使用する共有状態
#[derive(Clone)]
pub struct AgentAuthState {
    pub verifier: Arc<JwksVerifier>,
}

/// auth_middleware は Bearer トークンを検証し、Claims をリクエストエクステンションに格納する
pub async fn auth_middleware(
    State(state): State<AgentAuthState>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let token = match extract_bearer_token(&req) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": {
                        "code": "SYS_AUTH_MISSING_TOKEN",
                        "message": "Authorization header with Bearer token is required"
                    }
                })),
            )
                .into_response();
        }
    };

    match state.verifier.verify_token(&token).await {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            next.run(req).await
        }
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": {
                    "code": "SYS_AUTH_TOKEN_INVALID",
                    "message": "Token validation failed"
                }
            })),
        )
            .into_response(),
    }
}

/// リクエストヘッダーからBearerトークンを抽出する
fn extract_bearer_token(req: &Request<Body>) -> Option<String> {
    let auth_header = req.headers().get(axum::http::header::AUTHORIZATION)?;
    let auth_str = auth_header.to_str().ok()?;
    let token = auth_str.strip_prefix("Bearer ")?;
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}
