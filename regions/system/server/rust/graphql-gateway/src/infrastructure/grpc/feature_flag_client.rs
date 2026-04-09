use std::collections::HashSet;
use std::time::Duration;

use async_graphql::futures_util::Stream;
use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::model::{FeatureFlag, FlagRule, FlagVariant};
use crate::domain::port::FeatureFlagPort;
use crate::infrastructure::config::BackendConfig;

// HIGH-001 監査対応: tonic::include_proto!で展開される生成コードのClippy警告を抑制する
#[allow(
    dead_code,
    clippy::default_trait_access,
    clippy::trivially_copy_pass_by_ref,
    clippy::too_many_lines,
    clippy::doc_markdown,
    clippy::must_use_candidate
)]
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
    /// バックエンドサービスのアドレス。gRPC Health Check Protocol のためのチャネル生成に使用する。
    address: String,
    /// `タイムアウト設定（ミリ秒）。health_check` のチャネル生成にも適用する。
    timeout_ms: u64,
}

impl FeatureFlagGrpcClient {
    /// proto Timestamp を RFC3339 形式文字列に変換する。
    /// proto Timestamp が None の場合はエポック時刻文字列を返す（フィールド必須のため）。
    fn timestamp_to_string(ts: Option<proto::k1s0::system::common::v1::Timestamp>) -> String {
        match ts {
            Some(t) => {
                // proto Timestamp は seconds と nanos で構成される
                // RFC3339 形式に変換するため chrono を使用する
                use std::time::{Duration as StdDuration, UNIX_EPOCH};
                // LOW-008: 安全な型変換（オーバーフロー防止）
                let secs = u64::try_from(t.seconds).unwrap_or(0);
                let nanos = u32::try_from(t.nanos).unwrap_or(0);
                let system_time = UNIX_EPOCH + StdDuration::new(secs, nanos);
                // chrono の DateTime を使って RFC3339 に変換する
                let datetime = chrono::DateTime::<chrono::Utc>::from(system_time);
                datetime.to_rfc3339()
            }
            // Timestamp が None の場合はエポック時刻を返す
            None => "1970-01-01T00:00:00Z".to_string(),
        }
    }

    /// proto Operator i32 値を文字列表現に変換する。
    /// proto enum Operator { `OPERATOR_UNSPECIFIED` = 0; `OPERATOR_EQ` = 1; `OPERATOR_NE` = 2;
    ///   `OPERATOR_CONTAINS` = 3; `OPERATOR_GT` = 4; `OPERATOR_LT` = 5; }
    fn operator_to_string(op: i32) -> String {
        match op {
            1 => "EQ".to_string(),
            2 => "NE".to_string(),
            3 => "CONTAINS".to_string(),
            4 => "GT".to_string(),
            5 => "LT".to_string(),
            _ => "UNSPECIFIED".to_string(),
        }
    }

    /// proto `ProtoFeatureFlag` をドメインモデル `FeatureFlag` に変換する。
    /// CRIT-007 対応: proto に存在しない `name/rollout_percentage/target_environments` を除去し、
    /// `variants/rules/created_at/updated_at` を直接マッピングする。
    fn to_domain_flag(flag: ProtoFeatureFlag) -> FeatureFlag {
        // proto FlagVariant をドメイン FlagVariant に変換する
        let variants = flag
            .variants
            .into_iter()
            .map(|v| FlagVariant {
                name: v.name,
                value: v.value,
                weight: v.weight,
            })
            .collect();

        // proto FlagRule をドメイン FlagRule に変換する（operator は i32 → 文字列変換）
        let rules = flag
            .rules
            .into_iter()
            .map(|r| FlagRule {
                attribute: r.attribute,
                operator: Self::operator_to_string(r.operator),
                value: r.value,
                variant: r.variant,
            })
            .collect();

        FeatureFlag {
            id: flag.id,
            flag_key: flag.flag_key,
            description: if flag.description.is_empty() {
                None
            } else {
                Some(flag.description)
            },
            enabled: flag.enabled,
            variants,
            rules,
            created_at: Self::timestamp_to_string(flag.created_at),
            updated_at: Self::timestamp_to_string(flag.updated_at),
        }
    }

    /// バックエンド設定からクライアントを生成する。
    /// `connect_lazy()` により起動時の接続確立を不要とし、実際のRPC呼び出し時に接続する。
    pub fn new(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect_lazy();
        Ok(Self {
            client: FeatureFlagServiceClient::new(channel),
            address: cfg.address.clone(),
            timeout_ms: cfg.timeout_ms,
        })
    }

    /// gRPC Health Check Protocol を使ってサービスの疎通確認を行う。
    /// Bearer token なしで接続できるため readyz ヘルスチェックに適している。
    /// tonic-health サービスが登録されているサーバーに対して Check RPC を送信する。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn health_check(&self) -> anyhow::Result<()> {
        let channel = Channel::from_shared(self.address.clone())?
            .timeout(Duration::from_millis(self.timeout_ms))
            .connect_lazy();
        let mut health_client = tonic_health::pb::health_client::HealthClient::new(channel);
        let request = tonic::Request::new(tonic_health::pb::HealthCheckRequest {
            service: "k1s0.system.featureflag.v1.FeatureFlagService".to_string(),
        });
        health_client
            .check(request)
            .await
            .map_err(|e| anyhow::anyhow!("featureflag gRPC Health Check 失敗: {e}"))?;
        Ok(())
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_flag(&self, key: &str) -> anyhow::Result<Option<FeatureFlag>> {
        let request = tonic::Request::new(proto::k1s0::system::featureflag::v1::GetFlagRequest {
            flag_key: key.to_owned(),
        });

        match self.client.clone().get_flag(request).await {
            Ok(resp) => {
                // let-else: Noneの場合は早期リターン
                let Some(flag) = resp.into_inner().flag else { return Ok(None) };
                Ok(Some(Self::to_domain_flag(flag)))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!("FeatureFlagService.GetFlag failed: {e}")),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_flags(&self, environment: Option<&str>) -> anyhow::Result<Vec<FeatureFlag>> {
        let resp = self
            .client
            .clone()
            .list_flags(tonic::Request::new(
                // デフォルト値を使用して全フラグを取得する。
                // page_size: 0 はサーバーデフォルト値を使用することを示す。
                proto::k1s0::system::featureflag::v1::ListFlagsRequest::default(),
            ))
            .await
            .map_err(|e| anyhow::anyhow!("FeatureFlagService.ListFlags failed: {e}"))?
            .into_inner();

        let mut flags: Vec<FeatureFlag> =
            resp.flags.into_iter().map(Self::to_domain_flag).collect();

        // environment フィルタ: rules の attribute="environment" にマッチするフラグのみ返す
        if let Some(env) = environment {
            flags.retain(|f| {
                // rules が空の場合は全環境対象とみなす
                f.rules.is_empty()
                    || f.rules
                        .iter()
                        .any(|r| r.attribute == "environment" && r.value == env)
            });
        }

        Ok(flags)
    }

    /// `DataLoader` 向け: 複数キーをまとめて取得
    pub async fn list_flags_by_keys(&self, keys: &[String]) -> anyhow::Result<Vec<FeatureFlag>> {
        if keys.is_empty() {
            return Ok(vec![]);
        }

        let key_set: HashSet<&str> = keys.iter().map(String::as_str).collect();
        let all_flags = self.list_flags(None).await?;
        Ok(all_flags
            .into_iter()
            .filter(|f| key_set.contains(f.flag_key.as_str()))
            .collect())
    }

    /// フラグを更新する。
    /// CRIT-007 対応: `rollout_percentage/target_environments` の抽象化を廃止し、
    /// proto と整合する variants/rules を直接受け付ける。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn set_flag(
        &self,
        key: &str,
        enabled: bool,
        variants: Vec<proto::k1s0::system::featureflag::v1::FlagVariant>,
        rules: Vec<proto::k1s0::system::featureflag::v1::FlagRule>,
    ) -> anyhow::Result<FeatureFlag> {
        let request =
            tonic::Request::new(proto::k1s0::system::featureflag::v1::UpdateFlagRequest {
                flag_key: key.to_owned(),
                enabled: Some(enabled),
                description: Some(String::new()),
                rules,
                variants,
            });

        let flag = self
            .client
            .clone()
            .update_flag(request)
            .await
            .map_err(|e| anyhow::anyhow!("FeatureFlagService.UpdateFlag failed: {e}"))?
            .into_inner()
            .flag
            .ok_or_else(|| anyhow::anyhow!("empty flag in response"))?;

        Ok(Self::to_domain_flag(flag))
    }

    /// `WatchFeatureFlag` Server-Side Streaming を購読し、変更イベントを `FeatureFlag` として返す。
    /// .`expect()` によるパニックを排除し、接続失敗時は `anyhow::Error` として伝播する。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn watch_feature_flag(
        &self,
        key: &str,
    ) -> anyhow::Result<impl Stream<Item = FeatureFlag>> {
        let request = tonic::Request::new(
            proto::k1s0::system::featureflag::v1::WatchFeatureFlagRequest {
                flag_key: key.to_owned(),
            },
        );

        // gRPC ストリーム接続を確立し、失敗時はエラーを返す（パニックしない）
        let stream = self
            .client
            .clone()
            .watch_feature_flag(request)
            .await?
            .into_inner();

        // WatchFeatureFlag ストリームの各レスポンスを FeatureFlag ドメインモデルに変換する。
        // flag フィールドが None の場合（バックエンドの不整合）はイベントをスキップして次へ進む。
        // 空のデフォルト値で構築するとサイレントなデータ欠損を招くため、None はスキップが正しい。
        Ok(async_graphql::futures_util::stream::unfold(
            stream,
            |mut stream| async move {
                loop {
                    match stream.message().await {
                        Ok(Some(resp)) => {
                            // flag フィールドが存在する場合のみドメインモデルに変換して返す
                            if let Some(f) = resp.flag {
                                return Some((Self::to_domain_flag(f), stream));
                            }
                            // flag フィールドが None の場合はスキップして次のメッセージを待つ
                            tracing::warn!(
                                flag_key = %resp.flag_key,
                                "WatchFeatureFlag: received event with no flag payload, skipping"
                            );
                            // loop を継続して次のメッセージを取得する
                        }
                        _ => return None,
                    }
                }
            },
        ))
    }
}

// FeatureFlagPort トレイトの実装。ドメイン層が具象クライアント型に依存せず、
// ポートトレイト経由でフィーチャーフラグサービスにアクセスできるようにする。
#[async_trait::async_trait]
impl FeatureFlagPort for FeatureFlagGrpcClient {
    async fn list_flags_by_keys(&self, keys: &[String]) -> anyhow::Result<Vec<FeatureFlag>> {
        self.list_flags_by_keys(keys).await
    }
}
