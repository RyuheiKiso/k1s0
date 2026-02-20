use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tonic::transport::Channel;

use crate::infrastructure::config::ServiceEndpoint;

/// GrpcStepCaller はSagaステップのgRPC呼び出しトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait GrpcStepCaller: Send + Sync {
    /// サービスのメソッドを呼び出す。
    async fn call_step(
        &self,
        service_name: &str,
        method: &str,
        payload: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value>;
}

/// ServiceRegistry は静的サービスレジストリ。
pub struct ServiceRegistry {
    services: HashMap<String, ServiceEndpoint>,
}

impl ServiceRegistry {
    /// 新しいServiceRegistryを作成する。
    pub fn new(services: HashMap<String, ServiceEndpoint>) -> Self {
        Self { services }
    }

    /// サービス名からエンドポイントURLを解決する。
    pub fn resolve(&self, service_name: &str) -> anyhow::Result<String> {
        self.services
            .get(service_name)
            .map(|ep| format!("http://{}:{}", ep.host, ep.port))
            .ok_or_else(|| anyhow::anyhow!("service not found: {}", service_name))
    }
}

/// TonicGrpcCaller はtonic経由の動的gRPC呼び出し実装。
pub struct TonicGrpcCaller {
    registry: Arc<ServiceRegistry>,
    channels: RwLock<HashMap<String, Channel>>,
}

impl TonicGrpcCaller {
    /// 新しいTonicGrpcCallerを作成する。
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self {
            registry,
            channels: RwLock::new(HashMap::new()),
        }
    }

    /// エンドポイントのチャネルを取得または作成する。
    async fn get_channel(&self, endpoint: &str) -> anyhow::Result<Channel> {
        // Check cache first
        {
            let channels = self.channels.read().await;
            if let Some(channel) = channels.get(endpoint) {
                return Ok(channel.clone());
            }
        }

        // Create new channel
        let channel = Channel::from_shared(endpoint.to_string())?
            .connect()
            .await?;

        // Cache it
        {
            let mut channels = self.channels.write().await;
            channels.insert(endpoint.to_string(), channel.clone());
        }

        Ok(channel)
    }

    /// メソッド名 "ServiceName.MethodName" からgRPCパスを構築する。
    fn build_grpc_path(method: &str) -> anyhow::Result<String> {
        let parts: Vec<&str> = method.splitn(2, '.').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "invalid method format: expected 'ServiceName.MethodName', got '{}'",
                method
            );
        }
        Ok(format!("/{}/{}", parts[0], parts[1]))
    }
}

#[async_trait]
impl GrpcStepCaller for TonicGrpcCaller {
    async fn call_step(
        &self,
        service_name: &str,
        method: &str,
        payload: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let endpoint = self.registry.resolve(service_name)?;
        let channel = self.get_channel(&endpoint).await?;
        let _path = Self::build_grpc_path(method)?;

        // Note: In production, this would use tonic's generic codec to make
        // dynamic gRPC calls. For now, we serialize payload as JSON bytes
        // and send via a generic unary call.
        let _payload_bytes = serde_json::to_vec(payload)?;

        // Placeholder: actual gRPC call would use the channel and path
        // For production, implement with tonic::codec::ProstCodec or similar
        let _ = channel;
        tracing::info!(
            service = service_name,
            method = method,
            "executing gRPC step call"
        );

        // TODO: Implement actual gRPC call with generic codec
        // For now, return the payload as-is to enable testing
        Ok(serde_json::json!({"status": "ok"}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_registry() -> ServiceRegistry {
        let mut services = HashMap::new();
        services.insert(
            "inventory-service".to_string(),
            ServiceEndpoint {
                host: "localhost".to_string(),
                port: 50051,
            },
        );
        services.insert(
            "payment-service".to_string(),
            ServiceEndpoint {
                host: "localhost".to_string(),
                port: 50052,
            },
        );
        ServiceRegistry::new(services)
    }

    #[test]
    fn test_service_registry_resolve() {
        let registry = make_registry();
        assert_eq!(
            registry.resolve("inventory-service").unwrap(),
            "http://localhost:50051"
        );
        assert_eq!(
            registry.resolve("payment-service").unwrap(),
            "http://localhost:50052"
        );
    }

    #[test]
    fn test_service_registry_not_found() {
        let registry = make_registry();
        assert!(registry.resolve("unknown-service").is_err());
    }

    #[test]
    fn test_build_grpc_path() {
        assert_eq!(
            TonicGrpcCaller::build_grpc_path("InventoryService.Reserve").unwrap(),
            "/InventoryService/Reserve"
        );
        assert_eq!(
            TonicGrpcCaller::build_grpc_path("PaymentService.Charge").unwrap(),
            "/PaymentService/Charge"
        );
    }

    #[test]
    fn test_build_grpc_path_invalid() {
        assert!(TonicGrpcCaller::build_grpc_path("InvalidMethod").is_err());
    }

    #[tokio::test]
    async fn test_mock_grpc_step_caller() {
        let mut mock = MockGrpcStepCaller::new();
        mock.expect_call_step()
            .returning(|_, _, _| Ok(serde_json::json!({"reserved": true})));

        let result = mock
            .call_step(
                "inventory-service",
                "InventoryService.Reserve",
                &serde_json::json!({"item_id": "abc"}),
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["reserved"], true);
    }
}
