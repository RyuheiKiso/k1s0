use std::collections::HashMap;

use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::domain::service::{DeliveryClient, DeliveryError};

pub struct WebhookDeliveryClient {
    url: String,
    headers: HashMap<String, String>,
    client: Client,
}

impl WebhookDeliveryClient {
    pub fn new(url: String, headers: Option<HashMap<String, String>>) -> Self {
        Self {
            url,
            headers: headers.unwrap_or_default(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl DeliveryClient for WebhookDeliveryClient {
    async fn send(&self, recipient: &str, subject: &str, body: &str) -> Result<(), DeliveryError> {
        let payload = json!({
            "recipient": recipient,
            "subject": subject,
            "body": body,
        });

        let mut request = self.client.post(&self.url).json(&payload);

        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        let response = request
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
                "Webhook returned {}: {}",
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
    fn new_without_headers() {
        let client = WebhookDeliveryClient::new("https://example.com/webhook".to_string(), None);
        assert_eq!(client.url, "https://example.com/webhook");
        assert!(client.headers.is_empty());
    }

    #[test]
    fn new_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token".to_string());
        let client = WebhookDeliveryClient::new(
            "https://example.com/webhook".to_string(),
            Some(headers),
        );
        assert_eq!(client.headers.len(), 1);
        assert_eq!(
            client.headers.get("Authorization").unwrap(),
            "Bearer token"
        );
    }
}
