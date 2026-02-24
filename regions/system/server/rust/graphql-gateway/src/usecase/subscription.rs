use std::sync::Arc;
use std::time::Duration;

use async_graphql::futures_util::Stream;
use tracing::instrument;

use crate::domain::model::{ConfigEntry, FeatureFlag, Tenant};
use crate::infra::grpc::{ConfigGrpcClient, FeatureFlagGrpcClient, TenantGrpcClient};

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
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn watch_config(
        &self,
        namespaces: Vec<String>,
    ) -> impl Stream<Item = ConfigEntry> {
        self.config_client.watch_config(namespaces).await
    }

    /// テナント更新をポーリングで監視するストリームを返す。
    /// gRPC サーバー側にストリーミング RPC が追加された場合はそちらに切り替える。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub fn watch_tenant_updated(
        &self,
        tenant_id: String,
    ) -> impl Stream<Item = Tenant> {
        let client = self.tenant_client.clone();
        async_graphql::futures_util::stream::unfold(
            (client, tenant_id),
            |(client, tenant_id)| async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    match client.get_tenant(&tenant_id).await {
                        Ok(Some(tenant)) => return Some((tenant, (client, tenant_id))),
                        Ok(None) => continue,
                        Err(_) => continue,
                    }
                }
            },
        )
    }

    /// フィーチャーフラグ変更をポーリングで監視するストリームを返す。
    /// gRPC サーバー側にストリーミング RPC が追加された場合はそちらに切り替える。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub fn watch_feature_flag_changed(
        &self,
        key: String,
    ) -> impl Stream<Item = FeatureFlag> {
        let client = self.feature_flag_client.clone();
        async_graphql::futures_util::stream::unfold(
            (client, key),
            |(client, key)| async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    match client.get_flag(&key).await {
                        Ok(Some(flag)) => return Some((flag, (client, key))),
                        Ok(None) => continue,
                        Err(_) => continue,
                    }
                }
            },
        )
    }
}
