use std::sync::Arc;

use tracing::instrument;

use crate::domain::model::Navigation;
use crate::infrastructure::grpc::NavigationGrpcClient;

pub struct NavigationQueryResolver {
    client: Arc<NavigationGrpcClient>,
}

impl NavigationQueryResolver {
    pub fn new(client: Arc<NavigationGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_navigation(&self, bearer_token: &str) -> anyhow::Result<Navigation> {
        self.client.get_navigation(bearer_token).await
    }
}
