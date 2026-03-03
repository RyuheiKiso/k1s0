use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

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
                "SYS_EVSTORE_STREAM_NOT_FOUND",
                msg.as_str(),
            ),
            EventStoreError::EventNotFound(msg) => (
                StatusCode::NOT_FOUND,
                "SYS_EVSTORE_EVENT_NOT_FOUND",
                msg.as_str(),
            ),
            EventStoreError::SnapshotNotFound(msg) => (
                StatusCode::NOT_FOUND,
                "SYS_EVSTORE_SNAPSHOT_NOT_FOUND",
                msg.as_str(),
            ),
            EventStoreError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                "SYS_EVSTORE_VALIDATION_ERROR",
                msg.as_str(),
            ),
            EventStoreError::VersionConflict(msg) => (
                StatusCode::CONFLICT,
                "SYS_EVSTORE_VERSION_CONFLICT",
                msg.as_str(),
            ),
            EventStoreError::StreamAlreadyExists(msg) => (
                StatusCode::CONFLICT,
                "SYS_EVSTORE_STREAM_ALREADY_EXISTS",
                msg.as_str(),
            ),
            EventStoreError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "SYS_EVSTORE_INTERNAL_ERROR",
                msg.as_str(),
            ),
        };

        let body = ErrorResponse::new(code, message);
        (status, Json(body)).into_response()
    }
}
