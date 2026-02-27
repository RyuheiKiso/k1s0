use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use super::{AppState, ErrorResponse};
use crate::domain::entity::user::{User, UserListResult, UserRoles};
use crate::usecase::list_users::ListUsersParams;

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Health check OK"),
    )
)]
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

#[utoipa::path(
    get,
    path = "/readyz",
    responses(
        (status = 200, description = "Ready"),
        (status = 503, description = "Not ready"),
    )
)]
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let mut db_status = "skipped";
    let mut kc_status = "skipped";
    let mut overall_ok = true;

    // DB check
    if let Some(ref pool) = state.db_pool {
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => db_status = "ok",
            Err(_) => {
                db_status = "error";
                overall_ok = false;
            }
        }
    }

    // Keycloak check
    if let Some(ref url) = state.keycloak_url {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap_or_default();
        match client.get(url).send().await {
            Ok(_) => kc_status = "ok",
            Err(_) => {
                kc_status = "error";
                overall_ok = false;
            }
        }
    }

    let status_code = if overall_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status_code,
        Json(serde_json::json!({
            "status": if overall_ok { "ready" } else { "not ready" },
            "checks": {
                "database": db_status,
                "keycloak": kc_status
            }
        })),
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics"),
    )
)]
pub async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

/// POST /api/v1/auth/token/validate のリクエストボディ。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ValidateTokenRequest {
    pub token: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/token/validate",
    request_body = ValidateTokenRequest,
    responses(
        (status = 200, description = "Token is valid"),
        (status = 401, description = "Token is invalid"),
    )
)]
pub async fn validate_token(
    State(state): State<AppState>,
    Json(req): Json<ValidateTokenRequest>,
) -> impl IntoResponse {
    match state.validate_token_uc.execute(&req.token).await {
        Ok(claims) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "valid": true,
                "claims": claims
            })),
        )
            .into_response(),
        Err(_) => {
            let err = ErrorResponse::new("SYS_AUTH_TOKEN_INVALID", "Token validation failed");
            (StatusCode::UNAUTHORIZED, Json(err)).into_response()
        }
    }
}

/// POST /api/v1/auth/token/introspect のリクエストボディ。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct IntrospectTokenRequest {
    pub token: String,
    #[serde(default)]
    pub token_type_hint: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/token/introspect",
    request_body = IntrospectTokenRequest,
    responses(
        (status = 200, description = "Token introspection result"),
    )
)]
pub async fn introspect_token(
    State(state): State<AppState>,
    Json(req): Json<IntrospectTokenRequest>,
) -> impl IntoResponse {
    match state.validate_token_uc.execute(&req.token).await {
        Ok(claims) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "active": true,
                "sub": claims.sub,
                "client_id": claims.azp,
                "username": claims.preferred_username,
                "token_type": "Bearer",
                "exp": claims.exp,
                "iat": claims.iat,
                "scope": claims.scope,
                "realm_access": claims.realm_access
            })),
        )
            .into_response(),
        Err(_) => (StatusCode::OK, Json(serde_json::json!({"active": false}))).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    params(("id" = String, Path, description = "User ID")),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_user(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.get_user_uc.execute(&id).await {
        Ok(user) => (StatusCode::OK, Json(serde_json::to_value(user).unwrap())).into_response(),
        Err(_) => {
            let err = ErrorResponse::new(
                "SYS_AUTH_USER_NOT_FOUND",
                "The specified user was not found",
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    params(
        ("page" = Option<i32>, Query, description = "Page number"),
        ("page_size" = Option<i32>, Query, description = "Page size"),
    ),
    responses(
        (status = 200, description = "User list", body = UserListResult),
        (status = 400, description = "Bad request"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<ListUsersParams>,
) -> impl IntoResponse {
    match state.list_users_uc.execute(&params).await {
        Ok(result) => (StatusCode::OK, Json(serde_json::to_value(result).unwrap())).into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_AUTH_LIST_USERS_FAILED", &e.to_string());
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}

/// POST /api/v1/auth/permissions/check のリクエストボディ。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CheckPermissionRequest {
    pub roles: Vec<String>,
    pub permission: String,
    pub resource: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/permissions/check",
    request_body = CheckPermissionRequest,
    responses(
        (status = 200, description = "Permission check result"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn check_permission(
    State(state): State<AppState>,
    Json(req): Json<CheckPermissionRequest>,
) -> impl IntoResponse {
    let input = crate::usecase::check_permission::CheckPermissionInput {
        roles: req.roles,
        permission: req.permission,
        resource: req.resource,
    };
    let output = state.check_permission_uc.execute(&input);
    (StatusCode::OK, Json(output)).into_response()
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}/roles",
    params(("id" = String, Path, description = "User ID")),
    responses(
        (status = 200, description = "User roles", body = UserRoles),
        (status = 404, description = "User not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_user_roles(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.get_user_roles_uc.execute(&id).await {
        Ok(roles) => (StatusCode::OK, Json(serde_json::to_value(roles).unwrap())).into_response(),
        Err(_) => {
            let err = ErrorResponse::new(
                "SYS_AUTH_USER_NOT_FOUND",
                "The specified user was not found",
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::handler::router;
    use crate::domain::entity::claims::{Claims, RealmAccess};
    use crate::domain::entity::user::{Pagination, Role, User, UserListResult, UserRoles};
    use crate::domain::repository::audit_log_repository::MockAuditLogRepository;
    use crate::domain::repository::user_repository::MockUserRepository;
    use crate::infrastructure::MockTokenVerifier;
    use axum::body::Body;
    use axum::http::Request;
    use std::collections::HashMap;
    use tower::ServiceExt;

    fn make_valid_claims() -> Claims {
        Claims {
            sub: "user-uuid-1234".to_string(),
            iss: "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
            aud: "k1s0-api".to_string(),
            exp: chrono::Utc::now().timestamp() + 3600,
            iat: chrono::Utc::now().timestamp(),
            jti: "token-uuid-5678".to_string(),
            typ: "Bearer".to_string(),
            azp: "react-spa".to_string(),
            scope: "openid profile email".to_string(),
            preferred_username: "taro.yamada".to_string(),
            email: "taro.yamada@example.com".to_string(),
            realm_access: RealmAccess {
                roles: vec!["sys_auditor".to_string()],
            },
            resource_access: HashMap::new(),
            tier_access: vec!["system".to_string()],
        }
    }

    fn make_app_state(
        token_verifier: MockTokenVerifier,
        user_repo: MockUserRepository,
        audit_repo: MockAuditLogRepository,
    ) -> AppState {
        use crate::domain::repository::api_key_repository::MockApiKeyRepository;
        AppState::new(
            Arc::new(token_verifier),
            Arc::new(user_repo),
            Arc::new(audit_repo),
            Arc::new(MockApiKeyRepository::new()),
            "https://auth.k1s0.internal.example.com/realms/k1s0".to_string(),
            "k1s0-api".to_string(),
            None,
            None,
        )
    }

    use std::sync::Arc;

    #[tokio::test]
    async fn test_healthz() {
        let state = make_app_state(
            MockTokenVerifier::new(),
            MockUserRepository::new(),
            MockAuditLogRepository::new(),
        );
        let app = router(state);

        let req = Request::builder()
            .uri("/healthz")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn test_readyz() {
        let state = make_app_state(
            MockTokenVerifier::new(),
            MockUserRepository::new(),
            MockAuditLogRepository::new(),
        );
        let app = router(state);

        let req = Request::builder()
            .uri("/readyz")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ready");
    }

    #[tokio::test]
    async fn test_validate_token_success() {
        let mut token_verifier = MockTokenVerifier::new();
        let claims = make_valid_claims();
        let return_claims = claims.clone();
        token_verifier
            .expect_verify_token()
            .returning(move |_| Ok(return_claims.clone()));

        let state = make_app_state(
            token_verifier,
            MockUserRepository::new(),
            MockAuditLogRepository::new(),
        );
        let app = router(state);

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/token/validate")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"token":"valid-jwt-token"}"#))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["valid"], true);
        assert_eq!(json["claims"]["sub"], "user-uuid-1234");
    }

    #[tokio::test]
    async fn test_validate_token_invalid() {
        let mut token_verifier = MockTokenVerifier::new();
        token_verifier
            .expect_verify_token()
            .returning(|_| Err(anyhow::anyhow!("invalid signature")));

        let state = make_app_state(
            token_verifier,
            MockUserRepository::new(),
            MockAuditLogRepository::new(),
        );
        let app = router(state);

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/token/validate")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"token":"invalid-token"}"#))
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
    async fn test_introspect_token_active() {
        let mut token_verifier = MockTokenVerifier::new();
        let claims = make_valid_claims();
        let return_claims = claims.clone();
        // introspect はpublicエンドポイントのため verify_token は本文トークンに対して1回のみ呼ばれる
        token_verifier
            .expect_verify_token()
            .returning(move |_| Ok(return_claims.clone()));

        let state = make_app_state(
            token_verifier,
            MockUserRepository::new(),
            MockAuditLogRepository::new(),
        );
        let app = router(state);

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/token/introspect")
            .header("content-type", "application/json")
            .header("Authorization", "Bearer valid-jwt-token")
            .body(Body::from(
                r#"{"token":"valid-jwt-token","token_type_hint":"access_token"}"#,
            ))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["active"], true);
        assert_eq!(json["sub"], "user-uuid-1234");
        assert_eq!(json["username"], "taro.yamada");
    }

    #[tokio::test]
    async fn test_introspect_token_inactive() {
        let mut token_verifier = MockTokenVerifier::new();
        // introspect はpublicエンドポイント (RFC 7662) のため auth_middleware は不要
        // リクエストボディのトークン検証が失敗すると active: false を返す
        token_verifier
            .expect_verify_token()
            .returning(|_| Err(anyhow::anyhow!("invalid")));

        let state = make_app_state(
            token_verifier,
            MockUserRepository::new(),
            MockAuditLogRepository::new(),
        );
        let app = router(state);

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/token/introspect")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"token":"expired-token"}"#))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["active"], false);
    }

    #[tokio::test]
    async fn test_get_user_success() {
        let mut token_verifier = MockTokenVerifier::new();
        let claims = make_valid_claims();
        token_verifier
            .expect_verify_token()
            .returning(move |_| Ok(claims.clone()));

        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_find_by_id()
            .withf(|id| id == "user-uuid-1234")
            .returning(|_| {
                Ok(User {
                    id: "user-uuid-1234".to_string(),
                    username: "taro.yamada".to_string(),
                    email: "taro.yamada@example.com".to_string(),
                    first_name: "Taro".to_string(),
                    last_name: "Yamada".to_string(),
                    enabled: true,
                    email_verified: true,
                    created_at: chrono::Utc::now(),
                    attributes: HashMap::new(),
                })
            });

        let state = make_app_state(token_verifier, user_repo, MockAuditLogRepository::new());
        let app = router(state);

        let req = Request::builder()
            .uri("/api/v1/users/user-uuid-1234")
            .header("Authorization", "Bearer valid-token")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["id"], "user-uuid-1234");
        assert_eq!(json["username"], "taro.yamada");
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let mut token_verifier = MockTokenVerifier::new();
        let claims = make_valid_claims();
        token_verifier
            .expect_verify_token()
            .returning(move |_| Ok(claims.clone()));

        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("user not found")));

        let state = make_app_state(token_verifier, user_repo, MockAuditLogRepository::new());
        let app = router(state);

        let req = Request::builder()
            .uri("/api/v1/users/nonexistent")
            .header("Authorization", "Bearer valid-token")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"]["code"], "SYS_AUTH_USER_NOT_FOUND");
    }

    #[tokio::test]
    async fn test_list_users_success() {
        let mut token_verifier = MockTokenVerifier::new();
        let claims = make_valid_claims();
        token_verifier
            .expect_verify_token()
            .returning(move |_| Ok(claims.clone()));

        let mut user_repo = MockUserRepository::new();
        user_repo.expect_list().returning(|page, page_size, _, _| {
            Ok(UserListResult {
                users: vec![User {
                    id: "user-1".to_string(),
                    username: "taro.yamada".to_string(),
                    email: "taro@example.com".to_string(),
                    first_name: "Taro".to_string(),
                    last_name: "Yamada".to_string(),
                    enabled: true,
                    email_verified: true,
                    created_at: chrono::Utc::now(),
                    attributes: HashMap::new(),
                }],
                pagination: Pagination {
                    total_count: 1,
                    page,
                    page_size,
                    has_next: false,
                },
            })
        });

        let state = make_app_state(token_verifier, user_repo, MockAuditLogRepository::new());
        let app = router(state);

        let req = Request::builder()
            .uri("/api/v1/users?page=1&page_size=20")
            .header("Authorization", "Bearer valid-token")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["users"].as_array().unwrap().len(), 1);
        assert_eq!(json["pagination"]["total_count"], 1);
    }

    #[tokio::test]
    async fn test_get_user_roles_success() {
        let mut token_verifier = MockTokenVerifier::new();
        let claims = make_valid_claims();
        token_verifier
            .expect_verify_token()
            .returning(move |_| Ok(claims.clone()));

        let mut user_repo = MockUserRepository::new();
        user_repo
            .expect_get_roles()
            .withf(|id| id == "user-uuid-1234")
            .returning(|_| {
                Ok(UserRoles {
                    user_id: "user-uuid-1234".to_string(),
                    realm_roles: vec![Role {
                        id: "role-1".to_string(),
                        name: "user".to_string(),
                        description: "General user".to_string(),
                    }],
                    client_roles: HashMap::new(),
                })
            });

        let state = make_app_state(token_verifier, user_repo, MockAuditLogRepository::new());
        let app = router(state);

        let req = Request::builder()
            .uri("/api/v1/users/user-uuid-1234/roles")
            .header("Authorization", "Bearer valid-token")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["user_id"], "user-uuid-1234");
        assert_eq!(json["realm_roles"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let state = make_app_state(
            MockTokenVerifier::new(),
            MockUserRepository::new(),
            MockAuditLogRepository::new(),
        );
        let app = router(state);

        let req = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
