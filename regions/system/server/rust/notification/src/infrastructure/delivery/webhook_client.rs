use std::collections::HashMap;
use std::time::Duration;

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
    /// H-18 Webhookタイムアウト対応: 応答タイムアウト30秒・接続タイムアウト5秒を設定して
    /// 外部 Webhook エンドポイントが応答しない場合にリソースが長時間ブロックされることを防ぐ
    pub fn new(
        url: String,
        headers: Option<HashMap<String, String>>,
    ) -> Result<Self, DeliveryError> {
        let client = Client::builder()
            // 応答全体のタイムアウト: 外部エンドポイントの遅延応答によるリソースリークを防ぐ
            .timeout(Duration::from_secs(30))
            // TCP 接続確立のタイムアウト: ネットワーク到達不能なエンドポイントへの長時間待機を防ぐ
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| {
                DeliveryError::Other(format!("HTTPクライアントの初期化に失敗しました: {}", e))
            })?;
        Ok(Self {
            url,
            headers: headers.unwrap_or_default(),
            client,
        })
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn new_without_headers() {
        let client = WebhookDeliveryClient::new("https://example.com/webhook".to_string(), None)
            .expect("クライアント初期化に失敗");
        assert_eq!(client.url, "https://example.com/webhook");
        assert!(client.headers.is_empty());
    }

    #[test]
    fn new_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token".to_string());
        let client =
            WebhookDeliveryClient::new("https://example.com/webhook".to_string(), Some(headers))
                .expect("クライアント初期化に失敗");
        assert_eq!(client.headers.len(), 1);
        assert_eq!(
            client
                .headers
                .get("Authorization")
                .expect("ヘッダーが存在しない"),
            "Bearer token"
        );
    }
}
