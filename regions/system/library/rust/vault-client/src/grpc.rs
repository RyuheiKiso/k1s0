use async_trait::async_trait;

use crate::client::VaultClient;
use crate::error::VaultError;
use crate::secret::{Secret, SecretRotatedEvent};

/// gRPC-based vault client.
///
/// Note: transport channel wiring is intentionally thin here to keep this crate transport-agnostic.
pub struct GrpcVaultClient {
    endpoint: String,
}

impl GrpcVaultClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

#[async_trait]
impl VaultClient for GrpcVaultClient {
    async fn get_secret(&self, _path: &str) -> Result<Secret, VaultError> {
        Err(VaultError::ServerError(
            "grpc client is not wired to generated protobuf client yet".to_string(),
        ))
    }

    async fn get_secret_value(&self, _path: &str, _key: &str) -> Result<String, VaultError> {
        Err(VaultError::ServerError(
            "grpc client is not wired to generated protobuf client yet".to_string(),
        ))
    }

    async fn list_secrets(&self, _path_prefix: &str) -> Result<Vec<String>, VaultError> {
        Err(VaultError::ServerError(
            "grpc client is not wired to generated protobuf client yet".to_string(),
        ))
    }

    async fn watch_secret(
        &self,
        _path: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<SecretRotatedEvent>, VaultError> {
        Err(VaultError::ServerError(
            "grpc client is not wired to generated protobuf client yet".to_string(),
        ))
    }
}
