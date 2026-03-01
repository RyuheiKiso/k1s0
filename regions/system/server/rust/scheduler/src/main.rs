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

use adapter::grpc::SchedulerGrpcService;
use adapter::repository::scheduler_execution_postgres::SchedulerExecutionPostgresRepository;
use adapter::repository::scheduler_job_postgres::SchedulerJobPostgresRepository;
use domain::entity::scheduler_execution::SchedulerExecution;
use domain::entity::scheduler_job::SchedulerJob;
use domain::repository::{SchedulerExecutionRepository, SchedulerJobRepository};
use infrastructure::cache::JobCache;
use infrastructure::config::Config;
use infrastructure::cron_engine::CronSchedulerEngine;
use infrastructure::kafka_producer::{
    KafkaSchedulerProducer, NoopSchedulerEventPublisher, SchedulerEventPublisher,
};

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
        log_format: "json".to_string(),
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

    // --- Repository: PostgreSQL or InMemory fallback ---
    let (job_repo, execution_repo, distributed_lock): (
        Arc<dyn SchedulerJobRepository>,
        Arc<dyn SchedulerExecutionRepository>,
        Arc<dyn k1s0_distributed_lock::DistributedLock>,
    ) = if let Some(ref db_cfg) = cfg.database {
        info!("connecting to PostgreSQL: {}:{}/{}", db_cfg.host, db_cfg.port, db_cfg.name);
        let pool = Arc::new(infrastructure::database::connect(db_cfg).await?);
        let pg_lock = k1s0_distributed_lock::PostgresDistributedLock::new(
            pool.as_ref().clone(),
        )
        .with_prefix("scheduler");
        (
            Arc::new(SchedulerJobPostgresRepository::new(pool.clone())),
            Arc::new(SchedulerExecutionPostgresRepository::new(pool)),
            Arc::new(pg_lock),
        )
    } else {
        info!("no database configured, using in-memory repository");
        (
            Arc::new(InMemorySchedulerJobRepository::new()),
            Arc::new(InMemorySchedulerExecutionRepository::new()),
            Arc::new(k1s0_distributed_lock::InMemoryDistributedLock::new()),
        )
    };

    // --- Kafka: real or Noop fallback ---
    let event_publisher: Arc<dyn SchedulerEventPublisher> = if let Some(ref kafka_cfg) = cfg.kafka {
        info!("connecting to Kafka brokers: {:?}", kafka_cfg.brokers);
        let producer = KafkaSchedulerProducer::new(kafka_cfg)?;
        Arc::new(producer)
    } else {
        info!("no Kafka configured, using noop event publisher");
        Arc::new(NoopSchedulerEventPublisher)
    };

    // --- Cache ---
    let _job_cache = Arc::new(JobCache::default_config());
    info!("job cache initialized (max=1000, TTL=120s)");

    let list_jobs_uc = Arc::new(usecase::ListJobsUseCase::new(job_repo.clone()));
    let create_job_uc = Arc::new(usecase::CreateJobUseCase::new(job_repo.clone()));
    let get_job_uc = Arc::new(usecase::GetJobUseCase::new(job_repo.clone()));
    let delete_job_uc = Arc::new(usecase::DeleteJobUseCase::new(job_repo.clone()));
    let trigger_job_uc = Arc::new(usecase::TriggerJobUseCase::with_publisher(
        job_repo.clone(),
        execution_repo.clone(),
        event_publisher.clone(),
    ));
    let pause_job_uc = Arc::new(usecase::PauseJobUseCase::new(job_repo.clone()));
    let resume_job_uc = Arc::new(usecase::ResumeJobUseCase::new(job_repo.clone()));
    let update_job_uc = Arc::new(usecase::UpdateJobUseCase::new(job_repo.clone()));
    let list_executions_uc = Arc::new(usecase::ListExecutionsUseCase::new(
        job_repo.clone(),
        execution_repo.clone(),
    ));

    let grpc_svc = Arc::new(SchedulerGrpcService::new(trigger_job_uc.clone()));

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-scheduler-server",
    ));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = if let Some(ref auth_cfg) = cfg.auth {
        info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for scheduler-server");
        let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
            &auth_cfg.jwks_url,
            &auth_cfg.issuer,
            &auth_cfg.audience,
            std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
        ));
        Some(adapter::middleware::auth::SchedulerAuthState {
            verifier: jwks_verifier,
        })
    } else {
        info!("no auth configured, scheduler-server running without authentication");
        None
    };

    let mut state = adapter::handler::AppState {
        list_jobs_uc,
        create_job_uc,
        get_job_uc,
        delete_job_uc,
        pause_job_uc,
        resume_job_uc,
        update_job_uc,
        trigger_job_uc,
        list_executions_uc,
        metrics: metrics.clone(),
        auth_state: None,
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    let app = adapter::handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    // --- Cron Scheduler Engine ---
    let cron_engine = CronSchedulerEngine::new(
        job_repo.clone(),
        execution_repo.clone(),
        distributed_lock,
    );
    let _cron_handle = cron_engine.start();
    info!("cron scheduler engine started");

    // gRPC server
    use proto::k1s0::system::scheduler::v1::scheduler_service_server::SchedulerServiceServer;

    let scheduler_tonic = adapter::grpc::SchedulerServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(SchedulerServiceServer::new(scheduler_tonic))
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

    // Graceful shutdown
    cron_engine.stop();
    info!("cron scheduler engine stopped");

    if let Err(e) = event_publisher.close().await {
        tracing::warn!("failed to close event publisher: {}", e);
    }

    Ok(())
}

// --- InMemory Repository (fallback) ---

struct InMemorySchedulerExecutionRepository {
    executions: tokio::sync::RwLock<HashMap<Uuid, SchedulerExecution>>,
}

impl InMemorySchedulerExecutionRepository {
    fn new() -> Self {
        Self {
            executions: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl SchedulerExecutionRepository for InMemorySchedulerExecutionRepository {
    async fn create(&self, execution: &SchedulerExecution) -> anyhow::Result<()> {
        let mut execs = self.executions.write().await;
        execs.insert(execution.id, execution.clone());
        Ok(())
    }

    async fn find_by_job_id(&self, job_id: &Uuid) -> anyhow::Result<Vec<SchedulerExecution>> {
        let execs = self.executions.read().await;
        Ok(execs
            .values()
            .filter(|e| e.job_id == *job_id)
            .cloned()
            .collect())
    }

    async fn update_status(
        &self,
        id: &Uuid,
        status: String,
        error_message: Option<String>,
    ) -> anyhow::Result<()> {
        let mut execs = self.executions.write().await;
        if let Some(exec) = execs.get_mut(id) {
            exec.status = status;
            exec.error_message = error_message;
            exec.completed_at = Some(chrono::Utc::now());
        }
        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<SchedulerExecution>> {
        let execs = self.executions.read().await;
        Ok(execs.get(id).cloned())
    }
}

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
