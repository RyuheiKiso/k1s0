//! Quota サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// Quota ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum QuotaError {
    /// クォータが見つからない
    #[error("quota '{0}' not found")]
    NotFound(String),

    /// クォータの上限に達した
    #[error("quota exceeded: {0}")]
    QuotaExceeded(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// QuotaError から ServiceError への変換実装
impl From<QuotaError> for ServiceError {
    fn from(err: QuotaError) -> Self {
        match err {
            QuotaError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_QUOTA_NOT_FOUND"),
                message: msg,
            },
            QuotaError::QuotaExceeded(msg) => ServiceError::TooManyRequests {
                code: ErrorCode::new("SYS_QUOTA_EXCEEDED"),
                message: msg,
            },
            QuotaError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_QUOTA_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            QuotaError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_QUOTA_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
