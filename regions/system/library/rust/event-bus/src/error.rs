use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventBusError {
    #[error("ハンドラーエラー: {0}")]
    HandlerError(String),
    #[error("イベント発行エラー: {0}")]
    PublishError(String),
    #[error("内部エラー: {0}")]
    Internal(String),
}
