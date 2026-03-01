use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventBusError {
    #[error("publish failed: {0}")]
    PublishFailed(String),
    #[error("handler failed: {0}")]
    HandlerFailed(String),
    #[error("channel closed")]
    ChannelClosed,
}
