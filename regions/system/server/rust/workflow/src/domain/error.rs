//! Workflow サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// Workflow ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    /// ワークフローが見つからない
    #[error("workflow '{0}' not found")]
    NotFound(String),

    /// ワークフローの状態遷移が無効
    #[error("invalid status transition: '{from}' -> '{to}'")]
    InvalidStatusTransition { from: String, to: String },

    /// ワークフローが既に存在する
    #[error("workflow already exists: {0}")]
    AlreadyExists(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// WorkflowError から ServiceError への変換実装
impl From<WorkflowError> for ServiceError {
    fn from(err: WorkflowError) -> Self {
        match err {
            WorkflowError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_WORKFLOW_NOT_FOUND"),
                message: msg,
            },
            WorkflowError::InvalidStatusTransition { from, to } => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_WORKFLOW_INVALID_STATUS_TRANSITION"),
                message: format!("invalid status transition: '{}' -> '{}'", from, to),
                details: vec![],
            },
            WorkflowError::AlreadyExists(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_WORKFLOW_ALREADY_EXISTS"),
                message: msg,
                details: vec![],
            },
            WorkflowError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_WORKFLOW_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            WorkflowError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_WORKFLOW_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
