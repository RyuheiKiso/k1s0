use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::domain::service::{DeliveryClient, DeliveryError};

pub struct PushDeliveryClient {
    endpoint: String,
    auth_token: Option<String>,
    client: Client,
}

impl PushDeliveryClient {
    #[must_use] 
    pub fn new(endpoint: String, auth_token: Option<String>) -> Self {
        Self {
            endpoint,
            auth_token,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl DeliveryClient for PushDeliveryClient {
    async fn send(&self, recipient: &str, subject: &str, body: &str) -> Result<(), DeliveryError> {
        let payload = json!({
            "recipient": recipient,
            "title": subject,
            "body": body,
        });

        let mut request = self.client.post(&self.endpoint).json(&payload);
        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
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
                "Push provider returned {status}: {body_text}"
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_push_client() {
        let client = PushDeliveryClient::new(
            "https://example.com/push".to_string(),
            Some("token".to_string()),
        );
        assert_eq!(client.endpoint, "https://example.com/push");
        assert_eq!(client.auth_token.as_deref(), Some("token"));
    }
}
