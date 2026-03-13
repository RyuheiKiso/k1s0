use std::sync::Arc;
use tracing::instrument;
use crate::domain::model::{SecretMetadata, VaultAuditLogEntry};
use crate::infrastructure::grpc::VaultGrpcClient;

pub struct VaultQueryResolver {
    client: Arc<VaultGrpcClient>,
}

impl VaultQueryResolver {
    pub fn new(client: Arc<VaultGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_secret_metadata(&self, path: &str) -> anyhow::Result<Option<SecretMetadata>> {
        self.client.get_secret_metadata(path).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_secrets(&self, prefix: Option<&str>) -> anyhow::Result<Vec<String>> {
        self.client.list_secrets(prefix).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_audit_logs(
        &self,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> anyhow::Result<Vec<VaultAuditLogEntry>> {
        self.client.list_audit_logs(offset, limit).await
    }
}
