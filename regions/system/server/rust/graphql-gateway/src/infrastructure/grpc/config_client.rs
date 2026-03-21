use std::time::Duration;

use async_graphql::futures_util::Stream;
use chrono::{DateTime, Utc};
use tonic::transport::Channel;
use tracing::instrument;
use tracing::warn;

use crate::domain::model::ConfigEntry;
use crate::infrastructure::config::BackendConfig;

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
    /// バックエンド設定からクライアントを生成する。
    /// connect_lazy() により起動時の接続確立を不要とし、実際のRPC呼び出し時に接続する。
    pub fn new(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect_lazy();
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
                // UTF-8 デコード失敗時はエラーを伝播する（サイレントな空文字列化を避ける）
                let value_str = String::from_utf8(entry.value).map_err(|e| {
                    warn!(namespace = %entry.namespace, key = %entry.key, "config value is not valid UTF-8: {}", e);
                    anyhow::anyhow!("config value is not valid UTF-8: {}", e)
                })?;
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

    /// 複数の "namespace/key" キーに対して ListConfigs をバッチ呼び出しし、該当する ConfigEntry を返す。
    ///
    /// namespace ごとにグルーピングし、1回の ListConfigs RPC で該当エントリを取得する。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_configs_by_keys(&self, keys: &[String]) -> anyhow::Result<Vec<ConfigEntry>> {
        // namespace ごとにキーをグルーピング
        let mut ns_keys: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for key in keys {
            let parts: Vec<&str> = key.splitn(2, '/').collect();
            if parts.len() == 2 {
                ns_keys
                    .entry(parts[0].to_owned())
                    .or_default()
                    .push(parts[1].to_owned());
            }
        }

        let mut results = Vec::new();
        for (namespace, target_keys) in &ns_keys {
            let request =
                tonic::Request::new(proto::k1s0::system::config::v1::ListConfigsRequest {
                    namespace: namespace.clone(),
                    pagination: None,
                    search: String::new(),
                });
            match self.client.clone().list_configs(request).await {
                Ok(resp) => {
                    let target_set: std::collections::HashSet<&str> =
                        target_keys.iter().map(|s| s.as_str()).collect();
                    for entry in resp.into_inner().entries {
                        if target_set.contains(entry.key.as_str()) {
                            // UTF-8 デコード失敗時はエラーを伝播する（サイレントな空文字列化を避ける）
                            let value_str = String::from_utf8(entry.value).map_err(|e| {
                                warn!(namespace = %entry.namespace, key = %entry.key, "config value is not valid UTF-8: {}", e);
                                anyhow::anyhow!("config value is not valid UTF-8: {}", e)
                            })?;
                            results.push(ConfigEntry {
                                key: format!("{}/{}", entry.namespace, entry.key),
                                value: value_str,
                                updated_at: timestamp_to_rfc3339(entry.updated_at),
                            });
                        }
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("ConfigService.ListConfigs failed: {}", e));
                }
            }
        }
        Ok(results)
    }

    /// WatchConfig Server-Side Streaming を購読し、変更イベントを ConfigEntry として返す。
    /// ストリームアイテムは `Result<ConfigEntry, tonic::Status>` として返し、
    /// 接続中の gRPC エラーをサブスクライバーに伝播する（P2-26）。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn watch_config(
        &self,
        namespaces: Vec<String>,
    ) -> anyhow::Result<impl Stream<Item = Result<ConfigEntry, tonic::Status>>> {
        let request =
            tonic::Request::new(proto::k1s0::system::config::v1::WatchConfigRequest { namespaces });

        // gRPC ストリーム接続を確立し、失敗時はエラーを返す（パニックしない）
        let stream = self
            .client
            .clone()
            .watch_config(request)
            .await?
            .into_inner();

        // unfold のクロージャ内で loop を使い、UTF-8 デコード失敗時はスキップして次メッセージを処理する。
        // ストリームエラー（tonic::Status）は Err() としてサブスクライバーに伝播し、
        // 切断理由が不明のまま購読が終了するのを防ぐ。
        Ok(async_graphql::futures_util::stream::unfold(
            stream,
            |mut stream| async move {
                loop {
                    match stream.message().await {
                        Ok(Some(resp)) => {
                            match String::from_utf8(resp.new_value) {
                                Ok(value_str) => {
                                    let entry = ConfigEntry {
                                        key: format!("{}/{}", resp.namespace, resp.key),
                                        value: value_str,
                                        updated_at: timestamp_to_rfc3339(resp.changed_at),
                                    };
                                    return Some((Ok(entry), stream));
                                }
                                Err(e) => {
                                    // UTF-8 デコード失敗時は警告を出してこのエントリをスキップする
                                    warn!(namespace = %resp.namespace, key = %resp.key, "watch_config: value is not valid UTF-8, skipping: {}", e);
                                    // ループ継続して次メッセージを処理する
                                }
                            }
                        }
                        // ストリームが正常終了した場合は購読を終了する
                        Ok(None) => return None,
                        // ストリームエラーはサブスクライバーに伝播してから購読を終了する
                        Err(status) => {
                            tracing::error!(
                                error = %status,
                                "watch_config: gRPC stream error, terminating subscription"
                            );
                            return Some((Err(status), stream));
                        }
                    }
                }
            },
        ))
    }
}

fn timestamp_to_rfc3339(ts: Option<proto::k1s0::system::common::v1::Timestamp>) -> String {
    ts.and_then(|ts| DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32))
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}
