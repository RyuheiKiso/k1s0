//! Payment サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// Payment ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("Payment '{0}' not found")]
    NotFound(String),

    #[error("invalid status transition: '{from}' -> '{to}'")]
    InvalidStatusTransition { from: String, to: String },

    #[error("validation failed: {0}")]
    ValidationFailed(String),

    #[error("version conflict for payment '{0}'")]
    VersionConflict(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<PaymentError> for ServiceError {
    fn from(err: PaymentError) -> Self {
        match err {
            PaymentError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SVC_PAYMENT_NOT_FOUND"),
                message: msg,
            },
            PaymentError::InvalidStatusTransition { from, to } => ServiceError::BadRequest {
                code: ErrorCode::new("SVC_PAYMENT_INVALID_STATUS_TRANSITION"),
                message: format!("invalid status transition: '{}' -> '{}'", from, to),
                details: vec![],
            },
            PaymentError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SVC_PAYMENT_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            PaymentError::VersionConflict(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SVC_PAYMENT_VERSION_CONFLICT"),
                message: msg,
                details: vec![],
            },
            PaymentError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SVC_PAYMENT_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
