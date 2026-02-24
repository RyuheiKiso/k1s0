use std::sync::Arc;

use tracing::instrument;

use crate::domain::model::ConfigEntry;
use crate::infra::grpc::ConfigGrpcClient;

pub struct ConfigQueryResolver {
    client: Arc<ConfigGrpcClient>,
}

impl ConfigQueryResolver {
    pub fn new(client: Arc<ConfigGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_config(&self, key: &str) -> anyhow::Result<Option<ConfigEntry>> {
        // key は "namespace/key" 形式を想定
        let parts: Vec<&str> = key.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Ok(None);
        }
        self.client.get_config(parts[0], parts[1]).await
    }
}
