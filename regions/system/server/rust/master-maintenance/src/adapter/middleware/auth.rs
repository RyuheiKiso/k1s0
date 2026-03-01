use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use k1s0_auth::JwksVerifier;
use crate::adapter::handler::error::AppError;

#[derive(Clone)]
pub struct MasterMaintenanceAuthState {
    pub verifier: Arc<JwksVerifier>,
}

pub async fn auth_middleware(
    State(state): State<MasterMaintenanceAuthState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_bearer_token(&req)
        .ok_or_else(|| AppError::unauthorized("SYS_AUTH_MISSING_TOKEN", "Missing bearer token"))?;

    let claims = state
        .verifier
        .verify_token(&token)
        .await
        .map_err(|_| AppError::unauthorized("SYS_AUTH_TOKEN_INVALID", "Invalid or expired token"))?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

fn extract_bearer_token(req: &Request<Body>) -> Option<String> {
    let header = req.headers().get("Authorization")?.to_str().ok()?;
    if header.starts_with("Bearer ") {
        let token = header[7..].trim().to_string();
        if token.is_empty() {
            None
        } else {
            Some(token)
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_bearer_token_valid() {
        let mut req = Request::builder().body(Body::empty()).unwrap();
        req.headers_mut().insert("Authorization", HeaderValue::from_static("Bearer my-token"));
        assert_eq!(extract_bearer_token(&req), Some("my-token".to_string()));
    }

    #[test]
    fn test_extract_bearer_token_missing() {
        let req = Request::builder().body(Body::empty()).unwrap();
        assert_eq!(extract_bearer_token(&req), None);
    }

    #[test]
    fn test_extract_bearer_token_wrong_scheme() {
        let mut req = Request::builder().body(Body::empty()).unwrap();
        req.headers_mut().insert("Authorization", HeaderValue::from_static("Basic abc123"));
        assert_eq!(extract_bearer_token(&req), None);
    }
}
