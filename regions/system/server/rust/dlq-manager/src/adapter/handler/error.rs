use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use k1s0_server_common::error as codes;
use k1s0_server_common::ErrorResponse;

/// DlqError は DLQ 操作のエラー型。
///
/// Error codes follow the `SYS_DLQ_*` pattern via k1s0-server-common.
#[derive(Debug, thiserror::Error)]
pub enum DlqError {
    #[error("dlq message not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("retry processing failed: {0}")]
    ProcessFailed(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for DlqError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            DlqError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                codes::dlq::not_found(),
                msg.as_str(),
            ),
            DlqError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                codes::dlq::validation_error(),
                msg.as_str(),
            ),
            DlqError::Conflict(msg) => (
                StatusCode::CONFLICT,
                codes::dlq::conflict(),
                msg.as_str(),
            ),
            DlqError::ProcessFailed(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                codes::dlq::process_failed(),
                msg.as_str(),
            ),
            DlqError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                codes::dlq::internal_error(),
                msg.as_str(),
            ),
        };

        let body = ErrorResponse::new(code, message);
        (status, Json(body)).into_response()
    }
}
