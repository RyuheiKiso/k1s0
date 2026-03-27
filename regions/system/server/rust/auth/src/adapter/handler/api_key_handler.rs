use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use super::{AppState, ErrorResponse};
use crate::domain::entity::api_key::CreateApiKeyRequest;
use crate::usecase::validate_api_key::ValidateApiKeyError;

/// POST /api/v1/api-keys のリクエストボディ。
/// SEC-008: 入力バリデーションを追加し、不正な値の受け入れを防止する。
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateApiKeyHttpRequest {
    /// テナント ID（1〜128 文字）
    #[validate(length(min = 1, max = 128))]
    pub tenant_id: String,
    /// API キー名（1〜256 文字）
    #[validate(length(min = 1, max = 256))]
    pub name: String,
    /// スコープ一覧（最大 50 個）
    #[serde(default)]
    #[validate(length(max = 50))]
    pub scopes: Vec<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// GET /api/v1/api-keys のクエリパラメータ。
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListApiKeysQuery {
    pub tenant_id: String,
}

/// POST /api/v1/api-keys/validate のリクエストボディ。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ValidateApiKeyRequest {
    pub api_key: String,
}

/// POST /api/v1/api-keys/validate のレスポンスボディ。
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ValidateApiKeyResponse {
    pub valid: bool,
    pub tenant_id: Option<String>,
    pub name: Option<String>,
    pub scopes: Vec<String>,
    pub reason: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/api-keys",
    request_body = CreateApiKeyHttpRequest,
    responses(
        (status = 201, description = "API key created"),
        (status = 400, description = "Validation error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_api_key(
    State(state): State<AppState>,
    Json(req): Json<CreateApiKeyHttpRequest>,
) -> impl IntoResponse {
    // SEC-008: リクエストボディのバリデーションを実行し、不正な入力を早期に拒否する
    if let Err(errors) = req.validate() {
        let err = ErrorResponse::new("SYS_AUTH_API_KEY_VALIDATION", errors.to_string());
        return (StatusCode::BAD_REQUEST, Json(err)).into_response();
    }

    let create_req = CreateApiKeyRequest {
        tenant_id: req.tenant_id,
        name: req.name,
        scopes: req.scopes,
        expires_at: req.expires_at,
    };

    match state.create_api_key_uc.execute(create_req).await {
        Ok(resp) => (StatusCode::CREATED, Json(resp)).into_response(),
        Err(crate::usecase::create_api_key::CreateApiKeyError::Validation(msg)) => {
            let err = ErrorResponse::new("SYS_AUTH_API_KEY_VALIDATION", &msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_AUTH_INTERNAL_ERROR", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/api-keys/{id}",
    params(
        ("id" = String, Path, description = "API key ID")
    ),
    responses(
        (status = 200, description = "API key found"),
        (status = 404, description = "API key not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_api_key(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match state.get_api_key_uc.execute(id).await {
        Ok(summary) => (StatusCode::OK, Json(summary)).into_response(),
        Err(crate::usecase::get_api_key::GetApiKeyError::NotFound(_)) => {
            let err = ErrorResponse::new(
                "SYS_AUTH_API_KEY_NOT_FOUND",
                "The specified API key was not found",
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_AUTH_INTERNAL_ERROR", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/api-keys",
    params(ListApiKeysQuery),
    responses(
        (status = 200, description = "API key list"),
        (status = 400, description = "Validation error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_api_keys(
    State(state): State<AppState>,
    Query(query): Query<ListApiKeysQuery>,
) -> impl IntoResponse {
    match state.list_api_keys_uc.execute(&query.tenant_id).await {
        Ok(keys) => (StatusCode::OK, Json(keys)).into_response(),
        Err(crate::usecase::list_api_keys::ListApiKeysError::Validation(msg)) => {
            let err = ErrorResponse::new("SYS_AUTH_API_KEY_VALIDATION", &msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_AUTH_INTERNAL_ERROR", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/api-keys/{id}/revoke",
    params(
        ("id" = String, Path, description = "API key ID")
    ),
    responses(
        (status = 204, description = "API key revoked"),
        (status = 404, description = "API key not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn revoke_api_key(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.revoke_api_key_uc.execute(id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(crate::usecase::revoke_api_key::RevokeApiKeyError::NotFound(_)) => {
            let err = ErrorResponse::new(
                "SYS_AUTH_API_KEY_NOT_FOUND",
                "The specified API key was not found",
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_AUTH_INTERNAL_ERROR", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// SEC-001: Bearer トークン認証を必須化（サービス間検証用、RBAC 不要）
#[utoipa::path(
    post,
    path = "/api/v1/api-keys/validate",
    request_body = ValidateApiKeyRequest,
    responses(
        (status = 200, description = "API key validation result", body = ValidateApiKeyResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn validate_api_key(
    State(state): State<AppState>,
    Json(req): Json<ValidateApiKeyRequest>,
) -> impl IntoResponse {
    match state.validate_api_key_uc.execute(&req.api_key).await {
        Ok(result) => (
            StatusCode::OK,
            Json(ValidateApiKeyResponse {
                valid: true,
                tenant_id: Some(result.tenant_id),
                name: Some(result.name),
                scopes: result.scopes,
                reason: None,
            }),
        )
            .into_response(),
        Err(ValidateApiKeyError::Invalid) => (
            StatusCode::OK,
            Json(ValidateApiKeyResponse {
                valid: false,
                tenant_id: None,
                name: None,
                scopes: vec![],
                reason: Some("invalid".to_string()),
            }),
        )
            .into_response(),
        Err(ValidateApiKeyError::Revoked) => (
            StatusCode::OK,
            Json(ValidateApiKeyResponse {
                valid: false,
                tenant_id: None,
                name: None,
                scopes: vec![],
                reason: Some("revoked".to_string()),
            }),
        )
            .into_response(),
        Err(ValidateApiKeyError::Expired) => (
            StatusCode::OK,
            Json(ValidateApiKeyResponse {
                valid: false,
                tenant_id: None,
                name: None,
                scopes: vec![],
                reason: Some("expired".to_string()),
            }),
        )
            .into_response(),
        Err(ValidateApiKeyError::Internal(msg)) => {
            let err = ErrorResponse::new("SYS_AUTH_INTERNAL_ERROR", &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
        // ペッパーが未設定の場合はサーバー設定エラーとして 500 を返す
        Err(ValidateApiKeyError::PepperNotConfigured) => {
            let err = ErrorResponse::new(
                "SYS_AUTH_PEPPER_NOT_CONFIGURED",
                "server configuration error",
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}
