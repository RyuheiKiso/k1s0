use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::{Buf, BufMut};
use http::uri::PathAndQuery;
use tokio::sync::RwLock;
use tonic::codec::{Codec, DecodeBuf, EncodeBuf, Encoder, Decoder};
use tonic::transport::Channel;

use crate::infrastructure::config::ServiceEndpoint;

/// JSON codec for generic gRPC calls using raw bytes.
#[derive(Debug, Clone, Copy, Default)]
struct JsonCodec;

#[derive(Debug)]
struct JsonEncoder;

#[derive(Debug)]
struct JsonDecoder;

impl Encoder for JsonEncoder {
    type Item = Vec<u8>;
    type Error = tonic::Status;

    fn encode(&mut self, item: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        dst.put_slice(&item);
        Ok(())
    }
}

impl Decoder for JsonDecoder {
    type Item = Vec<u8>;
    type Error = tonic::Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let remaining = src.remaining();
        if remaining == 0 {
            return Ok(None);
        }
        let mut buf = vec![0u8; remaining];
        src.copy_to_slice(&mut buf);
        Ok(Some(buf))
    }
}

impl Codec for JsonCodec {
    type Encode = Vec<u8>;
    type Decode = Vec<u8>;
    type Encoder = JsonEncoder;
    type Decoder = JsonDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        JsonEncoder
    }

    fn decoder(&mut self) -> Self::Decoder {
        JsonDecoder
    }
}

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
        let path_str = Self::build_grpc_path(method)?;

        let payload_bytes = serde_json::to_vec(payload)?;

        tracing::info!(
            service = service_name,
            method = method,
            "executing gRPC step call"
        );

        let path = PathAndQuery::try_from(path_str)
            .map_err(|e| anyhow::anyhow!("invalid gRPC path: {}", e))?;

        let mut client = tonic::client::Grpc::new(channel);
        client.ready().await.map_err(|e| anyhow::anyhow!("gRPC not ready: {}", e))?;

        let request = tonic::Request::new(payload_bytes);
        let response = client
            .unary(request, path, JsonCodec)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC call failed: {}", e))?;

        let response_bytes = response.into_inner();
        let result: serde_json::Value = serde_json::from_slice(&response_bytes)
            .map_err(|e| anyhow::anyhow!("failed to deserialize gRPC response: {}", e))?;

        Ok(result)
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
