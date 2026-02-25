use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::usecase::delete_secret::{DeleteSecretError, DeleteSecretInput};
use crate::usecase::get_secret::{GetSecretError, GetSecretInput};
use crate::usecase::set_secret::{SetSecretError, SetSecretInput};
use crate::usecase::{DeleteSecretUseCase, GetSecretUseCase, ListSecretsUseCase, SetSecretUseCase};

#[derive(Clone)]
pub struct AppState {
    pub get_secret_uc: Arc<GetSecretUseCase>,
    pub set_secret_uc: Arc<SetSecretUseCase>,
    pub delete_secret_uc: Arc<DeleteSecretUseCase>,
    pub list_secrets_uc: Arc<ListSecretsUseCase>,
    pub db_pool: Option<sqlx::PgPool>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
}

// --- Request / Response DTOs ---

#[derive(Debug, Deserialize)]
pub struct SetSecretRequest {
    pub path: String,
    pub data: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct SetSecretResponse {
    pub path: String,
    pub version: i64,
}

#[derive(Debug, Serialize)]
pub struct GetSecretResponse {
    pub path: String,
    pub current_version: i64,
    pub data: HashMap<String, String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSecretRequest {
    pub data: HashMap<String, String>,
}

// --- Handlers ---

/// POST /api/v1/secrets
pub async fn create_secret(
    State(state): State<AppState>,
    Json(req): Json<SetSecretRequest>,
) -> impl IntoResponse {
    let input = SetSecretInput {
        path: req.path.clone(),
        data: req.data,
    };

    match state.set_secret_uc.execute(&input).await {
        Ok(version) => {
            let resp = SetSecretResponse {
                path: req.path,
                version,
            };
            (
                StatusCode::CREATED,
                Json(serde_json::to_value(resp).unwrap()),
            )
                .into_response()
        }
        Err(SetSecretError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// GET /api/v1/secrets/:key
pub async fn get_secret(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let input = GetSecretInput {
        path: key.clone(),
        version: None,
    };

    match state.get_secret_uc.execute(&input).await {
        Ok(secret) => {
            let current = secret.get_version(None);
            let data = current
                .map(|v| v.value.data.clone())
                .unwrap_or_default();

            let resp = GetSecretResponse {
                path: secret.path,
                current_version: secret.current_version,
                data,
                created_at: secret.created_at.to_rfc3339(),
                updated_at: secret.updated_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(GetSecretError::NotFound(path)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("secret not found: {}", path)})),
        )
            .into_response(),
        Err(GetSecretError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// PUT /api/v1/secrets/:key
pub async fn update_secret(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<UpdateSecretRequest>,
) -> impl IntoResponse {
    let input = SetSecretInput {
        path: key.clone(),
        data: req.data,
    };

    match state.set_secret_uc.execute(&input).await {
        Ok(version) => {
            let resp = SetSecretResponse {
                path: key,
                version,
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(SetSecretError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// DELETE /api/v1/secrets/:key
pub async fn delete_secret(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let input = DeleteSecretInput {
        path: key.clone(),
        versions: vec![], // delete all versions
    };

    match state.delete_secret_uc.execute(&input).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(DeleteSecretError::NotFound(path)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("secret not found: {}", path)})),
        )
            .into_response(),
        Err(DeleteSecretError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// GET /api/v1/secrets
pub async fn list_secrets(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let prefix = params.get("prefix").cloned().unwrap_or_default();

    match state.list_secrets_uc.execute(&prefix).await {
        Ok(paths) => (
            StatusCode::OK,
            Json(serde_json::json!({ "secrets": paths })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/v1/secrets/:key/metadata
pub async fn get_secret_metadata(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let input = GetSecretInput { path: key.clone(), version: None };

    match state.get_secret_uc.execute(&input).await {
        Ok(secret) => {
            let resp = serde_json::json!({
                "path": secret.path,
                "current_version": secret.current_version,
                "version_count": secret.versions.len(),
                "created_at": secret.created_at.to_rfc3339(),
                "updated_at": secret.updated_at.to_rfc3339(),
            });
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(GetSecretError::NotFound(path)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("secret not found: {}", path)})),
        )
            .into_response(),
        Err(GetSecretError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// POST /api/v1/secrets/:key/rotate
pub async fn rotate_secret(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    // ローテーションは新しいデータで上書き（バージョンインクリメント）
    let input = SetSecretInput {
        path: key.clone(),
        data: req,
    };

    match state.set_secret_uc.execute(&input).await {
        Ok(version) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "path": key,
                "new_version": version,
                "rotated": true,
            })),
        )
            .into_response(),
        Err(SetSecretError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}
