use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use k1s0_server_common::error as codes;

use super::ErrorResponse;

/// EventStoreError はイベントストア REST API のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    #[error("stream not found: {0}")]
    StreamNotFound(String),

    #[error("event not found: {0}")]
    EventNotFound(String),

    #[error("snapshot not found: {0}")]
    SnapshotNotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("version conflict: {0}")]
    VersionConflict(String),

    #[error("stream already exists: {0}")]
    StreamAlreadyExists(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for EventStoreError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            EventStoreError::StreamNotFound(msg) => (
                StatusCode::NOT_FOUND,
                codes::event_store::stream_not_found(),
                msg.as_str(),
            ),
            EventStoreError::EventNotFound(msg) => (
                StatusCode::NOT_FOUND,
                codes::event_store::event_not_found(),
                msg.as_str(),
            ),
            EventStoreError::SnapshotNotFound(msg) => (
                StatusCode::NOT_FOUND,
                codes::event_store::snapshot_not_found(),
                msg.as_str(),
            ),
            EventStoreError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                k1s0_server_common::error::ErrorCode::new("SYS_EVSTORE_VALIDATION_ERROR"),
                msg.as_str(),
            ),
            EventStoreError::VersionConflict(msg) => (
                StatusCode::CONFLICT,
                codes::event_store::version_conflict(),
                msg.as_str(),
            ),
            EventStoreError::StreamAlreadyExists(msg) => (
                StatusCode::CONFLICT,
                codes::event_store::stream_already_exists(),
                msg.as_str(),
            ),
            EventStoreError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                k1s0_server_common::error::ErrorCode::new("SYS_EVSTORE_INTERNAL_ERROR"),
                msg.as_str(),
            ),
        };

        let body = ErrorResponse::new(code.as_str(), message);
        (status, Json(body)).into_response()
    }
}
