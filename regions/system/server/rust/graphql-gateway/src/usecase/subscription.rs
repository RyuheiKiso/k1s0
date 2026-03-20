use std::sync::Arc;

use async_graphql::futures_util::Stream;
use tracing::instrument;

use crate::domain::model::{ConfigEntry, FeatureFlag, Tenant};
use crate::infrastructure::grpc::{ConfigGrpcClient, FeatureFlagGrpcClient, TenantGrpcClient};

pub struct SubscriptionResolver {
    config_client: Arc<ConfigGrpcClient>,
    tenant_client: Arc<TenantGrpcClient>,
    feature_flag_client: Arc<FeatureFlagGrpcClient>,
}

impl SubscriptionResolver {
    pub fn new(
        config_client: Arc<ConfigGrpcClient>,
        tenant_client: Arc<TenantGrpcClient>,
        feature_flag_client: Arc<FeatureFlagGrpcClient>,
    ) -> Self {
        Self {
            config_client,
            tenant_client,
            feature_flag_client,
        }
    }

    /// WatchConfig ストリームを返す。設定変更が発生するたびに ConfigEntry を配信する。
    /// 接続失敗時は anyhow::Error を返し、呼び出し元で適切にハンドリングする。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn watch_config(&self, namespaces: Vec<String>) -> anyhow::Result<impl Stream<Item = ConfigEntry>> {
        self.config_client.watch_config(namespaces).await
    }

    /// WatchTenant ストリームを返す。テナント変更が発生するたびに Tenant を配信する。
    /// 接続失敗時は anyhow::Error を返し、呼び出し元で適切にハンドリングする。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn watch_tenant_updated(&self, tenant_id: String) -> anyhow::Result<impl Stream<Item = Tenant>> {
        self.tenant_client.watch_tenant(&tenant_id).await
    }

    /// WatchFeatureFlag ストリームを返す。フラグ変更が発生するたびに FeatureFlag を配信する。
    /// 接続失敗時は anyhow::Error を返し、呼び出し元で適切にハンドリングする。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn watch_feature_flag_changed(&self, key: String) -> anyhow::Result<impl Stream<Item = FeatureFlag>> {
        self.feature_flag_client.watch_feature_flag(&key).await
    }
}
