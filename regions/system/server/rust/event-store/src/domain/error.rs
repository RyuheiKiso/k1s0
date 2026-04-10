//! Event Store サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// `EventStore` ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    /// ストリームが見つからない
    #[error("stream '{0}' not found")]
    StreamNotFound(String),

    /// イベントが見つからない
    #[error("event '{0}' not found")]
    EventNotFound(String),

    /// スナップショットが見つからない
    #[error("snapshot '{0}' not found")]
    SnapshotNotFound(String),

    /// ストリームが既に存在する
    #[error("stream already exists: {0}")]
    StreamAlreadyExists(String),

    /// バージョン競合
    #[error("version conflict: {0}")]
    VersionConflict(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// `EventStoreError` から `ServiceError` への変換実装
impl From<EventStoreError> for ServiceError {
    fn from(err: EventStoreError) -> Self {
        match err {
            EventStoreError::StreamNotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_EVSTORE_STREAM_NOT_FOUND"),
                message: msg,
            },
            EventStoreError::EventNotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_EVSTORE_EVENT_NOT_FOUND"),
                message: msg,
            },
            EventStoreError::SnapshotNotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_EVSTORE_SNAPSHOT_NOT_FOUND"),
                message: msg,
            },
            EventStoreError::StreamAlreadyExists(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_EVSTORE_STREAM_ALREADY_EXISTS"),
                message: msg,
                details: vec![],
            },
            EventStoreError::VersionConflict(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_EVSTORE_VERSION_CONFLICT"),
                message: msg,
                details: vec![],
            },
            EventStoreError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_EVSTORE_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            EventStoreError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_EVSTORE_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
