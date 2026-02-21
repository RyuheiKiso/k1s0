use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use super::ErrorResponse;

/// DlqError は DLQ 操作のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum DlqError {
    #[error("dlq message not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for DlqError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            DlqError::NotFound(msg) => (StatusCode::NOT_FOUND, "SYS_DLQ_NOT_FOUND", msg.as_str()),
            DlqError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                "SYS_DLQ_VALIDATION_ERROR",
                msg.as_str(),
            ),
            DlqError::Conflict(msg) => (StatusCode::CONFLICT, "SYS_DLQ_CONFLICT", msg.as_str()),
            DlqError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "SYS_DLQ_INTERNAL_ERROR",
                msg.as_str(),
            ),
        };

        let body = ErrorResponse::new(code, message);
        (status, Json(body)).into_response()
    }
}
