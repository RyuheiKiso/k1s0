use std::collections::HashSet;
use std::time::Duration;

use async_graphql::futures_util::Stream;
use chrono::{DateTime, Utc};
use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::model::{Tenant, TenantStatus};
use crate::domain::port::TenantPort;
use crate::infrastructure::config::BackendConfig;

/// gRPC レスポンスの中間表現。GraphQL の `TenantConnection` に変換前の生データ。
pub struct TenantPage {
    pub nodes: Vec<Tenant>,
    pub total_count: i64,
    pub has_next: bool,
}

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
            pub mod tenant {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.tenant.v1");
                }
            }
        }
    }
}

use proto::k1s0::system::tenant::v1::tenant_service_client::TenantServiceClient;
use proto::k1s0::system::tenant::v1::Tenant as ProtoTenant;

pub struct TenantGrpcClient {
    client: TenantServiceClient<Channel>,
    /// バックエンドサービスのアドレス。gRPC Health Check Protocol のためのチャネル生成に使用する。
    address: String,
    /// `タイムアウト設定（ミリ秒）。health_check` のチャネル生成にも適用する。
    timeout_ms: u64,
}

impl TenantGrpcClient {
    /// proto の Tenant メッセージをドメインモデルの Tenant に変換する。
    /// C-002 監査対応: proto `の全フィールド（display_name`, plan, `owner_id`, settings, `db_schema`, `keycloak_realm）をマッピングする`。
    /// 空文字列は Option フィールドに対して None に変換することで、GraphQL スキーマの nullable 型と整合させる。
    fn tenant_from_proto(t: ProtoTenant) -> Tenant {
        Tenant {
            id: t.id,
            name: t.name,
            display_name: t.display_name,
            status: TenantStatus::from(t.status),
            plan: t.plan,
            owner_id: t.owner_id,
            // 空文字列の場合は None に変換する（proto の optional フィールドと GraphQL nullable の整合）
            settings: if t.settings.is_empty() {
                None
            } else {
                Some(t.settings)
            },
            db_schema: if t.db_schema.is_empty() {
                None
            } else {
                Some(t.db_schema)
            },
            keycloak_realm: if t.keycloak_realm.is_empty() {
                None
            } else {
                Some(t.keycloak_realm)
            },
            created_at: timestamp_to_rfc3339(t.created_at),
            updated_at: timestamp_to_rfc3339(t.updated_at),
        }
    }

    /// バックエンド設定からクライアントを生成する。
    /// `connect_lazy()` により起動時の接続確立を不要とし、実際のRPC呼び出し時に接続する。
    pub fn new(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect_lazy();
        Ok(Self {
            client: TenantServiceClient::new(channel),
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
            service: "k1s0.system.tenant.v1.TenantService".to_string(),
        });
        health_client
            .check(request)
            .await
            .map_err(|e| anyhow::anyhow!("tenant gRPC Health Check 失敗: {e}"))?;
        Ok(())
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_tenant(&self, tenant_id: &str) -> anyhow::Result<Option<Tenant>> {
        let request = tonic::Request::new(proto::k1s0::system::tenant::v1::GetTenantRequest {
            tenant_id: tenant_id.to_owned(),
        });

        match self.client.clone().get_tenant(request).await {
            Ok(resp) => {
                // let-else: Noneの場合は早期リターン
                let Some(t) = resp.into_inner().tenant else { return Ok(None) };
                Ok(Some(Self::tenant_from_proto(t)))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!("TenantService.GetTenant failed: {e}")),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_tenants(&self, page: i32, page_size: i32) -> anyhow::Result<TenantPage> {
        let request = tonic::Request::new(proto::k1s0::system::tenant::v1::ListTenantsRequest {
            pagination: Some(proto::k1s0::system::common::v1::Pagination { page, page_size }),
        });

        let resp = self
            .client
            .clone()
            .list_tenants(request)
            .await
            .map_err(|e| anyhow::anyhow!("TenantService.ListTenants failed: {e}"))?
            .into_inner();

        let nodes = resp
            .tenants
            .into_iter()
            .map(Self::tenant_from_proto)
            .collect();

        let (total_count, has_next) = resp
            .pagination
            .map_or((0, false), |p| (p.total_count, p.has_next));

        Ok(TenantPage {
            nodes,
            total_count,
            has_next,
        })
    }

    /// `DataLoader` 向け: 複数 ID をまとめて取得（ListTenants + クライアント側フィルタ）
    pub async fn list_tenants_by_ids(&self, ids: &[String]) -> anyhow::Result<Vec<Tenant>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let id_set: HashSet<&str> = ids.iter().map(String::as_str).collect();
        let mut found = Vec::new();
        let mut page = 1;
        let page_size = 100;

        loop {
            let page_resp = self.list_tenants(page, page_size).await?;
            for tenant in page_resp.nodes {
                if id_set.contains(tenant.id.as_str()) {
                    found.push(tenant);
                }
            }
            if !page_resp.has_next || found.len() >= id_set.len() {
                break;
            }
            page += 1;
        }

        Ok(found)
    }

    /// テナント作成 gRPC リクエストを送信する。
    /// C-003 監査対応: `display_name`, `owner_id`, plan を個別引数として受け取り、proto の `CreateTenantRequest` にマッピングする。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn create_tenant(
        &self,
        name: &str,
        display_name: &str,
        owner_id: &str,
        plan: &str,
    ) -> anyhow::Result<Tenant> {
        let request = tonic::Request::new(proto::k1s0::system::tenant::v1::CreateTenantRequest {
            name: name.to_owned(),
            display_name: display_name.to_owned(),
            owner_id: owner_id.to_owned(),
            plan: plan.to_owned(),
        });

        let t = self
            .client
            .clone()
            .create_tenant(request)
            .await
            .map_err(|e| anyhow::anyhow!("TenantService.CreateTenant failed: {e}"))?
            .into_inner()
            .tenant
            .ok_or_else(|| anyhow::anyhow!("empty tenant in response"))?;

        Ok(Self::tenant_from_proto(t))
    }

    /// CRIT-007 対応: `display_name` と plan を proto `UpdateTenantRequest` に直接渡す
    /// status 変更は `suspend_tenant/activate_tenant/delete_tenant` で対応する
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_tenant(
        &self,
        id: &str,
        display_name: Option<&str>,
        plan: Option<&str>,
    ) -> anyhow::Result<Tenant> {
        // 少なくとも一方が指定されている場合のみ UpdateTenant RPC を呼び出す
        if display_name.is_none() && plan.is_none() {
            // 変更なし: 現在のテナント情報をそのまま返す
            let current = self
                .client
                .clone()
                .get_tenant(tonic::Request::new(
                    proto::k1s0::system::tenant::v1::GetTenantRequest {
                        tenant_id: id.to_owned(),
                    },
                ))
                .await
                .map_err(|e| anyhow::anyhow!("TenantService.GetTenant failed: {e}"))?
                .into_inner()
                .tenant
                .ok_or_else(|| anyhow::anyhow!("tenant not found: {id}"))?;
            return Ok(Self::tenant_from_proto(current));
        }

        // display_name または plan の片方だけ指定された場合は現在値を読んでから更新する
        let current = self
            .client
            .clone()
            .get_tenant(tonic::Request::new(
                proto::k1s0::system::tenant::v1::GetTenantRequest {
                    tenant_id: id.to_owned(),
                },
            ))
            .await
            .map_err(|e| anyhow::anyhow!("TenantService.GetTenant failed: {e}"))?
            .into_inner()
            .tenant
            .ok_or_else(|| anyhow::anyhow!("tenant not found: {id}"))?;

        let updated = self
            .client
            .clone()
            .update_tenant(tonic::Request::new(
                proto::k1s0::system::tenant::v1::UpdateTenantRequest {
                    tenant_id: id.to_owned(),
                    // 指定されていなければ現在値を維持する
                    display_name: display_name.unwrap_or(&current.display_name).to_owned(),
                    plan: plan.unwrap_or(&current.plan).to_owned(),
                },
            ))
            .await
            .map_err(|e| anyhow::anyhow!("TenantService.UpdateTenant failed: {e}"))?
            .into_inner()
            .tenant
            .ok_or_else(|| anyhow::anyhow!("empty tenant in update response"))?;

        Ok(Self::tenant_from_proto(updated))
    }

    /// `WatchTenant` Server-Side Streaming を購読し、変更イベントを Tenant として返す。
    /// .`expect()` によるパニックを排除し、接続失敗時は `anyhow::Error` として伝播する。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn watch_tenant(
        &self,
        tenant_id: &str,
    ) -> anyhow::Result<impl Stream<Item = Tenant>> {
        let request = tonic::Request::new(proto::k1s0::system::tenant::v1::WatchTenantRequest {
            tenant_id: tenant_id.to_owned(),
        });

        // gRPC ストリーム接続を確立し、失敗時はエラーを返す（パニックしない）
        let stream = self
            .client
            .clone()
            .watch_tenant(request)
            .await?
            .into_inner();

        // WatchTenant ストリームの各レスポンスを Tenant ドメインモデルに変換する。
        // tenant フィールドが None の場合（バックエンドの不整合）はイベントをスキップして次へ進む。
        // 空のデフォルト値で構築するとサイレントなデータ欠損を招くため、None はスキップが正しい。
        Ok(async_graphql::futures_util::stream::unfold(
            stream,
            |mut stream| async move {
                loop {
                    match stream.message().await {
                        Ok(Some(resp)) => {
                            // tenant フィールドが存在する場合のみドメインモデルに変換して返す
                            if let Some(t) = resp.tenant {
                                return Some((Self::tenant_from_proto(t), stream));
                            }
                            // tenant フィールドが None の場合はスキップして次のメッセージを待つ
                            tracing::warn!(
                                tenant_id = %resp.tenant_id,
                                "WatchTenant: received event with no tenant payload, skipping"
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

// TenantPort トレイトの実装。ドメイン層が具象クライアント型に依存せず、
// ポートトレイト経由でテナントサービスにアクセスできるようにする。
#[async_trait::async_trait]
impl TenantPort for TenantGrpcClient {
    async fn list_tenants_by_ids(&self, ids: &[String]) -> anyhow::Result<Vec<Tenant>> {
        self.list_tenants_by_ids(ids).await
    }
}

fn timestamp_to_rfc3339(ts: Option<proto::k1s0::system::common::v1::Timestamp>) -> String {
    // LOW-008: 安全な型変換（オーバーフロー防止）
    ts.and_then(|ts| DateTime::<Utc>::from_timestamp(ts.seconds, u32::try_from(ts.nanos).unwrap_or(0)))
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}
