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
use adapter::repository::scheduler_job_postgres::SchedulerJobPostgresRepository;
use domain::entity::scheduler_job::SchedulerJob;
use domain::repository::SchedulerJobRepository;
use infrastructure::cache::JobCache;
use infrastructure::config::Config;
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
    let job_repo: Arc<dyn SchedulerJobRepository> = if let Some(ref db_cfg) = cfg.database {
        info!("connecting to PostgreSQL: {}:{}/{}", db_cfg.host, db_cfg.port, db_cfg.name);
        let pool = infrastructure::database::connect(db_cfg).await?;
        Arc::new(SchedulerJobPostgresRepository::new(Arc::new(pool)))
    } else {
        info!("no database configured, using in-memory repository");
        Arc::new(InMemorySchedulerJobRepository::new())
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
        event_publisher.clone(),
    ));
    let pause_job_uc = Arc::new(usecase::PauseJobUseCase::new(job_repo.clone()));
    let resume_job_uc = Arc::new(usecase::ResumeJobUseCase::new(job_repo.clone()));
    let update_job_uc = Arc::new(usecase::UpdateJobUseCase::new(job_repo.clone()));
    let list_executions_uc = Arc::new(usecase::ListExecutionsUseCase::new(job_repo.clone()));

    let grpc_svc = Arc::new(SchedulerGrpcService::new(trigger_job_uc.clone()));

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-scheduler-server",
    ));

    let state = adapter::handler::AppState {
        list_jobs_uc,
        create_job_uc,
        get_job_uc,
        delete_job_uc,
        pause_job_uc,
        resume_job_uc,
        update_job_uc,
        trigger_job_uc,
        list_executions_uc,
        metrics,
    };

    let app = adapter::handler::router(state);

    // gRPC server
    use proto::k1s0::system::scheduler::v1::scheduler_service_server::SchedulerServiceServer;

    let scheduler_tonic = adapter::grpc::SchedulerServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], 50051).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_future = async move {
        tonic::transport::Server::builder()
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

    // Graceful shutdown: close Kafka producer
    if let Err(e) = event_publisher.close().await {
        tracing::warn!("failed to close event publisher: {}", e);
    }

    Ok(())
}

// --- InMemory Repository (fallback) ---

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
