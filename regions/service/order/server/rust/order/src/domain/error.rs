//! Order サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// Order ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum OrderError {
    #[error("Order '{0}' not found")]
    NotFound(String),

    #[error("invalid status transition: '{from}' -> '{to}'")]
    InvalidStatusTransition { from: String, to: String },

    #[error("validation failed: {0}")]
    ValidationFailed(String),

    #[error("version conflict for order '{0}'")]
    VersionConflict(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<OrderError> for ServiceError {
    fn from(err: OrderError) -> Self {
        match err {
            OrderError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SVC_ORDER_NOT_FOUND"),
                message: msg,
            },
            OrderError::InvalidStatusTransition { from, to } => ServiceError::BadRequest {
                code: ErrorCode::new("SVC_ORDER_INVALID_STATUS_TRANSITION"),
                message: format!("invalid status transition: '{}' -> '{}'", from, to),
                details: vec![],
            },
            OrderError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SVC_ORDER_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            OrderError::VersionConflict(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SVC_ORDER_VERSION_CONFLICT"),
                message: msg,
                details: vec![],
            },
            OrderError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SVC_ORDER_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
