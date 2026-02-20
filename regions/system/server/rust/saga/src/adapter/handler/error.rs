use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use super::ErrorResponse;

/// SagaError はSaga操作のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum SagaError {
    #[error("saga not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for SagaError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            SagaError::NotFound(msg) => (StatusCode::NOT_FOUND, "SYS_SAGA_NOT_FOUND", msg.as_str()),
            SagaError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                "SYS_SAGA_VALIDATION_ERROR",
                msg.as_str(),
            ),
            SagaError::Conflict(msg) => (StatusCode::CONFLICT, "SYS_SAGA_CONFLICT", msg.as_str()),
            SagaError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "SYS_SAGA_INTERNAL_ERROR",
                msg.as_str(),
            ),
        };

        let body = ErrorResponse::new(code, message);
        (status, Json(body)).into_response()
    }
}
