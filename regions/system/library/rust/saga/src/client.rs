use anyhow::Result;
use reqwest::Client;

/// SagaClient はSagaサービスへのHTTP RESTクライアント。
pub struct SagaClient {
    endpoint: String,
    http_client: Client,
}

impl SagaClient {
    /// 新しいSagaClientを作成する。
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            http_client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Sagaを開始する。
    pub async fn start_saga(
        &self,
        workflow_name: &str,
        payload: &serde_json::Value,
        correlation_id: Option<&str>,
        initiated_by: Option<&str>,
    ) -> Result<crate::types::StartSagaResponse> {
        let body = serde_json::json!({
            "workflow_name": workflow_name,
            "payload": payload,
            "correlation_id": correlation_id,
            "initiated_by": initiated_by,
        });

        let resp = self
            .http_client
            .post(format!("{}/api/v1/sagas", self.endpoint))
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("saga HTTP request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("saga server returned {}: {}", status, body);
        }

        let result: crate::types::StartSagaResponse = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("failed to parse saga response: {}", e))?;

        Ok(result)
    }

    /// SagaをIDで取得する。
    pub async fn get_saga(&self, saga_id: &str) -> Result<crate::types::SagaState> {
        let resp = self
            .http_client
            .get(format!("{}/api/v1/sagas/{}", self.endpoint, saga_id))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("saga HTTP request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("saga server returned {}: {}", status, body);
        }

        // Response has `saga` key wrapping the object
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("failed to parse saga response: {}", e))?;

        let state: crate::types::SagaState = serde_json::from_value(
            json.get("saga").cloned().unwrap_or(json),
        )
        .map_err(|e| anyhow::anyhow!("failed to deserialize saga state: {}", e))?;

        Ok(state)
    }

    /// Sagaをキャンセルする。
    pub async fn cancel_saga(&self, saga_id: &str) -> Result<()> {
        let resp = self
            .http_client
            .post(format!("{}/api/v1/sagas/{}/cancel", self.endpoint, saga_id))
            .json(&serde_json::json!({}))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("saga HTTP request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("saga server returned {}: {}", status, body);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = SagaClient::new("http://localhost:50051");
        assert_eq!(client.endpoint, "http://localhost:50051");
    }

    #[test]
    fn test_endpoint_trailing_slash_removed() {
        let client = SagaClient::new("http://localhost:8080/");
        assert_eq!(client.endpoint, "http://localhost:8080");
    }

    #[test]
    fn test_start_saga_url() {
        let client = SagaClient::new("http://saga-server:8080");
        let expected_url = format!("{}/api/v1/sagas", client.endpoint);
        assert_eq!(expected_url, "http://saga-server:8080/api/v1/sagas");
    }

    #[test]
    fn test_get_saga_url() {
        let client = SagaClient::new("http://saga-server:8080");
        let saga_id = "550e8400-e29b-41d4-a716-446655440000";
        let expected_url = format!("{}/api/v1/sagas/{}", client.endpoint, saga_id);
        assert_eq!(
            expected_url,
            "http://saga-server:8080/api/v1/sagas/550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn test_cancel_saga_url() {
        let client = SagaClient::new("http://saga-server:8080");
        let saga_id = "test-id";
        let expected_url = format!("{}/api/v1/sagas/{}/cancel", client.endpoint, saga_id);
        assert_eq!(
            expected_url,
            "http://saga-server:8080/api/v1/sagas/test-id/cancel"
        );
    }
}
