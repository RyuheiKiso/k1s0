use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::domain::service::{DeliveryClient, DeliveryError};

pub struct SmsDeliveryClient {
    endpoint: String,
    api_key: Option<String>,
    client: Client,
}

impl SmsDeliveryClient {
    #[must_use]
    pub fn new(endpoint: String, api_key: Option<String>) -> Self {
        Self {
            endpoint,
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl DeliveryClient for SmsDeliveryClient {
    async fn send(&self, recipient: &str, subject: &str, body: &str) -> Result<(), DeliveryError> {
        let payload = json!({
            "to": recipient,
            "subject": subject,
            "message": body,
        });

        let mut request = self.client.post(&self.endpoint).json(&payload);
        if let Some(api_key) = &self.api_key {
            request = request.bearer_auth(api_key);
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
                "SMS provider returned {status}: {body_text}"
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_sms_client() {
        let client = SmsDeliveryClient::new(
            "https://example.com/sms".to_string(),
            Some("secret".to_string()),
        );
        assert_eq!(client.endpoint, "https://example.com/sms");
        assert_eq!(client.api_key.as_deref(), Some("secret"));
    }
}
