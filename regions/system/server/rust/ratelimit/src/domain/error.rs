//! RateLimit サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// RateLimit ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    /// レート制限設定が見つからない
    #[error("rate limit config '{0}' not found")]
    NotFound(String),

    /// レート制限を超過
    #[error("rate limit exceeded: {0}")]
    RateExceeded(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// RateLimitError から ServiceError への変換実装
impl From<RateLimitError> for ServiceError {
    fn from(err: RateLimitError) -> Self {
        match err {
            RateLimitError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_RATELIMIT_NOT_FOUND"),
                message: msg,
            },
            RateLimitError::RateExceeded(msg) => ServiceError::TooManyRequests {
                code: ErrorCode::new("SYS_RATELIMIT_RATE_EXCEEDED"),
                message: msg,
            },
            RateLimitError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_RATELIMIT_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            RateLimitError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_RATELIMIT_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
