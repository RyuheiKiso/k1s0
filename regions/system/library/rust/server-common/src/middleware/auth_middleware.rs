use std::sync::Arc;

use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};
use k1s0_auth::JwksVerifier;

use crate::ServiceError;

/// 汎用認証ステート。各サーバー固有の AuthState は不要になる。
#[derive(Clone)]
pub struct AuthState {
    pub verifier: Arc<JwksVerifier>,
}

/// 汎用認証ミドルウェア。Bearer トークンを検証し、Claims をリクエスト拡張に挿入する。
pub async fn auth_middleware(
    State(state): State<AuthState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, ServiceError> {
    let token = extract_bearer_token(&req)
        .ok_or_else(|| ServiceError::unauthorized("AUTH", "Missing bearer token"))?;

    let claims = state
        .verifier
        .verify_token(&token)
        .await
        .map_err(|_| ServiceError::unauthorized("AUTH", "Invalid or expired token"))?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

fn extract_bearer_token(req: &Request<Body>) -> Option<String> {
    let header = req.headers().get("Authorization")?.to_str().ok()?;
    let token = header.strip_prefix("Bearer ")?;
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}
