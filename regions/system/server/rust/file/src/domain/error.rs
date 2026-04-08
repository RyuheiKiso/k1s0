//! File サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// File ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    /// ファイル名が不正（パストラバーサル等）
    #[error("invalid filename: {0}")]
    InvalidFileName(String),

    /// ファイルが見つからない
    #[error("file '{0}' not found")]
    NotFound(String),

    /// ファイルが既にアップロード完了済み
    #[error("file already completed: {0}")]
    AlreadyCompleted(String),

    /// ファイルがまだ利用可能でない
    #[error("file not available: {0}")]
    NotAvailable(String),

    /// アクセス権限がない
    #[error("access denied: {0}")]
    AccessDenied(String),

    /// ファイルサイズ超過
    #[error("file size exceeded: {0}")]
    SizeExceeded(String),

    /// ストレージエラー
    #[error("storage error: {0}")]
    StorageError(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// `FileError` から `ServiceError` への変換実装
impl From<FileError> for ServiceError {
    fn from(err: FileError) -> Self {
        match err {
            FileError::InvalidFileName(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_FILE_INVALID_FILENAME"),
                message: msg,
                details: vec![],
            },
            FileError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_FILE_NOT_FOUND"),
                message: msg,
            },
            FileError::AlreadyCompleted(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_FILE_ALREADY_COMPLETED"),
                message: msg,
                details: vec![],
            },
            FileError::NotAvailable(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_FILE_NOT_AVAILABLE"),
                message: msg,
                details: vec![],
            },
            FileError::AccessDenied(msg) => ServiceError::Forbidden {
                code: ErrorCode::new("SYS_FILE_ACCESS_DENIED"),
                message: msg,
            },
            FileError::SizeExceeded(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_FILE_SIZE_EXCEEDED"),
                message: msg,
                details: vec![],
            },
            FileError::StorageError(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_FILE_STORAGE_ERROR"),
                message: msg,
            },
            FileError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_FILE_VALIDATION"),
                message: msg,
                details: vec![],
            },
            FileError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_FILE_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
