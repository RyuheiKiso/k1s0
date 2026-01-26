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
