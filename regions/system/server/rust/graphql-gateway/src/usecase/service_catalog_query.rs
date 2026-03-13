use std::sync::Arc;

use tracing::instrument;

use crate::domain::model::{CatalogServiceConnection, ServiceHealth};
use crate::infrastructure::grpc::ServiceCatalogGrpcClient;

pub struct ServiceCatalogQueryResolver {
    client: Arc<ServiceCatalogGrpcClient>,
}

impl ServiceCatalogQueryResolver {
    pub fn new(client: Arc<ServiceCatalogGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_service(
        &self,
        service_id: &str,
    ) -> anyhow::Result<Option<crate::domain::model::CatalogService>> {
        self.client.get_service(service_id).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_services(
        &self,
        first: Option<i32>,
        tier: Option<&str>,
        status: Option<&str>,
        search: Option<&str>,
    ) -> anyhow::Result<CatalogServiceConnection> {
        let page_size = first.unwrap_or(20);
        self.client
            .list_services(1, page_size, tier, status, search)
            .await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn health_check(
        &self,
        service_id: Option<&str>,
    ) -> anyhow::Result<Vec<ServiceHealth>> {
        self.client.health_check(service_id).await
    }
}
