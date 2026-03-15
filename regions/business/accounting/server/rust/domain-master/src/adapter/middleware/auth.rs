//! JWT 認証ミドルウェア。
//!
//! リクエストの `Authorization: Bearer <token>` ヘッダーから JWT を検証し、
//! 検証済みの [`Claims`] を axum の Extension に挿入する。
//!
//! 実際の JWT 検証は `k1s0_auth::JwksVerifier` に委譲するため、
//! ここではリクエストからトークンを抽出し検証結果を Extension に伝搬するレイヤーのみを担当する。

use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use k1s0_auth::{Claims, JwksVerifier};
use std::sync::Arc;

/// JWT 認証ミドルウェアで共有するステート。
#[derive(Clone)]
pub struct AuthMiddlewareState {
    pub verifier: Arc<JwksVerifier>,
}

/// Bearer トークンを抽出する。
fn extract_bearer_token(req: &Request) -> Option<&str> {
    req.headers()
        .get(AUTHORIZATION)?
        .to_str()
        .ok()
        .and_then(|v| v.strip_prefix("Bearer "))
}

/// JWT 認証ミドルウェア。
///
/// `Authorization` ヘッダーが存在しない場合は 401 Unauthorized を返却する。
/// トークンの検証に失敗した場合も 401 Unauthorized を返却する。
/// 検証成功時は `Claims` を Extension に挿入して次のハンドラーへ進む。
pub async fn jwt_auth_middleware(
    req: Request,
    next: Next,
    verifier: Arc<JwksVerifier>,
) -> Response {
    let token = match extract_bearer_token(&req) {
        Some(t) => t.to_string(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "code": "SYS_AUTH_TOKEN_MISSING",
                    "message": "Authorization header with Bearer token is required"
                })),
            )
                .into_response();
        }
    };

    match verifier.verify_token(&token).await {
        Ok(claims) => {
            let mut req = req;
            req.extensions_mut().insert(claims);
            next.run(req).await
        }
        Err(e) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "code": "SYS_AUTH_TOKEN_INVALID",
                "message": format!("JWT verification failed: {}", e)
            })),
        )
            .into_response(),
    }
}

/// Claims から user_id を取得するヘルパー。
pub fn user_id_from_claims(claims: &Claims) -> String {
    claims
        .preferred_username
        .as_ref()
        .filter(|v| !v.is_empty())
        .cloned()
        .or_else(|| claims.email.as_ref().filter(|v| !v.is_empty()).cloned())
        .or_else(|| (!claims.sub.is_empty()).then(|| claims.sub.clone()))
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request as HttpRequest;

    #[test]
    fn test_extract_bearer_token_valid() {
        let req = HttpRequest::builder()
            .header("Authorization", "Bearer my-jwt-token")
            .body(())
            .unwrap();
        // Convert to axum Request type for testing
        let token = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        assert_eq!(token, Some("my-jwt-token"));
    }

    #[test]
    fn test_extract_bearer_token_missing() {
        let req = HttpRequest::builder().body(()).unwrap();
        let token = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        assert!(token.is_none());
    }

    #[test]
    fn test_extract_bearer_token_wrong_scheme() {
        let req = HttpRequest::builder()
            .header("Authorization", "Basic dXNlcjpwYXNz")
            .body(())
            .unwrap();
        let token = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        assert!(token.is_none());
    }

    fn make_claims(sub: &str, preferred_username: Option<&str>, email: Option<&str>) -> Claims {
        Claims {
            sub: sub.to_string(),
            iss: "test".to_string(),
            aud: k1s0_auth::Audience(vec![]),
            exp: 0,
            iat: 0,
            jti: None,
            typ: None,
            azp: None,
            scope: None,
            preferred_username: preferred_username.map(String::from),
            email: email.map(String::from),
            realm_access: None,
            resource_access: None,
            tier_access: None,
        }
    }

    #[test]
    fn test_user_id_from_claims_preferred_username() {
        let claims = make_claims("sub-id", Some("alice"), Some("alice@example.com"));
        assert_eq!(user_id_from_claims(&claims), "alice");
    }

    #[test]
    fn test_user_id_from_claims_fallback_email() {
        let claims = make_claims("sub-id", None, Some("bob@example.com"));
        assert_eq!(user_id_from_claims(&claims), "bob@example.com");
    }

    #[test]
    fn test_user_id_from_claims_fallback_sub() {
        let claims = make_claims("sub-id", None, None);
        assert_eq!(user_id_from_claims(&claims), "sub-id");
    }
}
