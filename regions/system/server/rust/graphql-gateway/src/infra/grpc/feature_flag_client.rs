use std::collections::HashSet;
use std::time::Duration;

use async_graphql::futures_util::Stream;
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
use proto::k1s0::system::featureflag::v1::FeatureFlag as ProtoFeatureFlag;

pub struct FeatureFlagGrpcClient {
    client: FeatureFlagServiceClient<Channel>,
}

impl FeatureFlagGrpcClient {
    fn rollout_to_variants(
        rollout_percentage: Option<i32>,
    ) -> Vec<proto::k1s0::system::featureflag::v1::FlagVariant> {
        let Some(rollout) = rollout_percentage else {
            return vec![];
        };
        let on_weight = rollout.clamp(0, 100);
        let off_weight = 100 - on_weight;
        vec![
            proto::k1s0::system::featureflag::v1::FlagVariant {
                name: "on".to_string(),
                value: "true".to_string(),
                weight: on_weight,
            },
            proto::k1s0::system::featureflag::v1::FlagVariant {
                name: "off".to_string(),
                value: "false".to_string(),
                weight: off_weight,
            },
        ]
    }

    fn target_env_to_rules(
        target_environments: Option<Vec<String>>,
    ) -> Vec<proto::k1s0::system::featureflag::v1::FlagRule> {
        target_environments
            .unwrap_or_default()
            .into_iter()
            .filter(|env| !env.trim().is_empty())
            .map(|env| proto::k1s0::system::featureflag::v1::FlagRule {
                attribute: "environment".to_string(),
                operator: "equals".to_string(),
                value: env,
                variant: "on".to_string(),
            })
            .collect()
    }

    fn to_domain_flag(
        flag: ProtoFeatureFlag,
        rollout_hint: Option<i32>,
        targets_hint: Option<Vec<String>>,
    ) -> FeatureFlag {
        let inferred_targets: Vec<String> = flag
            .rules
            .iter()
            .filter(|r| r.attribute == "environment")
            .map(|r| r.value.clone())
            .collect();
        let inferred_rollout = if !flag.enabled {
            0
        } else {
            flag.variants
                .iter()
                .map(|v| v.weight)
                .max()
                .unwrap_or(100)
                .clamp(0, 100)
        };

        FeatureFlag {
            key: flag.flag_key.clone(),
            name: flag.description.clone(),
            enabled: flag.enabled,
            rollout_percentage: rollout_hint.unwrap_or(inferred_rollout),
            target_environments: targets_hint.unwrap_or(inferred_targets),
        }
    }

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
        let request = tonic::Request::new(proto::k1s0::system::featureflag::v1::GetFlagRequest {
            flag_key: key.to_owned(),
        });

        match self.client.clone().get_flag(request).await {
            Ok(resp) => {
                let flag = match resp.into_inner().flag {
                    Some(f) => f,
                    None => return Ok(None),
                };
                Ok(Some(Self::to_domain_flag(flag, None, None)))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!("FeatureFlagService.GetFlag failed: {}", e)),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_flags(&self, environment: Option<&str>) -> anyhow::Result<Vec<FeatureFlag>> {
        let resp = self
            .client
            .clone()
            .list_flags(tonic::Request::new(
                proto::k1s0::system::featureflag::v1::ListFlagsRequest {},
            ))
            .await
            .map_err(|e| anyhow::anyhow!("FeatureFlagService.ListFlags failed: {}", e))?
            .into_inner();

        let mut flags: Vec<FeatureFlag> = resp
            .flags
            .into_iter()
            .map(|f| Self::to_domain_flag(f, None, None))
            .collect();

        if let Some(env) = environment {
            flags.retain(|f| {
                f.target_environments.is_empty() || f.target_environments.iter().any(|e| e == env)
            });
        }

        Ok(flags)
    }

    /// DataLoader 向け: 複数キーをまとめて取得
    pub async fn list_flags_by_keys(&self, keys: &[String]) -> anyhow::Result<Vec<FeatureFlag>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }

        let key_set: HashSet<&str> = keys.iter().map(String::as_str).collect();
        let all_flags = self.list_flags(None).await?;
        Ok(all_flags
            .into_iter()
            .filter(|f| key_set.contains(f.key.as_str()))
            .collect())
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn set_flag(
        &self,
        key: &str,
        enabled: bool,
        rollout_percentage: Option<i32>,
        target_environments: Option<Vec<String>>,
    ) -> anyhow::Result<FeatureFlag> {
        let request =
            tonic::Request::new(proto::k1s0::system::featureflag::v1::UpdateFlagRequest {
                flag_key: key.to_owned(),
                enabled: Some(enabled),
                description: Some(String::new()),
                rules: Self::target_env_to_rules(target_environments.clone()),
                variants: Self::rollout_to_variants(rollout_percentage),
            });

        let flag = self
            .client
            .clone()
            .update_flag(request)
            .await
            .map_err(|e| anyhow::anyhow!("FeatureFlagService.UpdateFlag failed: {}", e))?
            .into_inner()
            .flag
            .ok_or_else(|| anyhow::anyhow!("empty flag in response"))?;

        Ok(Self::to_domain_flag(
            flag,
            rollout_percentage,
            target_environments,
        ))
    }

    /// WatchFeatureFlag Server-Side Streaming を購読し、変更イベントを FeatureFlag として返す。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn watch_feature_flag(&self, key: &str) -> impl Stream<Item = FeatureFlag> {
        let request = tonic::Request::new(
            proto::k1s0::system::featureflag::v1::WatchFeatureFlagRequest {
                flag_key: key.to_owned(),
            },
        );

        let stream = self
            .client
            .clone()
            .watch_feature_flag(request)
            .await
            .expect("WatchFeatureFlag stream failed")
            .into_inner();

        async_graphql::futures_util::stream::unfold(stream, |mut stream| async move {
            match stream.message().await {
                Ok(Some(resp)) => {
                    let flag = resp
                        .flag
                        .map(|f| Self::to_domain_flag(f, None, None))
                        .unwrap_or(FeatureFlag {
                            key: resp.flag_key,
                            name: String::new(),
                            enabled: false,
                            rollout_percentage: 0,
                            target_environments: vec![],
                        });
                    Some((flag, stream))
                }
                _ => None,
            }
        })
    }
}
