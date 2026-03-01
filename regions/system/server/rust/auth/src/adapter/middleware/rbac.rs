use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};

use crate::adapter::handler::AppState;
use crate::domain::entity::claims::Claims;
use crate::domain::service::AuthDomainService;

/// rbac_middleware は Request extension の Claims からロールを取得し、
/// AuthDomainService を使って指定リソース・アクションのパーミッションを確認する axum ミドルウェア。
///
/// Claims が extension に存在しない場合は 401 Unauthorized を返す。
/// パーミッションが不足する場合は 403 Forbidden を返す。
///
/// # 引数
/// - `resource`: チェック対象のリソース名
/// - `action`  : チェック対象のアクション名 ("read" / "write" / "delete" / "admin")
pub fn make_rbac_middleware(
    resource: &'static str,
    action: &'static str,
) -> impl Fn(
    State<AppState>,
    Request<axum::body::Body>,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
       + Clone {
    move |_state: State<AppState>, req: Request<axum::body::Body>, next: Next| {
        Box::pin(rbac_check(req, next, resource, action))
    }
}

/// Core RBAC check logic. Called from make_rbac_middleware.
pub async fn rbac_check(
    req: Request<axum::body::Body>,
    next: Next,
    resource: &str,
    action: &str,
) -> Response {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": {
                        "code": "SYS_AUTH_MISSING_CLAIMS",
                        "message": "Authentication is required. Please provide a valid Bearer token."
                    }
                })),
            )
                .into_response();
        }
    };

    let roles: Vec<String> = claims.realm_access.roles.clone();

    if AuthDomainService::check_permission(&roles, resource, action) {
        next.run(req).await
    } else {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": {
                    "code": "SYS_AUTH_PERMISSION_DENIED",
                    "message": format!(
                        "Insufficient permissions: action '{}' on resource '{}' is not allowed for the current roles.",
                        action, resource
                    )
                }
            })),
        )
            .into_response()
    }
}

/// rbac_middleware は resource・action を受け取り、Claims からパーミッションを確認する。
/// State から AppState を受け取る axum middleware::from_fn_with_state 向けの関数。
pub async fn rbac_middleware(
    State(_state): State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let claims = match req.extensions().get::<Claims>() {
        Some(c) => c.clone(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": {
                        "code": "SYS_AUTH_MISSING_CLAIMS",
                        "message": "Authentication is required. Please provide a valid Bearer token."
                    }
                })),
            )
                .into_response();
        }
    };

    // デフォルトのチェック: sys_auditor 以上であれば通過する
    let roles: Vec<String> = claims.realm_access.roles.clone();
    if AuthDomainService::is_auditor_or_above(&roles) {
        next.run(req).await
    } else {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": {
                    "code": "SYS_AUTH_PERMISSION_DENIED",
                    "message": "Insufficient permissions for the requested resource."
                }
            })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::claims::{Claims, RealmAccess};
    use axum::body::Body;
    use axum::http::Request;
    use std::collections::HashMap;

    fn make_claims_with_roles(role_names: &[&str]) -> Claims {
        Claims {
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
                roles: role_names.iter().map(|s| s.to_string()).collect(),
            },
            resource_access: HashMap::new(),
            tier_access: vec![],
        }
    }

    fn make_request_with_claims(claims: Claims) -> Request<Body> {
        let mut req = Request::builder()
            .uri("/protected")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut().insert(claims);
        req
    }

    fn make_request_without_claims() -> Request<Body> {
        Request::builder()
            .uri("/protected")
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn test_rbac_middleware_missing_claims_returns_401() {
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
                rbac_middleware,
            ))
            .with_state(state);

        // Claims がない状態でリクエスト
        let req = make_request_without_claims();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "SYS_AUTH_MISSING_CLAIMS");
    }

    #[tokio::test]
    async fn test_rbac_middleware_auditor_role_allowed() {
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
                rbac_middleware,
            ))
            .with_state(state);

        let claims = make_claims_with_roles(&["sys_auditor"]);
        let req = make_request_with_claims(claims);
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rbac_middleware_unknown_role_forbidden() {
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
                rbac_middleware,
            ))
            .with_state(state);

        let claims = make_claims_with_roles(&["user", "viewer"]);
        let req = make_request_with_claims(claims);
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "SYS_AUTH_PERMISSION_DENIED");
    }

    #[tokio::test]
    async fn test_rbac_check_admin_allowed_delete() {
        use crate::domain::repository::audit_log_repository::MockAuditLogRepository;
        use crate::domain::repository::user_repository::MockUserRepository;
        use crate::infrastructure::MockTokenVerifier;
        use axum::middleware;
        use axum::routing::delete;
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

        let state_clone = state.clone();
        let app = Router::new()
            .route(
                "/api/v1/users/:id",
                delete(|| async { StatusCode::NO_CONTENT }).layer(middleware::from_fn_with_state(
                    state_clone,
                    |_s: State<AppState>, req: Request<Body>, next: Next| {
                        rbac_check(req, next, "users", "delete")
                    },
                )),
            )
            .with_state(state);

        let claims = make_claims_with_roles(&["sys_admin"]);
        let mut req = Request::builder()
            .method("DELETE")
            .uri("/api/v1/users/user-uuid-1234")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut().insert(claims);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_rbac_check_operator_delete_forbidden() {
        use crate::domain::repository::audit_log_repository::MockAuditLogRepository;
        use crate::domain::repository::user_repository::MockUserRepository;
        use crate::infrastructure::MockTokenVerifier;
        use axum::middleware;
        use axum::routing::delete;
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

        let state_clone = state.clone();
        let app = Router::new()
            .route(
                "/api/v1/users/:id",
                delete(|| async { StatusCode::NO_CONTENT }).layer(middleware::from_fn_with_state(
                    state_clone,
                    |_s: State<AppState>, req: Request<Body>, next: Next| {
                        rbac_check(req, next, "users", "delete")
                    },
                )),
            )
            .with_state(state);

        let claims = make_claims_with_roles(&["sys_operator"]);
        let mut req = Request::builder()
            .method("DELETE")
            .uri("/api/v1/users/user-uuid-1234")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut().insert(claims);

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "SYS_AUTH_PERMISSION_DENIED");
    }
}
