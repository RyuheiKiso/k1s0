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
/// 成功した場合はトークン文字列を返す。ヘッダーがない・形式が違う場合は None を返す。
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

/// Tier の階層レベルを返す。
/// system(0) > business(1) > service(2) の順で上位 Tier ほど小さい値を返す。
fn tier_level(tier: &str) -> Option<u8> {
    match tier.to_ascii_lowercase().as_str() {
        "system" => Some(0),
        "business" => Some(1),
        "service" => Some(2),
        _ => None,
    }
}

/// tier_access 配列から、required_tier へのアクセスが許可されているかを判定する。
///
/// Tier 階層ルール:
/// - system tier を持つユーザーは全 Tier にアクセス可能
/// - business tier を持つユーザーは business と service にアクセス可能
/// - service tier を持つユーザーは service のみにアクセス可能
fn has_tier_access(tier_access: &[String], required_tier: &str) -> bool {
    let required_level = match tier_level(required_tier) {
        Some(level) => level,
        None => return false,
    };

    tier_access.iter().any(|user_tier| {
        tier_level(user_tier)
            .map(|user_level| user_level <= required_level)
            .unwrap_or(false)
    })
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
                "SYS_AUTH_MISSING_TOKEN",
                "Authorization header with Bearer token is required",
            );
        }
    };

    match state.validate_token_uc.execute(&token).await {
        Ok(claims) => {
            if !has_tier_access(&claims.tier_access, "system") {
                return error_response(
                    StatusCode::FORBIDDEN,
                    "SYS_AUTH_TIER_FORBIDDEN",
                    "Token does not include required tier access: system",
                );
            }
            req.extensions_mut().insert(claims);
            next.run(req).await
        }
        Err(_) => error_response(
            StatusCode::UNAUTHORIZED,
            "SYS_AUTH_TOKEN_INVALID",
            "Token validation failed",
        ),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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

    #[test]
    fn test_extract_bearer_token_bearer_only_no_space() {
        // "Bearer" だけで後のスペースも値もない場合
        let req = make_request_with_header("Bearer");
        let token = extract_bearer_token(&req);
        assert_eq!(token, None);
    }

    #[tokio::test]
    async fn test_auth_middleware_missing_token_returns_401() {
        use crate::domain::repository::audit_log_repository::MockAuditLogRepository;
        use crate::domain::repository::user_repository::MockUserRepository;
        use crate::infrastructure::MockTokenVerifier;
        use axum::middleware;
        use axum::routing::get;
        use axum::Router;
        use std::sync::Arc;
        use tower::ServiceExt;

        let state = {
            use crate::domain::repository::api_key_repository::MockApiKeyRepository;
            AppState::new(
                Arc::new(MockTokenVerifier::new()),
                Arc::new(MockUserRepository::new()),
                Arc::new(MockAuditLogRepository::new()),
                Arc::new(MockApiKeyRepository::new()),
                "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
                "k1s0-api".to_string(),
                None,
                None,
                None,
            )
        };

        let app = Router::new()
            .route("/protected", get(|| async { "ok" }))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state);

        let req = Request::builder()
            .uri("/protected")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "SYS_AUTH_MISSING_TOKEN");
        assert!(json["error"]["request_id"].is_string());
    }

    #[tokio::test]
    async fn test_auth_middleware_invalid_token_returns_401() {
        use crate::domain::repository::audit_log_repository::MockAuditLogRepository;
        use crate::domain::repository::user_repository::MockUserRepository;
        use crate::infrastructure::MockTokenVerifier;
        use axum::middleware;
        use axum::routing::get;
        use axum::Router;
        use std::sync::Arc;
        use tower::ServiceExt;

        let mut mock_verifier = MockTokenVerifier::new();
        mock_verifier
            .expect_verify_token()
            .returning(|_| Err(anyhow::anyhow!("invalid signature")));

        let state = {
            use crate::domain::repository::api_key_repository::MockApiKeyRepository;
            AppState::new(
                Arc::new(mock_verifier),
                Arc::new(MockUserRepository::new()),
                Arc::new(MockAuditLogRepository::new()),
                Arc::new(MockApiKeyRepository::new()),
                "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
                "k1s0-api".to_string(),
                None,
                None,
                None,
            )
        };

        let app = Router::new()
            .route("/protected", get(|| async { "ok" }))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state);

        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", "Bearer invalid-token")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "SYS_AUTH_TOKEN_INVALID");
        assert!(json["error"]["request_id"].is_string());
    }

    #[tokio::test]
    async fn test_auth_middleware_valid_token_passes_claims() {
        use crate::domain::entity::claims::{Claims, RealmAccess};
        use crate::domain::repository::audit_log_repository::MockAuditLogRepository;
        use crate::domain::repository::user_repository::MockUserRepository;
        use crate::infrastructure::MockTokenVerifier;
        use axum::middleware;
        use axum::routing::get;
        use axum::{Extension, Router};
        use std::collections::HashMap;
        use std::sync::Arc;
        use tower::ServiceExt;

        let valid_claims = Claims {
            sub: "user-uuid-1234".to_string(),
            iss: "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
            aud: "k1s0-api".to_string(),
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
            jti: "token-uuid".to_string(),
            typ: "Bearer".to_string(),
            azp: "react-spa".to_string(),
            scope: "openid".to_string(),
            preferred_username: "taro.yamada".to_string(),
            email: "taro@example.com".to_string(),
            realm_access: RealmAccess {
                roles: vec!["sys_admin".to_string()],
            },
            resource_access: HashMap::new(),
            tier_access: vec!["system".to_string()],
        };

        let return_claims = valid_claims.clone();
        let mut mock_verifier = MockTokenVerifier::new();
        mock_verifier
            .expect_verify_token()
            .returning(move |_| Ok(return_claims.clone()));

        let state = {
            use crate::domain::repository::api_key_repository::MockApiKeyRepository;
            AppState::new(
                Arc::new(mock_verifier),
                Arc::new(MockUserRepository::new()),
                Arc::new(MockAuditLogRepository::new()),
                Arc::new(MockApiKeyRepository::new()),
                "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
                "k1s0-api".to_string(),
                None,
                None,
                None,
            )
        };

        let app = Router::new()
            .route(
                "/protected",
                get(|Extension(claims): Extension<Claims>| async move {
                    axum::Json(serde_json::json!({"sub": claims.sub}))
                }),
            )
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state);

        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", "Bearer valid-token")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["sub"], "user-uuid-1234");
    }

    #[tokio::test]
    async fn test_auth_middleware_missing_required_tier_returns_403() {
        use crate::domain::entity::claims::{Claims, RealmAccess};
        use crate::domain::repository::audit_log_repository::MockAuditLogRepository;
        use crate::domain::repository::user_repository::MockUserRepository;
        use crate::infrastructure::MockTokenVerifier;
        use axum::middleware;
        use axum::routing::get;
        use axum::Router;
        use std::collections::HashMap;
        use std::sync::Arc;
        use tower::ServiceExt;

        let claims = Claims {
            sub: "user-uuid-1234".to_string(),
            iss: "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
            aud: "k1s0-api".to_string(),
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
            jti: "token-uuid".to_string(),
            typ: "Bearer".to_string(),
            azp: "react-spa".to_string(),
            scope: "openid".to_string(),
            preferred_username: "taro.yamada".to_string(),
            email: "taro@example.com".to_string(),
            realm_access: RealmAccess {
                roles: vec!["sys_admin".to_string()],
            },
            resource_access: HashMap::new(),
            tier_access: vec!["business".to_string()],
        };
        let return_claims = claims.clone();
        let mut mock_verifier = MockTokenVerifier::new();
        mock_verifier
            .expect_verify_token()
            .returning(move |_| Ok(return_claims.clone()));

        let state = {
            use crate::domain::repository::api_key_repository::MockApiKeyRepository;
            AppState::new(
                Arc::new(mock_verifier),
                Arc::new(MockUserRepository::new()),
                Arc::new(MockAuditLogRepository::new()),
                Arc::new(MockApiKeyRepository::new()),
                "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
                "k1s0-api".to_string(),
                None,
                None,
                None,
            )
        };

        let app = Router::new()
            .route("/protected", get(|| async { "ok" }))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state);

        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", "Bearer valid-token")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "SYS_AUTH_TIER_FORBIDDEN");
        assert!(json["error"]["request_id"].is_string());
    }
}
