use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdempotencyError {
    #[error("重複リクエストです: key={key}")]
    Duplicate { key: String },
    #[error("キーが見つかりません: {key}")]
    NotFound { key: String },
    #[error("無効なステータス遷移: {from:?} -> {to:?}")]
    InvalidStatusTransition { from: String, to: String },
    #[error("シリアライズエラー: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("ストレージエラー: {0}")]
    StorageError(String),
}
