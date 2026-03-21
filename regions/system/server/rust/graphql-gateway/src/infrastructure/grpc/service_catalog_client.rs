use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::model::{
    CatalogService, CatalogServiceConnection, MetadataEntry, ServiceHealth,
};
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
            pub mod servicecatalog {
                pub mod v1 {
                    tonic::include_proto!("k1s0.system.servicecatalog.v1");
                }
            }
        }
    }
}

use proto::k1s0::system::servicecatalog::v1::service_catalog_service_client::ServiceCatalogServiceClient;
use proto::k1s0::system::servicecatalog::v1::{
    ServiceHealth as ProtoServiceHealth, ServiceInfo as ProtoServiceInfo,
};

pub struct ServiceCatalogGrpcClient {
    client: ServiceCatalogServiceClient<Channel>,
}

impl ServiceCatalogGrpcClient {
    /// バックエンド設定からクライアントを生成する。
    /// connect_lazy() により起動時の接続確立を不要とし、実際のRPC呼び出し時に接続する。
    pub fn new(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect_lazy();
        Ok(Self {
            client: ServiceCatalogServiceClient::new(channel),
        })
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_service(&self, service_id: &str) -> anyhow::Result<Option<CatalogService>> {
        let request =
            tonic::Request::new(proto::k1s0::system::servicecatalog::v1::GetServiceRequest {
                service_id: service_id.to_owned(),
            });

        match self.client.clone().get_service(request).await {
            Ok(resp) => {
                let svc = match resp.into_inner().service {
                    Some(s) => s,
                    None => return Ok(None),
                };
                Ok(Some(service_from_proto(svc)))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!(
                "ServiceCatalogService.GetService failed: {}",
                e
            )),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_services(
        &self,
        page: i32,
        page_size: i32,
        tier: Option<&str>,
        status: Option<&str>,
        search: Option<&str>,
    ) -> anyhow::Result<CatalogServiceConnection> {
        let request = tonic::Request::new(
            proto::k1s0::system::servicecatalog::v1::ListServicesRequest {
                pagination: Some(proto::k1s0::system::common::v1::Pagination { page, page_size }),
                tier: tier.map(|s| s.to_owned()),
                status: status.map(|s| s.to_owned()),
                search: search.map(|s| s.to_owned()),
            },
        );

        let resp = self
            .client
            .clone()
            .list_services(request)
            .await
            .map_err(|e| anyhow::anyhow!("ServiceCatalogService.ListServices failed: {}", e))?
            .into_inner();

        let services = resp.services.into_iter().map(service_from_proto).collect();

        let (total_count, has_next) = resp
            .pagination
            .map(|p| (p.total_count, p.has_next))
            .unwrap_or((0, false));

        Ok(CatalogServiceConnection {
            services,
            total_count,
            has_next,
        })
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn register_service(
        &self,
        name: &str,
        display_name: &str,
        description: &str,
        tier: &str,
        version: &str,
        base_url: &str,
        grpc_endpoint: Option<&str>,
        health_url: &str,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<CatalogService> {
        let request = tonic::Request::new(
            proto::k1s0::system::servicecatalog::v1::RegisterServiceRequest {
                name: name.to_owned(),
                display_name: display_name.to_owned(),
                description: description.to_owned(),
                tier: tier.to_owned(),
                version: version.to_owned(),
                base_url: base_url.to_owned(),
                grpc_endpoint: grpc_endpoint.map(|s| s.to_owned()),
                health_url: health_url.to_owned(),
                metadata,
            },
        );

        let svc = self
            .client
            .clone()
            .register_service(request)
            .await
            .map_err(|e| anyhow::anyhow!("ServiceCatalogService.RegisterService failed: {}", e))?
            .into_inner()
            .service
            .ok_or_else(|| anyhow::anyhow!("empty service in register response"))?;

        Ok(service_from_proto(svc))
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_service(
        &self,
        service_id: &str,
        display_name: Option<&str>,
        description: Option<&str>,
        version: Option<&str>,
        base_url: Option<&str>,
        grpc_endpoint: Option<&str>,
        health_url: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<CatalogService> {
        let request = tonic::Request::new(
            proto::k1s0::system::servicecatalog::v1::UpdateServiceRequest {
                service_id: service_id.to_owned(),
                display_name: display_name.map(|s| s.to_owned()),
                description: description.map(|s| s.to_owned()),
                version: version.map(|s| s.to_owned()),
                base_url: base_url.map(|s| s.to_owned()),
                grpc_endpoint: grpc_endpoint.map(|s| s.to_owned()),
                health_url: health_url.map(|s| s.to_owned()),
                metadata,
            },
        );

        let svc = self
            .client
            .clone()
            .update_service(request)
            .await
            .map_err(|e| anyhow::anyhow!("ServiceCatalogService.UpdateService failed: {}", e))?
            .into_inner()
            .service
            .ok_or_else(|| anyhow::anyhow!("empty service in update response"))?;

        Ok(service_from_proto(svc))
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn delete_service(&self, service_id: &str) -> anyhow::Result<bool> {
        let request = tonic::Request::new(
            proto::k1s0::system::servicecatalog::v1::DeleteServiceRequest {
                service_id: service_id.to_owned(),
            },
        );

        let resp = self
            .client
            .clone()
            .delete_service(request)
            .await
            .map_err(|e| anyhow::anyhow!("ServiceCatalogService.DeleteService failed: {}", e))?
            .into_inner();

        Ok(resp.success)
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn health_check(
        &self,
        service_id: Option<&str>,
    ) -> anyhow::Result<Vec<ServiceHealth>> {
        let request = tonic::Request::new(
            proto::k1s0::system::servicecatalog::v1::HealthCheckRequest {
                service_id: service_id.map(|s| s.to_owned()),
            },
        );

        let resp = self
            .client
            .clone()
            .health_check(request)
            .await
            .map_err(|e| anyhow::anyhow!("ServiceCatalogService.HealthCheck failed: {}", e))?
            .into_inner();

        Ok(resp.services.into_iter().map(health_from_proto).collect())
    }
}

fn service_from_proto(s: ProtoServiceInfo) -> CatalogService {
    CatalogService {
        id: s.id,
        name: s.name,
        display_name: s.display_name,
        description: s.description,
        tier: s.tier,
        version: s.version,
        base_url: s.base_url,
        grpc_endpoint: s.grpc_endpoint.filter(|v| !v.is_empty()),
        health_url: s.health_url,
        status: s.status,
        metadata: s
            .metadata
            .into_iter()
            .map(|(k, v)| MetadataEntry { key: k, value: v })
            .collect(),
        created_at: timestamp_to_rfc3339(s.created_at),
        updated_at: timestamp_to_rfc3339(s.updated_at),
    }
}

fn health_from_proto(h: ProtoServiceHealth) -> ServiceHealth {
    ServiceHealth {
        service_id: h.service_id,
        service_name: h.service_name,
        status: h.status,
        response_time_ms: h.response_time_ms,
        error_message: h.error_message.filter(|v| !v.is_empty()),
        checked_at: timestamp_to_rfc3339(h.checked_at),
    }
}

fn timestamp_to_rfc3339(ts: Option<proto::k1s0::system::common::v1::Timestamp>) -> String {
    ts.and_then(|ts| DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32))
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}
