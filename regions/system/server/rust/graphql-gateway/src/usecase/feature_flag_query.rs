use std::sync::Arc;

use tracing::instrument;

use crate::domain::model::FeatureFlag;
use crate::infra::grpc::FeatureFlagGrpcClient;

pub struct FeatureFlagQueryResolver {
    client: Arc<FeatureFlagGrpcClient>,
}

impl FeatureFlagQueryResolver {
    pub fn new(client: Arc<FeatureFlagGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_feature_flag(&self, key: &str) -> anyhow::Result<Option<FeatureFlag>> {
        self.client.get_flag(key).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_feature_flags(
        &self,
        environment: Option<&str>,
    ) -> anyhow::Result<Vec<FeatureFlag>> {
        self.client.list_flags(environment).await
    }
}
