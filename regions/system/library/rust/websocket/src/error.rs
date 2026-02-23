use thiserror::Error;

#[derive(Debug, Error)]
pub enum WsError {
    #[error("connection error: {0}")]
    ConnectionError(String),
    #[error("send error: {0}")]
    SendError(String),
    #[error("receive error: {0}")]
    ReceiveError(String),
    #[error("not connected")]
    NotConnected,
    #[error("already connected")]
    AlreadyConnected,
    #[error("closed: {0}")]
    Closed(String),
}
