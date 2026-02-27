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

/// NotificationAuthState は notification-server の認証ミドルウェアが使用する共有状態。
#[derive(Clone)]
pub struct NotificationAuthState {
    pub verifier: Arc<JwksVerifier>,
}

/// auth_middleware は Bearer トークンを検証し、k1s0_auth::Claims をリクエストエクステンションに格納する。
pub async fn auth_middleware(
    State(state): State<NotificationAuthState>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;

    fn make_request_with_header(header_value: &str) -> Request<Body> {
        Request::builder()
            .header("Authorization", header_value)
            .body(Body::empty())
            .unwrap()
    }

    fn make_request_without_auth() -> Request<Body> {
        Request::builder().body(Body::empty()).unwrap()
    }

    #[test]
    fn test_extract_bearer_token_valid() {
        let req = make_request_with_header("Bearer my-secret-token");
        let token = extract_bearer_token(&req);
        assert_eq!(token, Some("my-secret-token".to_string()));
    }

    #[test]
    fn test_extract_bearer_token_no_header() {
        let req = make_request_without_auth();
        let token = extract_bearer_token(&req);
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_bearer_token_wrong_scheme() {
        let req = make_request_with_header("Basic dXNlcjpwYXNz");
        let token = extract_bearer_token(&req);
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_bearer_token_empty_token() {
        let req = make_request_with_header("Bearer ");
        let token = extract_bearer_token(&req);
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_bearer_token_jwt_format() {
        let jwt = "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ1c2VyLTEifQ.signature";
        let req = make_request_with_header(&format!("Bearer {}", jwt));
        let token = extract_bearer_token(&req);
        assert_eq!(token, Some(jwt.to_string()));
    }
}
