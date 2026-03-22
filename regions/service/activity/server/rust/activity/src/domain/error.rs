// アクティビティドメインエラー型。
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ActivityError {
    #[error("invalid status transition: from '{from}' to '{to}'")]
    InvalidStatusTransition { from: String, to: String },
    #[error("activity not found: {0}")]
    NotFound(String),
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    #[error("idempotency key already used: {0}")]
    DuplicateIdempotencyKey(String),
}
