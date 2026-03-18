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

/// Authorization ヘッダーから Bearer トークンを取り出すヘルパー。
/// ヘッダーが存在しない・Bearer プレフィックスがない・トークンが空の場合は None を返す。
fn extract_bearer_token(req: &Request<Body>) -> Option<String> {
    let header = req.headers().get("Authorization")?.to_str().ok()?;
    let token = header.strip_prefix("Bearer ")?;
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

    // 有効な Bearer トークンが正しく抽出されることを確認する
    #[test]
    fn test_extract_bearer_token_valid() {
        let req = make_request_with_header("Bearer my-secret-token");
        let token = extract_bearer_token(&req);
        assert_eq!(token, Some("my-secret-token".to_string()));
    }

    // Authorization ヘッダーがない場合 None が返ることを確認する
    #[test]
    fn test_extract_bearer_token_no_header() {
        let req = make_request_without_auth();
        let token = extract_bearer_token(&req);
        assert_eq!(token, None);
    }

    // Basic 認証スキームの場合 None が返ることを確認する
    #[test]
    fn test_extract_bearer_token_wrong_scheme() {
        let req = make_request_with_header("Basic dXNlcjpwYXNz");
        let token = extract_bearer_token(&req);
        assert_eq!(token, None);
    }

    // "Bearer " の後が空の場合 None が返ることを確認する
    #[test]
    fn test_extract_bearer_token_empty_token() {
        let req = make_request_with_header("Bearer ");
        let token = extract_bearer_token(&req);
        assert_eq!(token, None);
    }

    // JWT 形式のトークンが正しく抽出されることを確認する
    #[test]
    fn test_extract_bearer_token_jwt_format() {
        let jwt = "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiJ1c2VyLTEifQ.signature";
        let req = make_request_with_header(&format!("Bearer {}", jwt));
        let token = extract_bearer_token(&req);
        assert_eq!(token, Some(jwt.to_string()));
    }

    // "Bearer" のみ (スペースなし) の場合 None が返ることを確認する
    #[test]
    fn test_extract_bearer_token_no_space() {
        let req = make_request_with_header("Bearer");
        let token = extract_bearer_token(&req);
        assert_eq!(token, None);
    }
}
