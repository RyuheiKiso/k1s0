//! REST API実装
//!
//! axumを使用したREST APIハンドラーを提供する。

use std::sync::Arc;

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

use crate::application::AuthService;
use crate::domain::{PermissionRepository, RoleRepository, TokenRepository, UserRepository};

/// REST APIの状態
pub struct RestState<U, R, P, T>
where
    U: UserRepository + 'static,
    R: RoleRepository + 'static,
    P: PermissionRepository + 'static,
    T: TokenRepository + 'static,
{
    pub service: Arc<AuthService<U, R, P, T>>,
}

impl<U, R, P, T> Clone for RestState<U, R, P, T>
where
    U: UserRepository + 'static,
    R: RoleRepository + 'static,
    P: PermissionRepository + 'static,
    T: TokenRepository + 'static,
{
    fn clone(&self) -> Self {
        Self {
            service: Arc::clone(&self.service),
        }
    }
}

/// ログインリクエスト
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub login_id: String,
    pub password: String,
}

/// ログインレスポンス
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// トークンリフレッシュリクエスト
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// パーミッションチェックリクエスト
#[derive(Debug, Deserialize)]
pub struct CheckPermissionRequest {
    pub user_id: i64,
    pub permission_key: String,
    #[serde(default)]
    pub service_name: Option<String>,
}

/// パーミッションチェックレスポンス
#[derive(Debug, Serialize)]
pub struct CheckPermissionResponse {
    pub allowed: bool,
}

/// ユーザー情報レスポンス
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub user_id: i64,
    pub login_id: String,
    pub email: String,
    pub display_name: String,
    pub status: String,
}

/// ロールレスポンス
#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub role_id: i64,
    pub role_name: String,
    pub description: String,
}

/// エラーレスポンス
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// REST APIルーターを作成
pub fn create_router<U, R, P, T>(service: Arc<AuthService<U, R, P, T>>) -> Router
where
    U: UserRepository + Send + Sync + 'static,
    R: RoleRepository + Send + Sync + 'static,
    P: PermissionRepository + Send + Sync + 'static,
    T: TokenRepository + Send + Sync + 'static,
{
    let state = RestState { service };

    Router::new()
        .route("/v1/login", post(login))
        .route("/v1/refresh", post(refresh_token))
        .route("/v1/check-permission", post(check_permission))
        .route("/v1/users/:user_id", get(get_user))
        .route("/v1/users/:user_id/roles", get(list_user_roles))
        .route("/health", get(health_check))
        .with_state(state)
}

/// ヘルスチェック
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

/// ログイン
async fn login<U, R, P, T>(
    State(state): State<RestState<U, R, P, T>>,
    Json(request): Json<LoginRequest>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    R: RoleRepository + Send + Sync + 'static,
    P: PermissionRepository + Send + Sync + 'static,
    T: TokenRepository + Send + Sync + 'static,
{
    match state
        .service
        .authenticate(&request.login_id, &request.password)
        .await
    {
        Ok(token) => (
            StatusCode::OK,
            Json(serde_json::to_value(LoginResponse {
                access_token: token.access_token,
                refresh_token: token.refresh_token,
                token_type: token.token_type,
                expires_in: token.expires_in,
            })
            .unwrap()),
        ),
        Err(e) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::to_value(ErrorResponse {
                error: "authentication_failed".to_string(),
                message: e.to_string(),
            })
            .unwrap()),
        ),
    }
}

/// トークンリフレッシュ
async fn refresh_token<U, R, P, T>(
    State(state): State<RestState<U, R, P, T>>,
    Json(request): Json<RefreshTokenRequest>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    R: RoleRepository + Send + Sync + 'static,
    P: PermissionRepository + Send + Sync + 'static,
    T: TokenRepository + Send + Sync + 'static,
{
    match state.service.refresh_token(&request.refresh_token).await {
        Ok(token) => (
            StatusCode::OK,
            Json(serde_json::to_value(LoginResponse {
                access_token: token.access_token,
                refresh_token: token.refresh_token,
                token_type: token.token_type,
                expires_in: token.expires_in,
            })
            .unwrap()),
        ),
        Err(e) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::to_value(ErrorResponse {
                error: "invalid_token".to_string(),
                message: e.to_string(),
            })
            .unwrap()),
        ),
    }
}

/// パーミッションチェック
async fn check_permission<U, R, P, T>(
    State(state): State<RestState<U, R, P, T>>,
    Json(request): Json<CheckPermissionRequest>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    R: RoleRepository + Send + Sync + 'static,
    P: PermissionRepository + Send + Sync + 'static,
    T: TokenRepository + Send + Sync + 'static,
{
    match state
        .service
        .check_permission(
            request.user_id,
            &request.permission_key,
            request.service_name.as_deref(),
        )
        .await
    {
        Ok(allowed) => (
            StatusCode::OK,
            Json(serde_json::to_value(CheckPermissionResponse { allowed }).unwrap()),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ErrorResponse {
                error: "permission_check_failed".to_string(),
                message: e.to_string(),
            })
            .unwrap()),
        ),
    }
}

/// ユーザー情報取得
async fn get_user<U, R, P, T>(
    State(state): State<RestState<U, R, P, T>>,
    Path(user_id): Path<i64>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    R: RoleRepository + Send + Sync + 'static,
    P: PermissionRepository + Send + Sync + 'static,
    T: TokenRepository + Send + Sync + 'static,
{
    match state.service.get_user(user_id).await {
        Ok(user) => (
            StatusCode::OK,
            Json(serde_json::to_value(UserResponse {
                user_id: user.user_id,
                login_id: user.login_id,
                email: user.email,
                display_name: user.display_name,
                status: format!("{:?}", user.status),
            })
            .unwrap()),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::to_value(ErrorResponse {
                error: "user_not_found".to_string(),
                message: e.to_string(),
            })
            .unwrap()),
        ),
    }
}

/// ユーザーロール一覧取得
async fn list_user_roles<U, R, P, T>(
    State(state): State<RestState<U, R, P, T>>,
    Path(user_id): Path<i64>,
) -> impl IntoResponse
where
    U: UserRepository + Send + Sync + 'static,
    R: RoleRepository + Send + Sync + 'static,
    P: PermissionRepository + Send + Sync + 'static,
    T: TokenRepository + Send + Sync + 'static,
{
    match state.service.list_user_roles(user_id).await {
        Ok(roles) => {
            let roles: Vec<RoleResponse> = roles
                .into_iter()
                .map(|r| RoleResponse {
                    role_id: r.role_id,
                    role_name: r.role_name,
                    description: r.description,
                })
                .collect();
            (StatusCode::OK, Json(serde_json::to_value(roles).unwrap()))
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::to_value(ErrorResponse {
                error: "user_not_found".to_string(),
                message: e.to_string(),
            })
            .unwrap()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::AuthService;
    use crate::domain::{Role, User};
    use crate::infrastructure::{
        InMemoryPermissionRepository, InMemoryRoleRepository, InMemoryTokenRepository,
        InMemoryUserRepository,
    };
    use axum::{
        body::Body,
        http::{Request as HttpRequest, StatusCode as AxumStatusCode},
    };
    use tower::ServiceExt;

    // ========================================
    // Helper Functions
    // ========================================

    type TestUserRepo = InMemoryUserRepository;
    type TestRoleRepo = InMemoryRoleRepository;
    type TestPermRepo = InMemoryPermissionRepository;
    type TestTokenRepo = InMemoryTokenRepository;
    type TestAuthService = AuthService<TestUserRepo, TestRoleRepo, TestPermRepo, TestTokenRepo>;

    fn create_test_router() -> Router {
        let auth_service = AuthService::new(
            Arc::new(InMemoryUserRepository::new()),
            Arc::new(InMemoryRoleRepository::new()),
            Arc::new(InMemoryPermissionRepository::new()),
            Arc::new(InMemoryTokenRepository::new()),
            "k1s0-test",
            "secret123",
        );
        create_router(Arc::new(auth_service))
    }

    fn create_test_service() -> Arc<TestAuthService> {
        Arc::new(AuthService::new(
            Arc::new(InMemoryUserRepository::new()),
            Arc::new(InMemoryRoleRepository::new()),
            Arc::new(InMemoryPermissionRepository::new()),
            Arc::new(InMemoryTokenRepository::new()),
            "k1s0-test",
            "secret123",
        ))
    }

    // ========================================
    // Request/Response Struct Tests
    // ========================================

    #[test]
    fn test_login_request_deserialize() {
        let json = r#"{"login_id": "test", "password": "pass123"}"#;
        let request: LoginRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.login_id, "test");
        assert_eq!(request.password, "pass123");
    }

    #[test]
    fn test_login_response_serialize() {
        let response = LoginResponse {
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("access_token"));
        assert!(json.contains("refresh_token"));
        assert!(json.contains("Bearer"));
    }

    #[test]
    fn test_refresh_token_request_deserialize() {
        let json = r#"{"refresh_token": "token123"}"#;
        let request: RefreshTokenRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.refresh_token, "token123");
    }

    #[test]
    fn test_check_permission_request_deserialize() {
        let json = r#"{"user_id": 1, "permission_key": "user:read"}"#;
        let request: CheckPermissionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.user_id, 1);
        assert_eq!(request.permission_key, "user:read");
        assert!(request.service_name.is_none());
    }

    #[test]
    fn test_check_permission_request_with_service_name() {
        let json = r#"{"user_id": 1, "permission_key": "user:read", "service_name": "auth-svc"}"#;
        let request: CheckPermissionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.service_name, Some("auth-svc".to_string()));
    }

    #[test]
    fn test_user_response_serialize() {
        let response = UserResponse {
            user_id: 1,
            login_id: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            status: "Active".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("testuser"));
        assert!(json.contains("test@example.com"));
    }

    #[test]
    fn test_role_response_serialize() {
        let response = RoleResponse {
            role_id: 1,
            role_name: "admin".to_string(),
            description: "Administrator".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("admin"));
    }

    #[test]
    fn test_error_response_serialize() {
        let response = ErrorResponse {
            error: "test_error".to_string(),
            message: "Test message".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test_error"));
        assert!(json.contains("Test message"));
    }

    // ========================================
    // RestState Tests
    // ========================================

    #[test]
    fn test_rest_state_clone() {
        let service = create_test_service();
        let state = RestState { service };
        let cloned = state.clone();
        // Both should point to the same Arc
        assert!(Arc::ptr_eq(&state.service, &cloned.service));
    }

    // ========================================
    // Router Creation Tests
    // ========================================

    #[test]
    fn test_create_router() {
        let _router = create_test_router();
        // Router should be created without panicking
    }

    // ========================================
    // Health Check Tests
    // ========================================

    #[tokio::test]
    async fn test_health_check() {
        let router = create_test_router();

        let request = HttpRequest::builder()
            .uri("/health")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), AxumStatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
    }

    // ========================================
    // Login Tests
    // ========================================

    #[tokio::test]
    async fn test_login_success() {
        let service = create_test_service();

        // Add user
        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash:password123");
        service.user_repo.save(&user).await.unwrap();

        let router = create_router(service);

        let request = HttpRequest::builder()
            .uri("/v1/login")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"login_id": "testuser", "password": "password123"}"#))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), AxumStatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(!json["access_token"].as_str().unwrap().is_empty());
        assert_eq!(json["token_type"], "Bearer");
    }

    #[tokio::test]
    async fn test_login_invalid_credentials() {
        let service = create_test_service();

        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash:password123");
        service.user_repo.save(&user).await.unwrap();

        let router = create_router(service);

        let request = HttpRequest::builder()
            .uri("/v1/login")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"login_id": "testuser", "password": "wrongpassword"}"#))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), AxumStatusCode::UNAUTHORIZED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "authentication_failed");
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
        let router = create_test_router();

        let request = HttpRequest::builder()
            .uri("/v1/login")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"login_id": "nonexistent", "password": "password"}"#))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), AxumStatusCode::UNAUTHORIZED);
    }

    // ========================================
    // Refresh Token Tests
    // ========================================

    #[tokio::test]
    async fn test_refresh_token_success() {
        let service = create_test_service();

        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash:password123");
        service.user_repo.save(&user).await.unwrap();

        // First login to get a refresh token
        let auth_token = service.authenticate("testuser", "password123").await.unwrap();

        let router = create_router(service);

        let request = HttpRequest::builder()
            .uri("/v1/refresh")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(format!(r#"{{"refresh_token": "{}"}}"#, auth_token.refresh_token)))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), AxumStatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(!json["access_token"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_refresh_token_invalid() {
        let router = create_test_router();

        let request = HttpRequest::builder()
            .uri("/v1/refresh")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"refresh_token": "invalid_token"}"#))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), AxumStatusCode::UNAUTHORIZED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "invalid_token");
    }

    // ========================================
    // Check Permission Tests
    // ========================================

    #[tokio::test]
    async fn test_check_permission_allowed() {
        let service = create_test_service();

        let user = User::new(1, "admin", "admin@example.com", "Admin", "hash:password");
        service.user_repo.save(&user).await.unwrap();
        service.permission_repo.add_permission(1, "user:read", None);

        let router = create_router(service);

        let request = HttpRequest::builder()
            .uri("/v1/check-permission")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"user_id": 1, "permission_key": "user:read"}"#))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), AxumStatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["allowed"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_check_permission_denied() {
        let service = create_test_service();

        let user = User::new(1, "user", "user@example.com", "User", "hash:password");
        service.user_repo.save(&user).await.unwrap();

        let router = create_router(service);

        let request = HttpRequest::builder()
            .uri("/v1/check-permission")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"user_id": 1, "permission_key": "admin:all"}"#))
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), AxumStatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(!json["allowed"].as_bool().unwrap());
    }

    // ========================================
    // Get User Tests
    // ========================================

    #[tokio::test]
    async fn test_get_user_success() {
        let service = create_test_service();

        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash:password");
        service.user_repo.save(&user).await.unwrap();

        let router = create_router(service);

        let request = HttpRequest::builder()
            .uri("/v1/users/1")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), AxumStatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["user_id"], 1);
        assert_eq!(json["login_id"], "testuser");
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let router = create_test_router();

        let request = HttpRequest::builder()
            .uri("/v1/users/999")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), AxumStatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "user_not_found");
    }

    // ========================================
    // List User Roles Tests
    // ========================================

    #[tokio::test]
    async fn test_list_user_roles_success() {
        let service = create_test_service();

        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash:password");
        service.user_repo.save(&user).await.unwrap();

        let admin_role = Role::new(1, "admin", "Administrator");
        service.role_repo.add_role(admin_role);
        service.role_repo.assign_role(1, 1).await.unwrap();

        let router = create_router(service);

        let request = HttpRequest::builder()
            .uri("/v1/users/1/roles")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), AxumStatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_list_user_roles_empty() {
        let service = create_test_service();

        let user = User::new(1, "testuser", "test@example.com", "Test User", "hash:password");
        service.user_repo.save(&user).await.unwrap();

        let router = create_router(service);

        let request = HttpRequest::builder()
            .uri("/v1/users/1/roles")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), AxumStatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.is_array());
        assert!(json.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_list_user_roles_user_not_found() {
        let router = create_test_router();

        let request = HttpRequest::builder()
            .uri("/v1/users/999/roles")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), AxumStatusCode::NOT_FOUND);
    }
}
