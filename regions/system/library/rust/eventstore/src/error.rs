use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventStoreError {
    #[error("バージョン競合: expected={expected}, actual={actual}")]
    VersionConflict { expected: u64, actual: u64 },
    #[error("ストリームが見つかりません: {stream_id}")]
    StreamNotFound { stream_id: String },
    #[error("シリアライズエラー: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("ストレージエラー: {0}")]
    StorageError(String),
}
