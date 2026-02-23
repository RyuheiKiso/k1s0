use async_trait::async_trait;

use crate::error::WebhookError;
use crate::payload::WebhookPayload;

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait WebhookClient: Send + Sync {
    async fn send(&self, url: &str, payload: &WebhookPayload) -> Result<u16, WebhookError>;
    async fn send_with_signature(
        &self,
        url: &str,
        payload: &WebhookPayload,
        secret: &str,
    ) -> Result<u16, WebhookError>;
}
