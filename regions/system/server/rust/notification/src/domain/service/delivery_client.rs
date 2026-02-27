use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum DeliveryError {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("delivery rejected: {0}")]
    Rejected(String),

    #[error("delivery error: {0}")]
    Other(String),
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DeliveryClient: Send + Sync {
    async fn send(&self, recipient: &str, subject: &str, body: &str) -> Result<(), DeliveryError>;
}
