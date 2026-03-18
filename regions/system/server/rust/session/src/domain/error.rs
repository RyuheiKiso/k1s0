//! Session サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// Session ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    /// セッションが見つからない
    #[error("session '{0}' not found")]
    NotFound(String),

    /// セッションが期限切れ
    #[error("session expired: {0}")]
    Expired(String),

    /// セッションが既に無効化済み
    #[error("session already revoked: {0}")]
    AlreadyRevoked(String),

    /// デバイス数の上限に達した
    #[error("max devices exceeded: {0}")]
    MaxDevicesExceeded(String),

    /// アクセス権限がない
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// SessionError から ServiceError への変換実装
impl From<SessionError> for ServiceError {
    fn from(err: SessionError) -> Self {
        match err {
            SessionError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_SESSION_NOT_FOUND"),
                message: msg,
            },
            SessionError::Expired(msg) => ServiceError::Unauthorized {
                code: ErrorCode::new("SYS_SESSION_EXPIRED"),
                message: msg,
            },
            SessionError::AlreadyRevoked(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_SESSION_ALREADY_REVOKED"),
                message: msg,
                details: vec![],
            },
            SessionError::MaxDevicesExceeded(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_SESSION_MAX_DEVICES_EXCEEDED"),
                message: msg,
                details: vec![],
            },
            SessionError::Forbidden(msg) => ServiceError::Forbidden {
                code: ErrorCode::new("SYS_SESSION_FORBIDDEN"),
                message: msg,
            },
            SessionError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_SESSION_VALIDATION_ERROR"),
                message: msg,
                details: vec![],
            },
            SessionError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_SESSION_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
