//! AI Gateway サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// `AiGateway` ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum AiGatewayError {
    /// モデルが見つからない
    #[error("model '{0}' not found")]
    NotFound(String),

    /// モデルへのリクエストが失敗
    #[error("model request failed: {0}")]
    ModelRequestFailed(String),

    /// レート制限を超過
    #[error("rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// `AiGatewayError` から `ServiceError` への変換実装
impl From<AiGatewayError> for ServiceError {
    fn from(err: AiGatewayError) -> Self {
        match err {
            AiGatewayError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_AIGW_NOT_FOUND"),
                message: msg,
            },
            AiGatewayError::ModelRequestFailed(msg) => ServiceError::ServiceUnavailable {
                code: ErrorCode::new("SYS_AIGW_MODEL_REQUEST_FAILED"),
                message: msg,
            },
            AiGatewayError::RateLimitExceeded(msg) => ServiceError::TooManyRequests {
                code: ErrorCode::new("SYS_AIGW_RATE_LIMIT_EXCEEDED"),
                message: msg,
            },
            AiGatewayError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_AIGW_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            AiGatewayError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_AIGW_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
