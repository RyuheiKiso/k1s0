//! DLQ Manager サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// DlqManager ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum DlqManagerError {
    /// DLQ メッセージが見つからない
    #[error("DLQ message '{0}' not found")]
    NotFound(String),

    /// メッセージの再処理が失敗
    #[error("process failed: {0}")]
    ProcessFailed(String),

    /// 既にメッセージが処理済み
    #[error("already processed: {0}")]
    AlreadyProcessed(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// DlqManagerError から ServiceError への変換実装
impl From<DlqManagerError> for ServiceError {
    fn from(err: DlqManagerError) -> Self {
        match err {
            DlqManagerError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_DLQ_NOT_FOUND"),
                message: msg,
            },
            DlqManagerError::ProcessFailed(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_DLQ_PROCESS_FAILED"),
                message: msg,
            },
            DlqManagerError::AlreadyProcessed(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_DLQ_CONFLICT"),
                message: msg,
                details: vec![],
            },
            DlqManagerError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_DLQ_VALIDATION_ERROR"),
                message: msg,
                details: vec![],
            },
            DlqManagerError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_DLQ_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
