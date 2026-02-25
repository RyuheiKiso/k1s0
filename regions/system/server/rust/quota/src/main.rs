#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::info;

mod adapter;
mod domain;
mod infrastructure;
mod proto;
mod usecase;

use adapter::grpc::QuotaGrpcService;
use domain::entity::quota::QuotaPolicy;
use domain::repository::{QuotaPolicyRepository, QuotaUsageRepository};
use infrastructure::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-quota-server".to_string(),
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
        "starting quota server"
    );

    // --- Repository initialization: PostgreSQL or InMemory fallback ---
    let (policy_repo, usage_repo): (
        Arc<dyn QuotaPolicyRepository>,
        Arc<dyn QuotaUsageRepository>,
    ) = if let Some(ref db_cfg) = cfg.database {
        info!(url = %db_cfg.url, "connecting to PostgreSQL");
        match infrastructure::database::create_pool(&db_cfg.url, db_cfg.max_connections).await {
            Ok(pool) => {
                let pool = Arc::new(pool);
                info!("PostgreSQL connection pool created successfully");
                let policy_repo = Arc::new(
                    adapter::repository::QuotaPolicyPostgresRepository::new(pool.clone()),
                );
                let usage_repo = Arc::new(
                    adapter::repository::QuotaUsagePostgresRepository::new(pool),
                );
                (policy_repo, usage_repo)
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "failed to connect to PostgreSQL, falling back to InMemory"
                );
                (
                    Arc::new(InMemoryQuotaPolicyRepository::new()) as Arc<dyn QuotaPolicyRepository>,
                    Arc::new(InMemoryQuotaUsageRepository::new()) as Arc<dyn QuotaUsageRepository>,
                )
            }
        }
    } else {
        info!("no database config found, using InMemory repositories");
        (
            Arc::new(InMemoryQuotaPolicyRepository::new()) as Arc<dyn QuotaPolicyRepository>,
            Arc::new(InMemoryQuotaUsageRepository::new()) as Arc<dyn QuotaUsageRepository>,
        )
    };

    // --- Kafka event publisher initialization ---
    let _event_publisher: Arc<dyn infrastructure::kafka_producer::QuotaEventPublisher> =
        if let Some(ref kafka_cfg) = cfg.kafka {
            match infrastructure::kafka_producer::KafkaQuotaProducer::new(
                &kafka_cfg.brokers.join(","),
                &kafka_cfg.security_protocol,
                &kafka_cfg.topic_exceeded,
            ) {
                Ok(producer) => {
                    info!("Kafka producer initialized for quota exceeded events");
                    Arc::new(producer)
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "failed to create Kafka producer, using NoopQuotaEventPublisher"
                    );
                    Arc::new(infrastructure::kafka_producer::NoopQuotaEventPublisher)
                }
            }
        } else {
            info!("no Kafka config found, using NoopQuotaEventPublisher");
            Arc::new(infrastructure::kafka_producer::NoopQuotaEventPublisher)
        };

    let create_policy_uc =
        Arc::new(usecase::CreateQuotaPolicyUseCase::new(policy_repo.clone()));
    let get_policy_uc =
        Arc::new(usecase::GetQuotaPolicyUseCase::new(policy_repo.clone()));
    let list_policies_uc =
        Arc::new(usecase::ListQuotaPoliciesUseCase::new(policy_repo.clone()));
    let update_policy_uc =
        Arc::new(usecase::UpdateQuotaPolicyUseCase::new(policy_repo.clone()));
    let delete_policy_uc =
        Arc::new(usecase::DeleteQuotaPolicyUseCase::new(policy_repo.clone()));
    let get_usage_uc = Arc::new(usecase::GetQuotaUsageUseCase::new(
        policy_repo.clone(),
        usage_repo.clone(),
    ));
    let increment_usage_uc = Arc::new(usecase::IncrementQuotaUsageUseCase::new(
        policy_repo.clone(),
        usage_repo.clone(),
    ));
    let _reset_usage_uc = Arc::new(usecase::ResetQuotaUsageUseCase::new(
        policy_repo,
        usage_repo,
    ));

    let grpc_svc = Arc::new(QuotaGrpcService::new(
        create_policy_uc.clone(),
        get_policy_uc.clone(),
        list_policies_uc.clone(),
        update_policy_uc.clone(),
        delete_policy_uc,
        get_usage_uc.clone(),
        increment_usage_uc.clone(),
    ));

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-quota-server",
    ));

    let state = adapter::handler::AppState {
        create_policy_uc,
        get_policy_uc,
        list_policies_uc,
        update_policy_uc,
        get_usage_uc,
        increment_usage_uc,
        metrics,
    };

    let app = adapter::handler::router(state);

    // gRPC server
    use proto::k1s0::system::quota::v1::quota_service_server::QuotaServiceServer;

    let quota_tonic = adapter::grpc::QuotaServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], 50051).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_future = async move {
        tonic::transport::Server::builder()
            .add_service(QuotaServiceServer::new(quota_tonic))
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

// --- InMemory Repositories ---

struct InMemoryQuotaPolicyRepository {
    policies: RwLock<HashMap<String, QuotaPolicy>>,
}

impl InMemoryQuotaPolicyRepository {
    fn new() -> Self {
        Self {
            policies: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl QuotaPolicyRepository for InMemoryQuotaPolicyRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<QuotaPolicy>> {
        let policies = self.policies.read().await;
        Ok(policies.get(id).cloned())
    }

    async fn find_all(&self, page: u32, page_size: u32) -> anyhow::Result<(Vec<QuotaPolicy>, u64)> {
        let policies = self.policies.read().await;
        let all: Vec<QuotaPolicy> = policies.values().cloned().collect();
        let total = all.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let items: Vec<QuotaPolicy> = all.into_iter().skip(start).take(page_size as usize).collect();
        Ok((items, total))
    }

    async fn create(&self, policy: &QuotaPolicy) -> anyhow::Result<()> {
        let mut policies = self.policies.write().await;
        policies.insert(policy.id.clone(), policy.clone());
        Ok(())
    }

    async fn update(&self, policy: &QuotaPolicy) -> anyhow::Result<()> {
        let mut policies = self.policies.write().await;
        policies.insert(policy.id.clone(), policy.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        let mut policies = self.policies.write().await;
        Ok(policies.remove(id).is_some())
    }
}

struct InMemoryQuotaUsageRepository {
    counters: RwLock<HashMap<String, u64>>,
}

impl InMemoryQuotaUsageRepository {
    fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl QuotaUsageRepository for InMemoryQuotaUsageRepository {
    async fn get_usage(&self, quota_id: &str) -> anyhow::Result<Option<u64>> {
        let counters = self.counters.read().await;
        Ok(counters.get(quota_id).cloned())
    }

    async fn increment(&self, quota_id: &str, amount: u64) -> anyhow::Result<u64> {
        let mut counters = self.counters.write().await;
        let counter = counters.entry(quota_id.to_string()).or_insert(0);
        *counter += amount;
        Ok(*counter)
    }

    async fn reset(&self, quota_id: &str) -> anyhow::Result<()> {
        let mut counters = self.counters.write().await;
        counters.insert(quota_id.to_string(), 0);
        Ok(())
    }
}
