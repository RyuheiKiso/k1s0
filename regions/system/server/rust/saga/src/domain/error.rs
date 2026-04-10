//! Saga サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// Saga ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum SagaError {
    /// サガが見つからない
    #[error("saga '{0}' not found")]
    NotFound(String),

    /// サガの状態遷移が無効
    #[error("invalid status transition: '{from}' -> '{to}'")]
    InvalidStatusTransition { from: String, to: String },

    /// 補償処理が失敗
    #[error("compensation failed: {0}")]
    CompensationFailed(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// `SagaError` から `ServiceError` への変換実装
impl From<SagaError> for ServiceError {
    fn from(err: SagaError) -> Self {
        match err {
            SagaError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_SAGA_NOT_FOUND"),
                message: msg,
            },
            SagaError::InvalidStatusTransition { from, to } => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_SAGA_INVALID_STATUS_TRANSITION"),
                message: format!("invalid status transition: '{from}' -> '{to}'"),
                details: vec![],
            },
            SagaError::CompensationFailed(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_SAGA_COMPENSATION_FAILED"),
                message: msg,
            },
            SagaError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_SAGA_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            SagaError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_SAGA_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
