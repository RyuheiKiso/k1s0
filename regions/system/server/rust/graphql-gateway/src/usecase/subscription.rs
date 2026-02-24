use std::sync::Arc;

use async_graphql::futures_util::Stream;
use tracing::instrument;

use crate::domain::model::ConfigEntry;
use crate::infra::grpc::ConfigGrpcClient;

pub struct SubscriptionResolver {
    config_client: Arc<ConfigGrpcClient>,
}

impl SubscriptionResolver {
    pub fn new(config_client: Arc<ConfigGrpcClient>) -> Self {
        Self { config_client }
    }

    /// WatchConfig ストリームを返す。設定変更が発生するたびに ConfigEntry を配信する。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn watch_config(
        &self,
        namespaces: Vec<String>,
    ) -> impl Stream<Item = ConfigEntry> {
        self.config_client.watch_config(namespaces).await
    }
}
