// service-catalog クエリリゾルバ。
// service-catalog は REST のみ提供するため ServiceCatalogHttpClient を使用する。

use std::sync::Arc;

use tracing::instrument;

use crate::domain::model::{CatalogServiceConnection, ServiceHealth};
use crate::infrastructure::http::ServiceCatalogHttpClient;

pub struct ServiceCatalogQueryResolver {
    client: Arc<ServiceCatalogHttpClient>,
}

impl ServiceCatalogQueryResolver {
    #[must_use] 
    pub fn new(client: Arc<ServiceCatalogHttpClient>) -> Self {
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
