use std::sync::Arc;

use tracing::instrument;

use crate::domain::model::{Tenant, TenantConnection};
use crate::infra::grpc::TenantGrpcClient;

pub struct TenantQueryResolver {
    client: Arc<TenantGrpcClient>,
}

impl TenantQueryResolver {
    pub fn new(client: Arc<TenantGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_tenant(&self, id: &str) -> anyhow::Result<Option<Tenant>> {
        self.client.get_tenant(id).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_tenants(
        &self,
        page: i32,
        page_size: i32,
    ) -> anyhow::Result<TenantConnection> {
        self.client.list_tenants(page, page_size).await
    }
}
