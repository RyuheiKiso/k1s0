use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use super::ErrorResponse;

/// EventStoreError はイベントストア REST API のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for EventStoreError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            EventStoreError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, "SYS_EVENT_STORE_NOT_FOUND", msg.as_str())
            }
            EventStoreError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                "SYS_EVENT_STORE_VALIDATION_ERROR",
                msg.as_str(),
            ),
            EventStoreError::Conflict(msg) => {
                (StatusCode::CONFLICT, "SYS_EVENT_STORE_CONFLICT", msg.as_str())
            }
            EventStoreError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "SYS_EVENT_STORE_INTERNAL_ERROR",
                msg.as_str(),
            ),
        };

        let body = ErrorResponse::new(code, message);
        (status, Json(body)).into_response()
    }
}
