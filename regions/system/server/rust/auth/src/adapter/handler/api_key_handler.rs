use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use super::{AppState, ErrorResponse};
use crate::domain::entity::api_key::CreateApiKeyRequest;

/// POST /api/v1/api-keys のリクエストボディ。
#[derive(Debug, Deserialize)]
pub struct CreateApiKeyHttpRequest {
    pub tenant_id: String,
    pub name: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// GET /api/v1/api-keys のクエリパラメータ。
#[derive(Debug, Deserialize)]
pub struct ListApiKeysQuery {
    pub tenant_id: String,
}

pub async fn create_api_key(
    State(state): State<AppState>,
    Json(req): Json<CreateApiKeyHttpRequest>,
) -> impl IntoResponse {
    let create_req = CreateApiKeyRequest {
        tenant_id: req.tenant_id,
        name: req.name,
        scopes: req.scopes,
        expires_at: req.expires_at,
    };

    match state.create_api_key_uc.execute(create_req).await {
        Ok(resp) => (
            StatusCode::CREATED,
            Json(serde_json::to_value(resp).unwrap()),
        )
            .into_response(),
        Err(crate::usecase::create_api_key::CreateApiKeyError::Validation(msg)) => {
            let err = ErrorResponse::new("SYS_AUTH_API_KEY_VALIDATION", &msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_AUTH_API_KEY_CREATE_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn get_api_key(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.get_api_key_uc.execute(id).await {
        Ok(summary) => (
            StatusCode::OK,
            Json(serde_json::to_value(summary).unwrap()),
        )
            .into_response(),
        Err(crate::usecase::get_api_key::GetApiKeyError::NotFound(_)) => {
            let err = ErrorResponse::new(
                "SYS_AUTH_API_KEY_NOT_FOUND",
                "The specified API key was not found",
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_AUTH_API_KEY_GET_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn list_api_keys(
    State(state): State<AppState>,
    Query(query): Query<ListApiKeysQuery>,
) -> impl IntoResponse {
    match state.list_api_keys_uc.execute(&query.tenant_id).await {
        Ok(keys) => (StatusCode::OK, Json(serde_json::to_value(keys).unwrap())).into_response(),
        Err(crate::usecase::list_api_keys::ListApiKeysError::Validation(msg)) => {
            let err = ErrorResponse::new("SYS_AUTH_API_KEY_VALIDATION", &msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_AUTH_API_KEY_LIST_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

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
            let err = ErrorResponse::new("SYS_AUTH_API_KEY_REVOKE_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}
