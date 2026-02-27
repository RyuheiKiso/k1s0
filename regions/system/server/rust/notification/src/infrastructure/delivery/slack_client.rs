use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::domain::service::{DeliveryClient, DeliveryError};

pub struct SlackDeliveryClient {
    webhook_url: String,
    client: Client,
}

impl SlackDeliveryClient {
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl DeliveryClient for SlackDeliveryClient {
    async fn send(&self, _recipient: &str, subject: &str, body: &str) -> Result<(), DeliveryError> {
        let text = if subject.is_empty() {
            body.to_string()
        } else {
            format!("*{}*\n{}", subject, body)
        };

        let payload = json!({ "text": text });

        let response = self
            .client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| DeliveryError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            return Err(DeliveryError::Rejected(format!(
                "Slack returned {}: {}",
                status, body_text
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_client() {
        let client = SlackDeliveryClient::new("https://hooks.slack.com/services/test".to_string());
        assert_eq!(client.webhook_url, "https://hooks.slack.com/services/test");
    }
}
