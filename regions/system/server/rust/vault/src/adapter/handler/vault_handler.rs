use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use k1s0_server_common::{ErrorDetail, ErrorResponse};
use serde::{Deserialize, Serialize};

use crate::adapter::middleware::auth::VaultAuthState;
use crate::adapter::middleware::spiffe::SpiffeAuthState;
use crate::usecase::delete_secret::{DeleteSecretError, DeleteSecretInput};
use crate::usecase::get_secret::{GetSecretError, GetSecretInput};
use crate::usecase::list_audit_logs::ListAuditLogsInput;
use crate::usecase::rotate_secret::{RotateSecretError, RotateSecretInput, RotateSecretUseCase};
use crate::usecase::set_secret::{SetSecretError, SetSecretInput};
use crate::usecase::{
    DeleteSecretUseCase, GetSecretUseCase, ListAuditLogsUseCase, ListSecretsUseCase,
    SetSecretUseCase,
};

#[derive(Clone)]
pub struct AppState {
    pub get_secret_uc: Arc<GetSecretUseCase>,
    pub set_secret_uc: Arc<SetSecretUseCase>,
    pub rotate_secret_uc: Arc<RotateSecretUseCase>,
    pub delete_secret_uc: Arc<DeleteSecretUseCase>,
    pub list_secrets_uc: Arc<ListSecretsUseCase>,
    pub list_audit_logs_uc: Arc<ListAuditLogsUseCase>,
    pub db_pool: Option<sqlx::PgPool>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<VaultAuthState>,
    pub spiffe_state: Option<SpiffeAuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: VaultAuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }

    #[allow(dead_code)]
    pub fn with_spiffe(mut self, spiffe_state: SpiffeAuthState) -> Self {
        self.spiffe_state = Some(spiffe_state);
        self
    }
}

// --- Query DTOs ---

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    #[serde(default = "default_audit_offset")]
    pub offset: u32,
    #[serde(default = "default_audit_limit")]
    pub limit: u32,
}

#[derive(Debug, Deserialize, Default)]
pub struct SecretVersionQuery {
    pub version: Option<i64>,
}

fn default_audit_offset() -> u32 {
    0
}
fn default_audit_limit() -> u32 {
    20
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
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct GetSecretResponse {
    pub path: String,
    pub version: i64,
    pub data: HashMap<String, String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSecretRequest {
    pub data: HashMap<String, String>,
}

fn classify_vault_internal_error(msg: &str) -> (StatusCode, &'static str) {
    let lower = msg.to_ascii_lowercase();
    if lower.contains("denied") || lower.contains("forbidden") || lower.contains("permission") {
        return (StatusCode::FORBIDDEN, "SYS_VAULT_ACCESS_DENIED");
    }
    if lower.contains("cache") {
        return (StatusCode::INTERNAL_SERVER_ERROR, "SYS_VAULT_CACHE_ERROR");
    }
    if lower.contains("validation") || lower.contains("invalid") {
        return (StatusCode::BAD_REQUEST, "SYS_VAULT_VALIDATION_ERROR");
    }
    if lower.contains("upstream") || lower.contains("backend") || lower.contains("vault") {
        return (StatusCode::BAD_GATEWAY, "SYS_VAULT_UPSTREAM_ERROR");
    }
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "SYS_VAULT_INTERNAL_ERROR",
    )
}

fn internal_error_response(msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    let (status, code) = classify_vault_internal_error(msg);
    let err = if code == "SYS_VAULT_VALIDATION_ERROR" {
        ErrorResponse::with_details(
            code,
            msg,
            vec![ErrorDetail::new(
                "request",
                "validation_error",
                "invalid request payload",
            )],
        )
    } else {
        ErrorResponse::new(code, msg)
    };
    (status, Json(serde_json::to_value(err).unwrap()))
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
        Ok(output) => {
            let resp = SetSecretResponse {
                path: req.path,
                version: output.version,
                created_at: output.created_at.to_rfc3339(),
            };
            (
                StatusCode::CREATED,
                Json(serde_json::to_value(resp).unwrap()),
            )
                .into_response()
        }
        Err(SetSecretError::Internal(msg)) => internal_error_response(&msg).into_response(),
    }
}

/// GET /api/v1/secrets/:key
pub async fn get_secret(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Query(query): Query<SecretVersionQuery>,
) -> impl IntoResponse {
    let input = GetSecretInput {
        path: key.clone(),
        version: query.version,
    };

    match state.get_secret_uc.execute(&input).await {
        Ok(secret) => {
            let current = secret.get_version(None);
            let data = current.map(|v| v.value.data.clone()).unwrap_or_default();

            let resp = GetSecretResponse {
                path: secret.path,
                version: secret.current_version,
                data,
                created_at: secret.created_at.to_rfc3339(),
                updated_at: secret.updated_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(GetSecretError::NotFound(path)) => (
            StatusCode::NOT_FOUND,
            Json(
                serde_json::to_value(ErrorResponse::new(
                    "SYS_VAULT_NOT_FOUND",
                    format!("secret not found: {}", path),
                ))
                .unwrap(),
            ),
        )
            .into_response(),
        Err(GetSecretError::Internal(msg)) => internal_error_response(&msg).into_response(),
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
        Ok(output) => {
            let resp = SetSecretResponse {
                path: key,
                version: output.version,
                created_at: output.created_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(SetSecretError::Internal(msg)) => internal_error_response(&msg).into_response(),
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
            Json(
                serde_json::to_value(ErrorResponse::new(
                    "SYS_VAULT_NOT_FOUND",
                    format!("secret not found: {}", path),
                ))
                .unwrap(),
            ),
        )
            .into_response(),
        Err(DeleteSecretError::Internal(msg)) => internal_error_response(&msg).into_response(),
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
        Err(e) => internal_error_response(&e.to_string()).into_response(),
    }
}

/// GET /api/v1/secrets/:key/metadata
pub async fn get_secret_metadata(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let input = GetSecretInput {
        path: key.clone(),
        version: None,
    };

    match state.get_secret_uc.execute(&input).await {
        Ok(secret) => {
            let resp = serde_json::json!({
                "path": secret.path,
                "version": secret.current_version,
                "version_count": secret.versions.len(),
                "created_at": secret.created_at.to_rfc3339(),
                "updated_at": secret.updated_at.to_rfc3339(),
            });
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(GetSecretError::NotFound(path)) => (
            StatusCode::NOT_FOUND,
            Json(
                serde_json::to_value(ErrorResponse::new(
                    "SYS_VAULT_NOT_FOUND",
                    format!("secret not found: {}", path),
                ))
                .unwrap(),
            ),
        )
            .into_response(),
        Err(GetSecretError::Internal(msg)) => internal_error_response(&msg).into_response(),
    }
}

/// GET /api/v1/audit/logs
pub async fn list_audit_logs(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<AuditLogQuery>,
) -> impl IntoResponse {
    match state
        .list_audit_logs_uc
        .execute(&ListAuditLogsInput {
            offset: query.offset,
            limit: query.limit,
        })
        .await
    {
        Ok(logs) => {
            let entries: Vec<serde_json::Value> = logs
                .into_iter()
                .map(|log| {
                    let action = match &log.action {
                        crate::domain::entity::access_log::AccessAction::Read => "read",
                        crate::domain::entity::access_log::AccessAction::Write => "write",
                        crate::domain::entity::access_log::AccessAction::Delete => "delete",
                        crate::domain::entity::access_log::AccessAction::List => "list",
                    };
                    serde_json::json!({
                        "id": log.id.to_string(),
                        "key_path": log.path,
                        "action": action,
                        "actor_id": log.subject,
                        "ip_address": log.ip_address,
                        "success": log.success,
                        "error_msg": log.error_msg,
                        "created_at": log.created_at.to_rfc3339(),
                    })
                })
                .collect();
            (StatusCode::OK, Json(serde_json::json!({ "logs": entries }))).into_response()
        }
        Err(e) => internal_error_response(&e.to_string()).into_response(),
    }
}

/// POST /api/v1/secrets/:key/rotate
pub async fn rotate_secret(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(req): Json<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let input = RotateSecretInput {
        path: key.clone(),
        data: req,
    };

    match state.rotate_secret_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "path": output.path,
                "new_version": output.new_version,
                "rotated": output.rotated,
            })),
        )
            .into_response(),
        Err(RotateSecretError::NotFound(path)) => (
            StatusCode::NOT_FOUND,
            Json(
                serde_json::to_value(ErrorResponse::new(
                    "SYS_VAULT_NOT_FOUND",
                    format!("secret not found: {}", path),
                ))
                .unwrap(),
            ),
        )
            .into_response(),
        Err(RotateSecretError::Internal(msg)) => internal_error_response(&msg).into_response(),
    }
}
