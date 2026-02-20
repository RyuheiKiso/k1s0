use anyhow::Result;

/// SagaClient はSagaサービスへのgRPCクライアント。
pub struct SagaClient {
    endpoint: String,
}

impl SagaClient {
    /// 新しいSagaClientを作成する。
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
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
        // TODO: Implement gRPC call after proto codegen
        let _ = &self.endpoint;
        let _ = workflow_name;
        let _ = payload;
        let _ = correlation_id;
        let _ = initiated_by;
        anyhow::bail!("gRPC client not yet implemented: waiting for proto codegen")
    }

    /// SagaをIDで取得する。
    pub async fn get_saga(&self, saga_id: &str) -> Result<crate::types::SagaState> {
        let _ = &self.endpoint;
        let _ = saga_id;
        anyhow::bail!("gRPC client not yet implemented: waiting for proto codegen")
    }

    /// Sagaをキャンセルする。
    pub async fn cancel_saga(&self, saga_id: &str) -> Result<()> {
        let _ = &self.endpoint;
        let _ = saga_id;
        anyhow::bail!("gRPC client not yet implemented: waiting for proto codegen")
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
}
