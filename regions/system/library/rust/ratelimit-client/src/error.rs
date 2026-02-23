use thiserror::Error;

#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("レート制限超過: {retry_after_secs}秒後に再試行してください")]
    LimitExceeded { retry_after_secs: u64 },
    #[error("キーが見つかりません: {key}")]
    KeyNotFound { key: String },
    #[error("サーバーエラー: {0}")]
    ServerError(String),
    #[error("タイムアウト")]
    Timeout,
}
