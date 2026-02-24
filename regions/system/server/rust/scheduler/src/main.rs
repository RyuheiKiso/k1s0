#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;
use uuid::Uuid;

mod adapter;
mod domain;
mod infrastructure;
mod usecase;

use adapter::grpc::SchedulerGrpcService;
use domain::entity::scheduler_job::SchedulerJob;
use domain::repository::SchedulerJobRepository;
use infrastructure::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-scheduler-server".to_string(),
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
        "starting scheduler server"
    );

    let job_repo: Arc<dyn SchedulerJobRepository> =
        Arc::new(InMemorySchedulerJobRepository::new());

    let _create_job_uc = Arc::new(usecase::CreateJobUseCase::new(job_repo.clone()));
    let _get_job_uc = Arc::new(usecase::GetJobUseCase::new(job_repo.clone()));
    let trigger_job_uc = Arc::new(usecase::TriggerJobUseCase::new(job_repo.clone()));
    let _pause_job_uc = Arc::new(usecase::PauseJobUseCase::new(job_repo.clone()));
    let _resume_job_uc = Arc::new(usecase::ResumeJobUseCase::new(job_repo));

    let _grpc_svc = Arc::new(SchedulerGrpcService::new(trigger_job_uc));

    let app = axum::Router::new()
        .route("/healthz", axum::routing::get(adapter::handler::health::healthz))
        .route("/readyz", axum::routing::get(adapter::handler::health::readyz));

    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let grpc_addr: SocketAddr = "0.0.0.0:9090".parse()?;
    info!("gRPC server starting on {}", grpc_addr);

    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(grpc_addr).await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("failed to bind gRPC addr {}: {}", grpc_addr, e);
                return;
            }
        };
        tracing::info!("gRPC listener bound on {}", grpc_addr);
        drop(listener);
    });

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// --- InMemory Repository ---

struct InMemorySchedulerJobRepository {
    jobs: tokio::sync::RwLock<HashMap<Uuid, SchedulerJob>>,
}

impl InMemorySchedulerJobRepository {
    fn new() -> Self {
        Self {
            jobs: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl SchedulerJobRepository for InMemorySchedulerJobRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<SchedulerJob>> {
        let jobs = self.jobs.read().await;
        Ok(jobs.get(id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<SchedulerJob>> {
        let jobs = self.jobs.read().await;
        Ok(jobs.values().cloned().collect())
    }

    async fn create(&self, job: &SchedulerJob) -> anyhow::Result<()> {
        let mut jobs = self.jobs.write().await;
        jobs.insert(job.id, job.clone());
        Ok(())
    }

    async fn update(&self, job: &SchedulerJob) -> anyhow::Result<()> {
        let mut jobs = self.jobs.write().await;
        jobs.insert(job.id, job.clone());
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut jobs = self.jobs.write().await;
        Ok(jobs.remove(id).is_some())
    }

    async fn find_active_jobs(&self) -> anyhow::Result<Vec<SchedulerJob>> {
        let jobs = self.jobs.read().await;
        Ok(jobs
            .values()
            .filter(|j| j.status == "active")
            .cloned()
            .collect())
    }
}
