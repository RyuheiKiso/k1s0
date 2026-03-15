use std::collections::HashSet;
use std::time::Duration;

use async_graphql::futures_util::Stream;
use chrono::{DateTime, Utc};
use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::model::{Tenant, TenantStatus};
use crate::infrastructure::config::BackendConfig;

/// gRPC レスポンスの中間表現。GraphQL の TenantConnection に変換前の生データ。
pub struct TenantPage {
    pub nodes: Vec<Tenant>,
    pub total_count: i32,
    pub has_next: bool,
}

#[allow(dead_code)]
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
}

impl TenantGrpcClient {
    fn tenant_from_proto(t: ProtoTenant) -> Tenant {
        Tenant {
            id: t.id,
            name: t.name,
            status: TenantStatus::from(t.status),
            created_at: timestamp_to_rfc3339(t.created_at),
            updated_at: timestamp_to_rfc3339(t.updated_at),
        }
    }

    pub async fn connect(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect()
            .await?;
        Ok(Self {
            client: TenantServiceClient::new(channel),
        })
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_tenant(&self, tenant_id: &str) -> anyhow::Result<Option<Tenant>> {
        let request = tonic::Request::new(proto::k1s0::system::tenant::v1::GetTenantRequest {
            tenant_id: tenant_id.to_owned(),
        });

        match self.client.clone().get_tenant(request).await {
            Ok(resp) => {
                let t = match resp.into_inner().tenant {
                    Some(t) => t,
                    None => return Ok(None),
                };
                Ok(Some(Self::tenant_from_proto(t)))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!("TenantService.GetTenant failed: {}", e)),
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
            .map_err(|e| anyhow::anyhow!("TenantService.ListTenants failed: {}", e))?
            .into_inner();

        let nodes = resp
            .tenants
            .into_iter()
            .map(Self::tenant_from_proto)
            .collect();

        let (total_count, has_next) = resp
            .pagination
            .map(|p| (p.total_count, p.has_next))
            .unwrap_or((0, false));

        Ok(TenantPage {
            nodes,
            total_count,
            has_next,
        })
    }

    /// DataLoader 向け: 複数 ID をまとめて取得（ListTenants + クライアント側フィルタ）
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

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn create_tenant(&self, name: &str, owner_user_id: &str) -> anyhow::Result<Tenant> {
        let request = tonic::Request::new(proto::k1s0::system::tenant::v1::CreateTenantRequest {
            name: name.to_owned(),
            display_name: name.to_owned(),
            owner_id: owner_user_id.to_owned(),
            plan: "standard".to_owned(),
        });

        let t = self
            .client
            .clone()
            .create_tenant(request)
            .await
            .map_err(|e| anyhow::anyhow!("TenantService.CreateTenant failed: {}", e))?
            .into_inner()
            .tenant
            .ok_or_else(|| anyhow::anyhow!("empty tenant in response"))?;

        Ok(Self::tenant_from_proto(t))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_tenant(
        &self,
        id: &str,
        name: Option<&str>,
        status: Option<&str>,
    ) -> anyhow::Result<Tenant> {
        let current = self
            .client
            .clone()
            .get_tenant(tonic::Request::new(
                proto::k1s0::system::tenant::v1::GetTenantRequest {
                    tenant_id: id.to_owned(),
                },
            ))
            .await
            .map_err(|e| anyhow::anyhow!("TenantService.GetTenant failed: {}", e))?
            .into_inner()
            .tenant
            .ok_or_else(|| anyhow::anyhow!("tenant not found: {}", id))?;

        let mut latest = current;

        if let Some(display_name) = name {
            let updated = self
                .client
                .clone()
                .update_tenant(tonic::Request::new(
                    proto::k1s0::system::tenant::v1::UpdateTenantRequest {
                        tenant_id: id.to_owned(),
                        display_name: display_name.to_owned(),
                        plan: latest.plan.clone(),
                    },
                ))
                .await
                .map_err(|e| anyhow::anyhow!("TenantService.UpdateTenant failed: {}", e))?
                .into_inner()
                .tenant
                .ok_or_else(|| anyhow::anyhow!("empty tenant in update response"))?;
            latest = updated;
        }

        if let Some(status) = status {
            let status = status.to_ascii_uppercase();
            let transitioned = match status.as_str() {
                "ACTIVE" => Some(
                    self.client
                        .clone()
                        .activate_tenant(tonic::Request::new(
                            proto::k1s0::system::tenant::v1::ActivateTenantRequest {
                                tenant_id: id.to_owned(),
                            },
                        ))
                        .await
                        .map_err(|e| anyhow::anyhow!("TenantService.ActivateTenant failed: {}", e))?
                        .into_inner()
                        .tenant
                        .ok_or_else(|| anyhow::anyhow!("empty tenant in activate response"))?,
                ),
                "SUSPENDED" => Some(
                    self.client
                        .clone()
                        .suspend_tenant(tonic::Request::new(
                            proto::k1s0::system::tenant::v1::SuspendTenantRequest {
                                tenant_id: id.to_owned(),
                            },
                        ))
                        .await
                        .map_err(|e| anyhow::anyhow!("TenantService.SuspendTenant failed: {}", e))?
                        .into_inner()
                        .tenant
                        .ok_or_else(|| anyhow::anyhow!("empty tenant in suspend response"))?,
                ),
                "DELETED" => Some(
                    self.client
                        .clone()
                        .delete_tenant(tonic::Request::new(
                            proto::k1s0::system::tenant::v1::DeleteTenantRequest {
                                tenant_id: id.to_owned(),
                            },
                        ))
                        .await
                        .map_err(|e| anyhow::anyhow!("TenantService.DeleteTenant failed: {}", e))?
                        .into_inner()
                        .tenant
                        .ok_or_else(|| anyhow::anyhow!("empty tenant in delete response"))?,
                ),
                _ => None,
            };
            if let Some(next) = transitioned {
                latest = next;
            }
        }

        Ok(Self::tenant_from_proto(latest))
    }

    /// WatchTenant Server-Side Streaming を購読し、変更イベントを Tenant として返す。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn watch_tenant(&self, tenant_id: &str) -> impl Stream<Item = Tenant> {
        let request = tonic::Request::new(proto::k1s0::system::tenant::v1::WatchTenantRequest {
            tenant_id: tenant_id.to_owned(),
        });

        let stream = self
            .client
            .clone()
            .watch_tenant(request)
            .await
            .expect("WatchTenant stream failed")
            .into_inner();

        async_graphql::futures_util::stream::unfold(stream, |mut stream| async move {
            match stream.message().await {
                Ok(Some(resp)) => {
                    let tenant = resp.tenant.map(Self::tenant_from_proto).unwrap_or(Tenant {
                        id: resp.tenant_id,
                        name: String::new(),
                        status: TenantStatus::from(String::new()),
                        created_at: String::new(),
                        updated_at: String::new(),
                    });
                    Some((tenant, stream))
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
