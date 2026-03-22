// タスクドメインエラー型。
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("invalid status transition: from '{from}' to '{to}'")]
    InvalidStatusTransition { from: String, to: String },
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    #[error("task not found: {0}")]
    NotFound(String),
}
