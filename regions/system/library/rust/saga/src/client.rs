use reqwest::Client;

use crate::error::SagaError;
use crate::types::{SagaState, StartSagaRequest, StartSagaResponse};

/// SagaClient は Saga サービスへの HTTP REST クライアント。
pub struct SagaClient {
    endpoint: String,
    http_client: Client,
}

impl SagaClient {
    /// 新しい SagaClient を作成する。
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            http_client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Saga を開始する。
    pub async fn start_saga(
        &self,
        request: &StartSagaRequest,
    ) -> Result<StartSagaResponse, SagaError> {
        let resp = self
            .http_client
            .post(format!("{}/api/v1/sagas", self.endpoint))
            .json(request)
            .send()
            .await
            .map_err(|e| SagaError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            let status_code = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(SagaError::ApiError {
                status_code,
                message: body,
            });
        }

        resp.json()
            .await
            .map_err(|e| SagaError::DeserializeError(e.to_string()))
    }

    /// Saga を ID で取得する。
    pub async fn get_saga(&self, saga_id: &str) -> Result<SagaState, SagaError> {
        let resp = self
            .http_client
            .get(format!("{}/api/v1/sagas/{}", self.endpoint, saga_id))
            .send()
            .await
            .map_err(|e| SagaError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            let status_code = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(SagaError::ApiError {
                status_code,
                message: body,
            });
        }

        // レスポンスは `saga` キーでラップされている場合がある
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SagaError::DeserializeError(e.to_string()))?;

        let state_value = json.get("saga").cloned().unwrap_or(json);
        serde_json::from_value(state_value)
            .map_err(|e| SagaError::DeserializeError(e.to_string()))
    }

    /// Saga をキャンセルする。
    pub async fn cancel_saga(&self, saga_id: &str) -> Result<(), SagaError> {
        let resp = self
            .http_client
            .post(format!("{}/api/v1/sagas/{}/cancel", self.endpoint, saga_id))
            .json(&serde_json::json!({}))
            .send()
            .await
            .map_err(|e| SagaError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            let status_code = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(SagaError::ApiError {
                status_code,
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
