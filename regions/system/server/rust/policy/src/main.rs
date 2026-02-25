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

use adapter::grpc::PolicyGrpcService;
use domain::entity::policy::Policy;
use domain::entity::policy_bundle::PolicyBundle;
use domain::repository::PolicyRepository;
use domain::repository::PolicyBundleRepository;
use infrastructure::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-policy-server".to_string(),
        version: "0.1.0".to_string(),
        tier: "system".to_string(),
        environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string()),
        trace_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
        sample_rate: 1.0,
        log_level: "info".to_string(),
    };
    k1s0_telemetry::init_telemetry(&telemetry_cfg).expect("failed to init telemetry");

    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting policy server"
    );

    let policy_repo: Arc<dyn PolicyRepository> = Arc::new(InMemoryPolicyRepository::new());
    let bundle_repo: Arc<dyn PolicyBundleRepository> = Arc::new(InMemoryPolicyBundleRepository::new());

    let create_policy_uc = Arc::new(usecase::CreatePolicyUseCase::new(policy_repo.clone()));
    let get_policy_uc = Arc::new(usecase::GetPolicyUseCase::new(policy_repo.clone()));
    let update_policy_uc = Arc::new(usecase::UpdatePolicyUseCase::new(policy_repo.clone()));
    let evaluate_policy_uc = Arc::new(usecase::EvaluatePolicyUseCase::new(policy_repo.clone()));
    let _create_bundle_uc = Arc::new(usecase::CreateBundleUseCase::new(bundle_repo));

    let grpc_svc = Arc::new(PolicyGrpcService::new(
        evaluate_policy_uc.clone(),
        get_policy_uc.clone(),
    ));

    let state = adapter::handler::AppState {
        policy_repo,
        create_policy_uc,
        get_policy_uc,
        update_policy_uc,
        evaluate_policy_uc,
    };

    let app = adapter::handler::router(state);

    // gRPC server
    use proto::k1s0::system::policy::v1::policy_service_server::PolicyServiceServer;

    let policy_tonic = adapter::grpc::PolicyServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], 50051).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_future = async move {
        tonic::transport::Server::builder()
            .add_service(PolicyServiceServer::new(policy_tonic))
            .serve(grpc_addr)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    // REST server
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
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

// --- InMemoryPolicyRepository ---

struct InMemoryPolicyRepository {
    policies: tokio::sync::RwLock<HashMap<Uuid, Policy>>,
}

impl InMemoryPolicyRepository {
    fn new() -> Self {
        Self {
            policies: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl PolicyRepository for InMemoryPolicyRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Policy>> {
        let policies = self.policies.read().await;
        Ok(policies.get(id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<Policy>> {
        let policies = self.policies.read().await;
        Ok(policies.values().cloned().collect())
    }

    async fn create(&self, policy: &Policy) -> anyhow::Result<()> {
        let mut policies = self.policies.write().await;
        policies.insert(policy.id, policy.clone());
        Ok(())
    }

    async fn update(&self, policy: &Policy) -> anyhow::Result<()> {
        let mut policies = self.policies.write().await;
        policies.insert(policy.id, policy.clone());
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut policies = self.policies.write().await;
        Ok(policies.remove(id).is_some())
    }

    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool> {
        let policies = self.policies.read().await;
        Ok(policies.values().any(|p| p.name == name))
    }
}

// --- InMemoryPolicyBundleRepository ---

struct InMemoryPolicyBundleRepository {
    bundles: tokio::sync::RwLock<HashMap<Uuid, PolicyBundle>>,
}

impl InMemoryPolicyBundleRepository {
    fn new() -> Self {
        Self {
            bundles: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl PolicyBundleRepository for InMemoryPolicyBundleRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<PolicyBundle>> {
        let bundles = self.bundles.read().await;
        Ok(bundles.get(id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<PolicyBundle>> {
        let bundles = self.bundles.read().await;
        Ok(bundles.values().cloned().collect())
    }

    async fn create(&self, bundle: &PolicyBundle) -> anyhow::Result<()> {
        let mut bundles = self.bundles.write().await;
        bundles.insert(bundle.id, bundle.clone());
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut bundles = self.bundles.write().await;
        Ok(bundles.remove(id).is_some())
    }
}
