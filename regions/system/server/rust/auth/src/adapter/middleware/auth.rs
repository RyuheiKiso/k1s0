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
/// RFC 7235: Authorization スキーム名は大文字小文字を区別しない（RUST-HIGH-001 対応）
/// 成功した場合はトークン文字列を返す。ヘッダーがない・形式が違う場合は None を返す。
pub fn extract_bearer_token<B>(req: &Request<B>) -> Option<String> {
    // "Bearer ", "bearer ", "BEARER " いずれも受け入れる
    const BEARER_PREFIX_LEN: usize = 7; // "bearer ".len()
    let auth_header = req.headers().get(axum::http::header::AUTHORIZATION)?;
    let auth_str = auth_header.to_str().ok()?;
    if auth_str.len() < BEARER_PREFIX_LEN {
        return None;
    }
    if !auth_str[..BEARER_PREFIX_LEN].eq_ignore_ascii_case("bearer ") {
        return None;
    }
    let token = &auth_str[BEARER_PREFIX_LEN..];
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

/// LOW-13 監査対応: 設定ファイルから注入された Tier 階層リストを使い、指定 Tier の階層レベルを返す。
/// インデックスが小さいほど上位（高権限）の Tier であり、system=0 > business=1 > service=2 となる。
/// ハードコードを排除し、新 Tier 追加時は設定変更のみで対応可能とする。
fn tier_level(tiers: &[String], tier: &str) -> Option<u8> {
    tiers
        .iter()
        .position(|t| t.eq_ignore_ascii_case(tier))
        // LOW-008: 安全な型変換（オーバーフロー防止）
        .map(|i| u8::try_from(i).unwrap_or(u8::MAX))
}

/// LOW-13 監査対応: 設定駆動の Tier 階層を用いて `tier_access` から `required_tier` へのアクセス可否を判定する。
///
/// Tier 階層ルール（tiers リストの順序に従う）:
/// - 上位 Tier（インデックスが小さい）を持つユーザーは、それ以下の全 Tier にアクセス可能
/// - 設定ファイルの `tier_hierarchy.tiers` に存在しない Tier は拒否される
fn has_tier_access(tiers: &[String], tier_access: &[String], required_tier: &str) -> bool {
    // 要求されたTierが設定済み階層に存在しない場合はアクセス拒否
    let Some(required_level) = tier_level(tiers, required_tier) else {
        return false;
    };

    tier_access.iter().any(|user_tier| {
        tier_level(tiers, user_tier).is_some_and(|user_level| user_level <= required_level)
    })
}

/// `auth_middleware` は Bearer トークンを検証して、Request extension に Claims を格納する axum ミドルウェア。
/// トークンが存在しないか無効な場合は 401 Unauthorized を返す。
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    // Bearerトークンが存在しない場合は401エラーを返す
    let Some(token) = extract_bearer_token(&req) else {
        return error_response(
            StatusCode::UNAUTHORIZED,
            "SYS_AUTH_MISSING_TOKEN",
            "Authorization header with Bearer token is required",
        );
    };

    match state.validate_token_uc.execute(&token).await {
        Ok(claims) => {
            // LOW-13 監査対応: ハードコードされた ["system","business","service"] の代わりに
            // AppState 経由で設定ファイルから注入された tier_hierarchy を使用する。
            if !has_tier_access(&state.tier_hierarchy, &claims.tier_access, "system") {
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
            // aud を Vec<String> で設定する（複数 audience 対応）
            aud: vec!["k1s0-api".to_string()],
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
            // aud を Vec<String> で設定する（複数 audience 対応）
            aud: vec!["k1s0-api".to_string()],
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
