use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("キャッシュキーが見つかりません: {key}")]
    NotFound { key: String },
    #[error("ロック取得に失敗しました: {key}")]
    LockFailed { key: String },
    #[error("ロックが期限切れです: {key}")]
    LockExpired { key: String },
    #[error("シリアライズエラー: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("接続エラー: {0}")]
    ConnectionError(String),
}
