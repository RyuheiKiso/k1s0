use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("シリアライズエラー: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("送信エラー: {0}")]
    SendError(String),
    #[error("内部エラー: {0}")]
    Internal(String),
}
