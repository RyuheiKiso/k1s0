//! AI Agent サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// AiAgent ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum AiAgentError {
    /// エージェントが見つからない
    #[error("agent '{0}' not found")]
    NotFound(String),

    /// エージェントの実行に失敗
    #[error("execution failed: {0}")]
    ExecutionFailed(String),

    /// エージェントが既に存在する
    #[error("agent already exists: {0}")]
    AlreadyExists(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// AiAgentError から ServiceError への変換実装
impl From<AiAgentError> for ServiceError {
    fn from(err: AiAgentError) -> Self {
        match err {
            AiAgentError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_AIAGENT_NOT_FOUND"),
                message: msg,
            },
            AiAgentError::ExecutionFailed(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_AIAGENT_EXECUTION_FAILED"),
                message: msg,
            },
            AiAgentError::AlreadyExists(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_AIAGENT_ALREADY_EXISTS"),
                message: msg,
                details: vec![],
            },
            AiAgentError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_AIAGENT_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            AiAgentError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_AIAGENT_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
