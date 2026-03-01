use reqwest::Client;

use crate::error::DlqError;
use crate::types::{DlqMessage, ListDlqMessagesResponse, RetryDlqMessageResponse};

/// DlqClient は DLQ 管理サーバーへの HTTP REST クライアント。
pub struct DlqClient {
    endpoint: String,
    http_client: Client,
}

impl DlqClient {
    /// 新しい DlqClient を作成する。
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            http_client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    /// DLQ メッセージ一覧を取得する。
    /// GET /api/v1/dlq/:topic?page=:page&page_size=:page_size
    pub async fn list_messages(
        &self,
        topic: &str,
        page: u32,
        page_size: u32,
    ) -> Result<ListDlqMessagesResponse, DlqError> {
        let url = format!(
            "{}/api/v1/dlq/{}?page={}&page_size={}",
            self.endpoint, topic, page, page_size
        );

        let resp = self.http_client.get(&url).send().await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(DlqError::Api {
                status,
                message: body,
            });
        }

        let result: ListDlqMessagesResponse = resp
            .json()
            .await
            .map_err(|e| DlqError::Deserialize(e.to_string()))?;

        Ok(result)
    }

    /// DLQ メッセージの詳細を取得する。
    /// GET /api/v1/dlq/messages/:id
    pub async fn get_message(&self, message_id: &str) -> Result<DlqMessage, DlqError> {
        let url = format!("{}/api/v1/dlq/messages/{}", self.endpoint, message_id);

        let resp = self.http_client.get(&url).send().await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(DlqError::Api {
                status,
                message: body,
            });
        }

        let result: DlqMessage = resp
            .json()
            .await
            .map_err(|e| DlqError::Deserialize(e.to_string()))?;

        Ok(result)
    }

    /// DLQ メッセージを再処理する。
    /// POST /api/v1/dlq/messages/:id/retry
    pub async fn retry_message(
        &self,
        message_id: &str,
    ) -> Result<RetryDlqMessageResponse, DlqError> {
        let url = format!("{}/api/v1/dlq/messages/{}/retry", self.endpoint, message_id);

        let resp = self
            .http_client
            .post(&url)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(DlqError::Api {
                status,
                message: body,
            });
        }

        let result: RetryDlqMessageResponse = resp
            .json()
            .await
            .map_err(|e| DlqError::Deserialize(e.to_string()))?;

        Ok(result)
    }

    /// DLQ メッセージを削除する。
    /// DELETE /api/v1/dlq/messages/:id
    pub async fn delete_message(&self, message_id: &str) -> Result<(), DlqError> {
        let url = format!("{}/api/v1/dlq/messages/{}", self.endpoint, message_id);

        let resp = self.http_client.delete(&url).send().await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(DlqError::Api {
                status,
                message: body,
            });
        }

        Ok(())
    }

    /// トピック内全メッセージを一括再処理する。
    /// POST /api/v1/dlq/:topic/retry-all
    pub async fn retry_all(&self, topic: &str) -> Result<(), DlqError> {
        let url = format!("{}/api/v1/dlq/{}/retry-all", self.endpoint, topic);

        let resp = self
            .http_client
            .post(&url)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(DlqError::Api {
                status,
                message: body,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = DlqClient::new("http://localhost:8080");
        assert_eq!(client.endpoint, "http://localhost:8080");
    }

    #[test]
    fn test_endpoint_trailing_slash_removed() {
        let client = DlqClient::new("http://localhost:8080/");
        assert_eq!(client.endpoint, "http://localhost:8080");
    }

    #[test]
    fn test_list_messages_url() {
        let client = DlqClient::new("http://dlq-server:8080");
        let expected = format!(
            "{}/api/v1/dlq/orders.dlq.v1?page=1&page_size=20",
            client.endpoint
        );
        assert_eq!(
            expected,
            "http://dlq-server:8080/api/v1/dlq/orders.dlq.v1?page=1&page_size=20"
        );
    }

    #[test]
    fn test_get_message_url() {
        let client = DlqClient::new("http://dlq-server:8080");
        let message_id = "msg-123";
        let expected = format!("{}/api/v1/dlq/messages/{}", client.endpoint, message_id);
        assert_eq!(
            expected,
            "http://dlq-server:8080/api/v1/dlq/messages/msg-123"
        );
    }

    #[test]
    fn test_retry_message_url() {
        let client = DlqClient::new("http://dlq-server:8080");
        let message_id = "msg-456";
        let expected = format!(
            "{}/api/v1/dlq/messages/{}/retry",
            client.endpoint, message_id
        );
        assert_eq!(
            expected,
            "http://dlq-server:8080/api/v1/dlq/messages/msg-456/retry"
        );
    }

    #[test]
    fn test_retry_all_url() {
        let client = DlqClient::new("http://dlq-server:8080");
        let topic = "orders.dlq.v1";
        let expected = format!("{}/api/v1/dlq/{}/retry-all", client.endpoint, topic);
        assert_eq!(
            expected,
            "http://dlq-server:8080/api/v1/dlq/orders.dlq.v1/retry-all"
        );
    }
}
