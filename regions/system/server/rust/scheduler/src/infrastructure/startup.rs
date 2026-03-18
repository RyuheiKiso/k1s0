use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use super::cache::JobCache;
use super::config::Config;
use super::cron_engine::CronSchedulerEngine;
use super::job_executor::TargetJobExecutor;
use super::kafka_producer::{
    KafkaSchedulerProducer, NoopSchedulerEventPublisher, SchedulerEventPublisher,
};
use crate::adapter::grpc::SchedulerGrpcService;
use crate::adapter::repository::scheduler_execution_postgres::SchedulerExecutionPostgresRepository;
use crate::adapter::repository::scheduler_job_postgres::SchedulerJobPostgresRepository;
use crate::domain::entity::scheduler_execution::SchedulerExecution;
use crate::domain::entity::scheduler_job::SchedulerJob;
use crate::domain::repository::{SchedulerExecutionRepository, SchedulerJobRepository};

pub async fn run() -> anyhow::Result<()> {
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-scheduler-server".to_string(),
        version: "0.1.0".to_string(),
        tier: "system".to_string(),
        environment: cfg.app.environment.clone(),
        trace_endpoint: cfg
            .observability
            .trace
            .enabled
            .then(|| cfg.observability.trace.endpoint.clone()),
        sample_rate: cfg.observability.trace.sample_rate,
        log_level: cfg.observability.log.level.clone(),
        log_format: cfg.observability.log.format.clone(),
    };
    k1s0_telemetry::init_telemetry(&telemetry_cfg).expect("failed to init telemetry");

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
        info!(
            "connecting to PostgreSQL: {}:{}/{}",
            db_cfg.host, db_cfg.port, db_cfg.name
        );
        let pool = Arc::new(super::database::connect(db_cfg).await?);
        let pg_lock = k1s0_distributed_lock::PostgresDistributedLock::new(pool.as_ref().clone())
            .with_prefix("scheduler");
        (
            Arc::new(SchedulerJobPostgresRepository::new(pool.clone())),
            Arc::new(SchedulerExecutionPostgresRepository::new(pool)),
            Arc::new(pg_lock),
        )
    } else {
        // infra_guard: stable サービスでは DB 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "scheduler",
            k1s0_server_common::InfraKind::Database,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no database configured, using in-memory repository (dev/test bypass)");
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
        // infra_guard: stable サービスでは Kafka 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "scheduler",
            k1s0_server_common::InfraKind::Kafka,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no Kafka configured, using noop event publisher (dev/test bypass)");
        Arc::new(NoopSchedulerEventPublisher)
    };

    // --- Cache ---
    let _job_cache = Arc::new(JobCache::default_config());
    info!("job cache initialized (max=1000, TTL=120s)");

    let job_executor = Arc::new(TargetJobExecutor::new(cfg.kafka.as_ref())?);

    let list_jobs_uc = Arc::new(crate::usecase::ListJobsUseCase::new(job_repo.clone()));
    let create_job_uc = Arc::new(crate::usecase::CreateJobUseCase::new(
        job_repo.clone(),
        event_publisher.clone(),
    ));
    let get_job_uc = Arc::new(crate::usecase::GetJobUseCase::new(job_repo.clone()));
    let delete_job_uc = Arc::new(crate::usecase::DeleteJobUseCase::new(
        job_repo.clone(),
        execution_repo.clone(),
    ));
    let trigger_job_uc = Arc::new(crate::usecase::TriggerJobUseCase::with_dependencies(
        job_repo.clone(),
        execution_repo.clone(),
        job_executor.clone(),
        event_publisher.clone(),
    ));
    let pause_job_uc = Arc::new(crate::usecase::PauseJobUseCase::new(job_repo.clone()));
    let resume_job_uc = Arc::new(crate::usecase::ResumeJobUseCase::new(job_repo.clone()));
    let update_job_uc = Arc::new(crate::usecase::UpdateJobUseCase::new(job_repo.clone()));
    let list_executions_uc = Arc::new(crate::usecase::ListExecutionsUseCase::new(
        job_repo.clone(),
        execution_repo.clone(),
    ));

    let grpc_svc = Arc::new(SchedulerGrpcService::new(
        create_job_uc.clone(),
        get_job_uc.clone(),
        list_jobs_uc.clone(),
        update_job_uc.clone(),
        delete_job_uc.clone(),
        pause_job_uc.clone(),
        resume_job_uc.clone(),
        trigger_job_uc.clone(),
        list_executions_uc.clone(),
        execution_repo.clone(),
    ));

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-scheduler-server",
    ));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = k1s0_server_common::require_auth_state(
        "scheduler-server",
        &cfg.app.environment,
        cfg.auth.as_ref().map(|auth_cfg| {
            info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for scheduler-server");
            let jwks_verifier = Arc::new(
                k1s0_auth::JwksVerifier::new(
                    &auth_cfg.jwks_url,
                    &auth_cfg.issuer,
                    &auth_cfg.audience,
                    std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
                )
                .expect("Failed to create JWKS verifier"),
            );
            crate::adapter::middleware::auth::SchedulerAuthState {
                verifier: jwks_verifier,
            }
        }),
    )?;

    let mut state = crate::adapter::handler::AppState {
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
    let grpc_auth_state = state.auth_state.clone();

    let app = crate::adapter::handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

    // --- Cron Scheduler Engine ---
    let cron_engine = CronSchedulerEngine::new(
        job_repo.clone(),
        execution_repo.clone(),
        job_executor,
        distributed_lock,
    );
    let _cron_handle = cron_engine.start();
    info!("cron scheduler engine started");

    // gRPC server
    use crate::proto::k1s0::system::scheduler::v1::scheduler_service_server::SchedulerServiceServer;

    let scheduler_tonic = crate::adapter::grpc::SchedulerServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_auth_layer =
        crate::adapter::middleware::grpc_auth::GrpcAuthLayer::new(grpc_auth_state);
    // gRPCサーバーのグレースフルシャットダウン用シグナル
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(SchedulerServiceServer::new(scheduler_tonic))
            .serve_with_shutdown(grpc_addr, async move {
                let _ = grpc_shutdown.await;
            })
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    // REST server
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    // RESTサーバーのグレースフルシャットダウン設定
    let rest_future = axum::serve(listener, app).with_graceful_shutdown(async {
        let _ = k1s0_server_common::shutdown::shutdown_signal().await;
    });

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

    // テレメトリのシャットダウン処理
    k1s0_telemetry::shutdown();

    Ok(())
}

// --- InMemory Repository (fallback) ---

struct InMemorySchedulerExecutionRepository {
    executions: tokio::sync::RwLock<HashMap<String, SchedulerExecution>>,
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
        execs.insert(execution.id.clone(), execution.clone());
        Ok(())
    }

    async fn find_by_job_id(&self, job_id: &str) -> anyhow::Result<Vec<SchedulerExecution>> {
        let execs = self.executions.read().await;
        Ok(execs
            .values()
            .filter(|e| e.job_id == job_id)
            .cloned()
            .collect())
    }

    async fn update_status(
        &self,
        id: &str,
        status: String,
        error_message: Option<String>,
    ) -> anyhow::Result<()> {
        let mut execs = self.executions.write().await;
        if let Some(exec) = execs.get_mut(id) {
            exec.status = status;
            exec.error_message = error_message;
            exec.finished_at = Some(chrono::Utc::now());
        }
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<SchedulerExecution>> {
        let execs = self.executions.read().await;
        Ok(execs.get(id).cloned())
    }
}

struct InMemorySchedulerJobRepository {
    jobs: tokio::sync::RwLock<HashMap<String, SchedulerJob>>,
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
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<SchedulerJob>> {
        let jobs = self.jobs.read().await;
        Ok(jobs.get(id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<SchedulerJob>> {
        let jobs = self.jobs.read().await;
        Ok(jobs.values().cloned().collect())
    }

    async fn create(&self, job: &SchedulerJob) -> anyhow::Result<()> {
        let mut jobs = self.jobs.write().await;
        jobs.insert(job.id.clone(), job.clone());
        Ok(())
    }

    async fn update(&self, job: &SchedulerJob) -> anyhow::Result<()> {
        let mut jobs = self.jobs.write().await;
        jobs.insert(job.id.clone(), job.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
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
