#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;
use uuid::Uuid;

mod adapter;
mod domain;
mod infrastructure;
mod proto;
mod usecase;

use adapter::handler::{self, AppState};
use domain::entity::{ProvisioningJob, Tenant, TenantMember, TenantStatus};
use domain::repository::{MemberRepository, TenantRepository};

#[derive(Debug, Clone, serde::Deserialize)]
struct Config {
    app: AppConfig,
    server: ServerConfig,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AppConfig {
    name: String,
    #[serde(default = "default_version")]
    version: String,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ServerConfig {
    #[serde(default = "default_http_port")]
    http_port: u16,
    #[serde(default = "default_grpc_port")]
    grpc_port: u16,
}

fn default_http_port() -> u16 {
    8089
}

fn default_grpc_port() -> u16 {
    50058
}

// --- InMemory Repository ---

struct InMemoryTenantRepository {
    tenants: tokio::sync::RwLock<Vec<Tenant>>,
}

impl InMemoryTenantRepository {
    fn new() -> Self {
        Self {
            tenants: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl TenantRepository for InMemoryTenantRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Tenant>> {
        let tenants = self.tenants.read().await;
        Ok(tenants.iter().find(|t| t.id == *id).cloned())
    }

    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<Tenant>> {
        let tenants = self.tenants.read().await;
        Ok(tenants.iter().find(|t| t.name == name).cloned())
    }

    async fn list(&self, page: i32, page_size: i32) -> anyhow::Result<(Vec<Tenant>, i64)> {
        let tenants = self.tenants.read().await;
        let total = tenants.len() as i64;
        let offset = ((page - 1) * page_size) as usize;
        let result: Vec<_> = tenants.iter().skip(offset).take(page_size as usize).cloned().collect();
        Ok((result, total))
    }

    async fn create(&self, tenant: &Tenant) -> anyhow::Result<()> {
        let mut tenants = self.tenants.write().await;
        tenants.push(tenant.clone());
        Ok(())
    }

    async fn update(&self, tenant: &Tenant) -> anyhow::Result<()> {
        let mut tenants = self.tenants.write().await;
        if let Some(existing) = tenants.iter_mut().find(|t| t.id == tenant.id) {
            *existing = tenant.clone();
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let config_content = std::fs::read_to_string(&config_path)?;
    let cfg: Config = serde_yaml::from_str(&config_content)?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        http_port = cfg.server.http_port,
        grpc_port = cfg.server.grpc_port,
        "starting tenant server"
    );

    let tenant_repo: Arc<dyn TenantRepository> = Arc::new(InMemoryTenantRepository::new());
    let member_repo: Arc<dyn MemberRepository> = Arc::new(InMemoryMemberRepository::new());

    let create_tenant_uc = Arc::new(usecase::CreateTenantUseCase::new(tenant_repo.clone()));
    let get_tenant_uc = Arc::new(usecase::GetTenantUseCase::new(tenant_repo.clone()));
    let list_tenants_uc = Arc::new(usecase::ListTenantsUseCase::new(tenant_repo));
    let add_member_uc = Arc::new(usecase::AddMemberUseCase::new(member_repo.clone()));
    let remove_member_uc = Arc::new(usecase::RemoveMemberUseCase::new(member_repo.clone()));
    let get_provisioning_status_uc =
        Arc::new(usecase::GetProvisioningStatusUseCase::new(member_repo));

    // gRPC service
    use adapter::grpc::TenantGrpcService;
    use proto::k1s0::system::tenant::v1::tenant_service_server::TenantServiceServer;

    let tenant_grpc_svc = Arc::new(TenantGrpcService::new(
        create_tenant_uc.clone(),
        get_tenant_uc.clone(),
        list_tenants_uc.clone(),
        add_member_uc,
        remove_member_uc,
        get_provisioning_status_uc,
    ));
    let tenant_tonic = adapter::grpc::TenantServiceTonic::new(tenant_grpc_svc);

    let state = AppState {
        create_tenant_uc,
        get_tenant_uc,
        list_tenants_uc,
    };
    let app = handler::router(state);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_future = async move {
        tonic::transport::Server::builder()
            .add_service(TenantServiceServer::new(tenant_tonic))
            .serve(grpc_addr)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.http_port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    let rest_future = axum::serve(listener, app);

    tokio::select! {
        result = rest_future => {
            if let Err(e) = result {
                tracing::error!("REST server error: {}", e);
            }
        }
        result = grpc_future => {
            if let Err(e) = result {
                tracing::error!("gRPC server error: {}", e);
            }
        }
    }

    Ok(())
}

// --- InMemory MemberRepository ---

struct InMemoryMemberRepository {
    members: tokio::sync::RwLock<Vec<TenantMember>>,
    jobs: tokio::sync::RwLock<Vec<ProvisioningJob>>,
}

impl InMemoryMemberRepository {
    fn new() -> Self {
        Self {
            members: tokio::sync::RwLock::new(Vec::new()),
            jobs: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl MemberRepository for InMemoryMemberRepository {
    async fn find_by_tenant(&self, tenant_id: &Uuid) -> anyhow::Result<Vec<TenantMember>> {
        let members = self.members.read().await;
        Ok(members
            .iter()
            .filter(|m| m.tenant_id == *tenant_id)
            .cloned()
            .collect())
    }

    async fn find_member(
        &self,
        tenant_id: &Uuid,
        user_id: &Uuid,
    ) -> anyhow::Result<Option<TenantMember>> {
        let members = self.members.read().await;
        Ok(members
            .iter()
            .find(|m| m.tenant_id == *tenant_id && m.user_id == *user_id)
            .cloned())
    }

    async fn add(&self, member: &TenantMember) -> anyhow::Result<()> {
        let mut members = self.members.write().await;
        members.push(member.clone());
        Ok(())
    }

    async fn remove(&self, tenant_id: &Uuid, user_id: &Uuid) -> anyhow::Result<bool> {
        let mut members = self.members.write().await;
        let len_before = members.len();
        members.retain(|m| !(m.tenant_id == *tenant_id && m.user_id == *user_id));
        Ok(members.len() < len_before)
    }

    async fn find_job(&self, job_id: &Uuid) -> anyhow::Result<Option<ProvisioningJob>> {
        let jobs = self.jobs.read().await;
        Ok(jobs.iter().find(|j| j.id == *job_id).cloned())
    }
}
