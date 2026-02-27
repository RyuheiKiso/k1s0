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
use adapter::repository::bundle_postgres::BundlePostgresRepository;
use adapter::repository::cached_policy_repository::CachedPolicyRepository;
use adapter::repository::policy_postgres::PolicyPostgresRepository;
use domain::entity::policy::Policy;
use domain::entity::policy_bundle::PolicyBundle;
use domain::repository::PolicyBundleRepository;
use domain::repository::PolicyRepository;
use infrastructure::cache::PolicyCache;
use infrastructure::config::Config;
use infrastructure::kafka_producer::{
    KafkaPolicyProducer, NoopPolicyEventPublisher, PolicyEventPublisher,
};
use infrastructure::opa_client::OpaClient;

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

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-policy-server",
    ));

    // Cache
    let cache = Arc::new(PolicyCache::new(
        cfg.cache.max_entries,
        cfg.cache.ttl_seconds,
    ));

    // Repositories: PostgreSQL or InMemory fallback
    let (policy_repo, bundle_repo): (
        Arc<dyn PolicyRepository>,
        Arc<dyn PolicyBundleRepository>,
    ) = if let Some(ref db_cfg) = cfg.database {
        info!("connecting to PostgreSQL: {}:{}/{}", db_cfg.host, db_cfg.port, db_cfg.name);
        let pool = Arc::new(infrastructure::database::connect(db_cfg).await?);
        info!("PostgreSQL connection established");

        let pg_policy_repo: Arc<dyn PolicyRepository> =
            Arc::new(PolicyPostgresRepository::new(pool.clone()));
        let cached_policy_repo: Arc<dyn PolicyRepository> =
            Arc::new(CachedPolicyRepository::new(pg_policy_repo, cache.clone()));

        let bundle_repo: Arc<dyn PolicyBundleRepository> =
            Arc::new(BundlePostgresRepository::new(pool));

        (cached_policy_repo, bundle_repo)
    } else {
        info!("no database configured, using in-memory repositories");
        let policy_repo: Arc<dyn PolicyRepository> =
            Arc::new(InMemoryPolicyRepository::new());
        let bundle_repo: Arc<dyn PolicyBundleRepository> =
            Arc::new(InMemoryPolicyBundleRepository::new());
        (policy_repo, bundle_repo)
    };

    // Kafka event publisher
    let _event_publisher: Arc<dyn PolicyEventPublisher> = if let Some(ref kafka_cfg) = cfg.kafka {
        info!(
            brokers = %kafka_cfg.brokers.join(","),
            topic = %kafka_cfg.topic,
            "initializing Kafka policy event publisher"
        );
        let producer = KafkaPolicyProducer::new(kafka_cfg)?
            .with_metrics(metrics.clone());
        Arc::new(producer)
    } else {
        info!("no Kafka configured, using no-op event publisher");
        Arc::new(NoopPolicyEventPublisher)
    };

    // OPA client
    let opa_client: Option<Arc<OpaClient>> = if let Some(ref opa_cfg) = cfg.opa {
        info!(url = %opa_cfg.url, timeout_ms = %opa_cfg.timeout_ms, "initializing OPA client");
        Some(Arc::new(OpaClient::new(opa_cfg)?))
    } else {
        info!("no OPA configured, using policy.enabled fallback");
        None
    };

    let create_policy_uc = Arc::new(usecase::CreatePolicyUseCase::new(policy_repo.clone()));
    let get_policy_uc = Arc::new(usecase::GetPolicyUseCase::new(policy_repo.clone()));
    let update_policy_uc = Arc::new(usecase::UpdatePolicyUseCase::new(policy_repo.clone()));
    let delete_policy_uc = Arc::new(usecase::DeletePolicyUseCase::new(policy_repo.clone()));
    let evaluate_policy_uc = Arc::new(usecase::EvaluatePolicyUseCase::new(policy_repo.clone(), opa_client));
    let create_bundle_uc = Arc::new(usecase::CreateBundleUseCase::new(bundle_repo.clone()));
    let list_bundles_uc = Arc::new(usecase::ListBundlesUseCase::new(bundle_repo));

    let grpc_svc = Arc::new(PolicyGrpcService::new(
        evaluate_policy_uc.clone(),
        get_policy_uc.clone(),
    ));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = if let Some(ref auth_cfg) = cfg.auth {
        info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for policy-server");
        let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
            &auth_cfg.jwks_url,
            &auth_cfg.issuer,
            &auth_cfg.audience,
            std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
        ));
        Some(adapter::middleware::auth::PolicyAuthState {
            verifier: jwks_verifier,
        })
    } else {
        info!("no auth configured, policy-server running without authentication");
        None
    };

    let mut state = adapter::handler::AppState {
        policy_repo,
        create_policy_uc,
        get_policy_uc,
        update_policy_uc,
        delete_policy_uc,
        evaluate_policy_uc,
        create_bundle_uc,
        list_bundles_uc,
        metrics: metrics.clone(),
        auth_state: None,
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    let app = adapter::handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    // gRPC server
    use proto::k1s0::system::policy::v1::policy_service_server::PolicyServiceServer;

    let policy_tonic = adapter::grpc::PolicyServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], 50051).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
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
