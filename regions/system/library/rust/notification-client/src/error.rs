use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotificationClientError {
    #[error("送信エラー: {0}")]
    SendError(String),
    #[error("バッチ送信エラー: {0}")]
    BatchError(String),
    #[error("無効なチャネル: {0}")]
    InvalidChannel(String),
    #[error("内部エラー: {0}")]
    Internal(String),
}
