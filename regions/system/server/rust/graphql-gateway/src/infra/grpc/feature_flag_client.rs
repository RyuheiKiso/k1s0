use std::time::Duration;

use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::model::FeatureFlag;
use crate::infra::config::BackendConfig;

pub mod proto {
    pub mod k1s0 {
        pub mod system {
            pub mod common {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.common.v1");
                }
            }
            pub mod featureflag {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.featureflag.v1");
                }
            }
        }
    }
}

use proto::k1s0::system::featureflag::v1::feature_flag_service_client::FeatureFlagServiceClient;

pub struct FeatureFlagGrpcClient {
    client: FeatureFlagServiceClient<Channel>,
}

impl FeatureFlagGrpcClient {
    pub async fn connect(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect()
            .await?;
        Ok(Self {
            client: FeatureFlagServiceClient::new(channel),
        })
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_flag(&self, key: &str) -> anyhow::Result<Option<FeatureFlag>> {
        let request = tonic::Request::new(
            proto::k1s0::system::featureflag::v1::GetFlagRequest {
                flag_key: key.to_owned(),
            },
        );

        match self.client.clone().get_flag(request).await {
            Ok(resp) => {
                let flag = match resp.into_inner().flag {
                    Some(f) => f,
                    None => return Ok(None),
                };
                Ok(Some(FeatureFlag {
                    key: flag.flag_key.clone(),
                    name: flag.description.clone(),
                    enabled: flag.enabled,
                    rollout_percentage: 0,
                    target_environments: vec![],
                }))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!(
                "FeatureFlagService.GetFlag failed: {}",
                e
            )),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_flags(
        &self,
        _environment: Option<&str>,
    ) -> anyhow::Result<Vec<FeatureFlag>> {
        // FeatureFlagService に ListFlags RPC がないため、空リストを返す
        Ok(vec![])
    }

    /// DataLoader 向け: 複数キーをまとめて取得
    pub async fn list_flags_by_keys(&self, _keys: &[String]) -> anyhow::Result<Vec<FeatureFlag>> {
        Ok(vec![])
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn set_flag(
        &self,
        key: &str,
        enabled: bool,
        _rollout_percentage: Option<i32>,
        _target_environments: Option<Vec<String>>,
    ) -> anyhow::Result<FeatureFlag> {
        let request = tonic::Request::new(
            proto::k1s0::system::featureflag::v1::UpdateFlagRequest {
                flag_key: key.to_owned(),
                enabled,
                description: String::new(),
            },
        );

        let flag = self
            .client
            .clone()
            .update_flag(request)
            .await
            .map_err(|e| anyhow::anyhow!("FeatureFlagService.UpdateFlag failed: {}", e))?
            .into_inner()
            .flag
            .ok_or_else(|| anyhow::anyhow!("empty flag in response"))?;

        Ok(FeatureFlag {
            key: flag.flag_key.clone(),
            name: flag.description.clone(),
            enabled: flag.enabled,
            rollout_percentage: 0,
            target_environments: vec![],
        })
    }
}
