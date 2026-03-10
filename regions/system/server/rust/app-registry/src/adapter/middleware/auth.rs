use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use k1s0_server_common::ErrorResponse;

use crate::adapter::handler::AppState;

fn error_response(status: StatusCode, code: &str, message: impl Into<String>) -> Response {
    (status, Json(ErrorResponse::new(code, message.into()))).into_response()
}

/// Authorization ヘッダーから Bearer トークンを取り出すヘルパー。
pub fn extract_bearer_token<B>(req: &Request<B>) -> Option<String> {
    let auth_header = req.headers().get(axum::http::header::AUTHORIZATION)?;
    let auth_str = auth_header.to_str().ok()?;
    let token = auth_str.strip_prefix("Bearer ")?;
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

/// auth_middleware は Bearer トークンを検証して、Request extension に Claims を格納する axum ミドルウェア。
/// トークンが存在しないか無効な場合は 401 Unauthorized を返す。
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let token = match extract_bearer_token(&req) {
        Some(t) => t,
        None => {
            return error_response(
                StatusCode::UNAUTHORIZED,
                "SYS_APPS_MISSING_TOKEN",
                "Authorization header with Bearer token is required",
            );
        }
    };

    match state.validate_token_uc.execute(&token).await {
        Ok(claims) => {
            // system tier アクセスチェック
            let has_system = claims
                .tier_access
                .iter()
                .any(|tier| tier.eq_ignore_ascii_case("system"));
            if !has_system {
                return error_response(
                    StatusCode::FORBIDDEN,
                    "SYS_APPS_TIER_FORBIDDEN",
                    "Token does not include required tier access: system",
                );
            }
            req.extensions_mut().insert(claims);
            next.run(req).await
        }
        Err(_) => error_response(
            StatusCode::UNAUTHORIZED,
            "SYS_APPS_TOKEN_INVALID",
            "Token validation failed",
        ),
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
}
