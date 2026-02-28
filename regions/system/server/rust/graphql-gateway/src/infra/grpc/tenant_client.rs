use std::time::Duration;

use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::model::{Tenant, TenantStatus};
use crate::infra::config::BackendConfig;

/// gRPC レスポンスの中間表現。GraphQL の TenantConnection に変換前の生データ。
pub struct TenantPage {
    pub nodes: Vec<Tenant>,
    pub total_count: i32,
    pub has_next: bool,
}

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

pub struct TenantGrpcClient {
    client: TenantServiceClient<Channel>,
}

impl TenantGrpcClient {
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
                Ok(Some(Tenant {
                    id: t.id,
                    name: t.name,
                    status: TenantStatus::from(t.status),
                    created_at: t
                        .created_at
                        .map(|ts| ts.seconds.to_string())
                        .unwrap_or_default(),
                    updated_at: String::new(),
                }))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!("TenantService.GetTenant failed: {}", e)),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_tenants(
        &self,
        page: i32,
        page_size: i32,
    ) -> anyhow::Result<TenantPage> {
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
            .map(|t| Tenant {
                id: t.id,
                name: t.name,
                status: TenantStatus::from(t.status),
                created_at: t
                    .created_at
                    .map(|ts| ts.seconds.to_string())
                    .unwrap_or_default(),
                updated_at: String::new(),
            })
            .collect();

        let (total_count, has_next) = resp
            .pagination
            .map(|p| (p.total_count as i32, p.has_next))
            .unwrap_or((0, false));

        Ok(TenantPage {
            nodes,
            total_count,
            has_next,
        })
    }

    /// DataLoader 向け: 複数 ID をまとめて取得（ListTenants + クライアント側フィルタ）
    pub async fn list_tenants_by_ids(&self, _ids: &[String]) -> anyhow::Result<Vec<Tenant>> {
        Ok(vec![])
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn create_tenant(
        &self,
        name: &str,
        owner_user_id: &str,
    ) -> anyhow::Result<Tenant> {
        let request =
            tonic::Request::new(proto::k1s0::system::tenant::v1::CreateTenantRequest {
                name: name.to_owned(),
                display_name: name.to_owned(),
                owner_user_id: owner_user_id.to_owned(),
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

        Ok(Tenant {
            id: t.id,
            name: t.name,
            status: TenantStatus::from(t.status),
            created_at: t
                .created_at
                .map(|ts| ts.seconds.to_string())
                .unwrap_or_default(),
            updated_at: String::new(),
        })
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_tenant(
        &self,
        _id: &str,
        _name: Option<&str>,
        _status: Option<&str>,
    ) -> anyhow::Result<Tenant> {
        // TenantService に UpdateTenant RPC が追加された時点で実装
        anyhow::bail!("UpdateTenant not yet implemented in TenantService");
    }
}
