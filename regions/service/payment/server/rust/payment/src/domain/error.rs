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

    /// 同一 order_id で異なる金額/通貨の決済が既に存在する場合の冪等性違反エラー。
    #[error("冪等性違反: 注文 '{order_id}' の決済は異なる金額/通貨で既に存在します")]
    IdempotencyViolation { order_id: String },

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
            // 冪等性違反は HTTP 409 Conflict にマッピングする。
            PaymentError::IdempotencyViolation { order_id } => ServiceError::Conflict {
                code: ErrorCode::new("SVC_PAYMENT_IDEMPOTENCY_VIOLATION"),
                message: format!("冪等性違反: 注文 '{}' の決済は異なる金額/通貨で既に存在します", order_id),
                details: vec![],
            },
            PaymentError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SVC_PAYMENT_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
