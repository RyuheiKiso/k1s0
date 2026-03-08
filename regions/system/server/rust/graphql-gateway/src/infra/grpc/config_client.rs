use std::time::Duration;

use async_graphql::futures_util::Stream;
use chrono::{DateTime, Utc};
use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::model::ConfigEntry;
use crate::infra::config::BackendConfig;

#[allow(dead_code)]
pub mod proto {
    pub mod k1s0 {
        pub mod system {
            pub mod common {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.common.v1");
                }
            }
            pub mod config {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.config.v1");
                }
            }
        }
    }
}

use proto::k1s0::system::config::v1::config_service_client::ConfigServiceClient;

pub struct ConfigGrpcClient {
    client: ConfigServiceClient<Channel>,
}

impl ConfigGrpcClient {
    pub async fn connect(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect()
            .await?;
        Ok(Self {
            client: ConfigServiceClient::new(channel),
        })
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_config(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Option<ConfigEntry>> {
        let request = tonic::Request::new(proto::k1s0::system::config::v1::GetConfigRequest {
            namespace: namespace.to_owned(),
            key: key.to_owned(),
        });

        match self.client.clone().get_config(request).await {
            Ok(resp) => {
                let entry = match resp.into_inner().entry {
                    Some(e) => e,
                    None => return Ok(None),
                };
                let value_str = String::from_utf8(entry.value).unwrap_or_default();
                Ok(Some(ConfigEntry {
                    key: format!("{}/{}", entry.namespace, entry.key),
                    value: value_str,
                    updated_at: timestamp_to_rfc3339(entry.updated_at),
                }))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!("ConfigService.GetConfig failed: {}", e)),
        }
    }

    /// WatchConfig Server-Side Streaming を購読し、変更イベントを ConfigEntry として返す。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn watch_config(&self, namespaces: Vec<String>) -> impl Stream<Item = ConfigEntry> {
        let request =
            tonic::Request::new(proto::k1s0::system::config::v1::WatchConfigRequest { namespaces });

        let stream = self
            .client
            .clone()
            .watch_config(request)
            .await
            .expect("WatchConfig stream failed")
            .into_inner();

        async_graphql::futures_util::stream::unfold(stream, |mut stream| async move {
            match stream.message().await {
                Ok(Some(resp)) => {
                    let value_str = String::from_utf8(resp.new_value).unwrap_or_default();
                    let entry = ConfigEntry {
                        key: format!("{}/{}", resp.namespace, resp.key),
                        value: value_str,
                        updated_at: timestamp_to_rfc3339(resp.changed_at),
                    };
                    Some((entry, stream))
                }
                _ => None,
            }
        })
    }
}

fn timestamp_to_rfc3339(ts: Option<proto::k1s0::system::common::v1::Timestamp>) -> String {
    ts.and_then(|ts| DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32))
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}
