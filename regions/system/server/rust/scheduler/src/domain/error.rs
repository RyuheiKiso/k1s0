//! Scheduler サービスのドメインエラー型。
//!
//! 文字列マッチングではなく、型安全な分類で HTTP ステータスコードを決定する。

use k1s0_server_common::error::{ErrorCode, ServiceError};

/// Scheduler ドメイン固有のエラー型。
#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    /// ジョブが見つからない
    #[error("job '{0}' not found")]
    NotFound(String),

    /// ジョブが既に存在する
    #[error("job already exists: {0}")]
    AlreadyExists(String),

    /// 無効なスケジュール式
    #[error("invalid schedule expression: {0}")]
    InvalidSchedule(String),

    /// バリデーションエラー
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// 内部エラー
    #[error("internal error: {0}")]
    Internal(String),
}

/// SchedulerError から ServiceError への変換実装
impl From<SchedulerError> for ServiceError {
    fn from(err: SchedulerError) -> Self {
        match err {
            SchedulerError::NotFound(msg) => ServiceError::NotFound {
                code: ErrorCode::new("SYS_SCHED_NOT_FOUND"),
                message: msg,
            },
            SchedulerError::AlreadyExists(msg) => ServiceError::Conflict {
                code: ErrorCode::new("SYS_SCHED_ALREADY_EXISTS"),
                message: msg,
                details: vec![],
            },
            SchedulerError::InvalidSchedule(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_SCHED_INVALID_SCHEDULE"),
                message: msg,
                details: vec![],
            },
            SchedulerError::ValidationFailed(msg) => ServiceError::BadRequest {
                code: ErrorCode::new("SYS_SCHED_VALIDATION_FAILED"),
                message: msg,
                details: vec![],
            },
            SchedulerError::Internal(msg) => ServiceError::Internal {
                code: ErrorCode::new("SYS_SCHED_INTERNAL_ERROR"),
                message: msg,
            },
        }
    }
}
