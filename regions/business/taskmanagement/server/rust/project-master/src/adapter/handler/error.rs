// REST ハンドラエラー型。
// ドメインエラーを HTTP レスポンスにマッピングする。
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use crate::domain::error::ProjectMasterError;

/// サービスエラー（HTTP レスポンス用）
#[derive(Debug)]
pub enum ServiceError {
    NotFound(String),
    Conflict(String),
    Validation(String),
    Internal(String),
    Unauthorized(String),
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ServiceError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ServiceError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ServiceError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            ServiceError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ServiceError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

/// ドメインエラーをサービスエラーに変換する
pub fn map_domain_error(err: anyhow::Error) -> ServiceError {
    match err.downcast::<ProjectMasterError>() {
        Ok(domain_err) => match domain_err {
            ProjectMasterError::ProjectTypeNotFound(msg) => ServiceError::NotFound(msg),
            ProjectMasterError::StatusDefinitionNotFound(msg) => ServiceError::NotFound(msg),
            ProjectMasterError::TenantExtensionNotFound(t, s) => {
                ServiceError::NotFound(format!("tenant={}, status={}", t, s))
            }
            ProjectMasterError::ValidationFailed(msg) => ServiceError::Validation(msg),
            ProjectMasterError::DuplicateCode(msg) => ServiceError::Conflict(msg),
            ProjectMasterError::InvalidValidationSchema(msg) => ServiceError::Validation(msg),
            ProjectMasterError::Internal(msg) => ServiceError::Internal(msg),
        },
        Err(other) => ServiceError::Internal(other.to_string()),
    }
}
