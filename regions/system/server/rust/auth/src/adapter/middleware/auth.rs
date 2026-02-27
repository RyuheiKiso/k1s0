use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::adapter::handler::AppState;

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

    match state.validate_token_uc.execute(&token).await {
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
            tier_access: vec![],
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
}
