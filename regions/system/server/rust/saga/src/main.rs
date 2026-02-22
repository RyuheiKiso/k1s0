#![allow(dead_code, unused_imports)]

use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod infrastructure;
mod proto;
mod usecase;

use adapter::grpc::{SagaGrpcService, SagaServiceTonic};
use adapter::handler::{self, AppState};
use adapter::repository::saga_postgres::SagaPostgresRepository;
use adapter::repository::workflow_in_memory::InMemoryWorkflowRepository;
use domain::repository::WorkflowRepository;
use infrastructure::config::Config;
use infrastructure::grpc_caller::{ServiceRegistry, TonicGrpcCaller};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Telemetry
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-saga-server".to_string(),
        version: "0.1.0".to_string(),
        tier: "system".to_string(),
        environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string()),
        trace_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
        sample_rate: 1.0,
        log_level: "info".to_string(),
    };
    k1s0_telemetry::init_telemetry(&telemetry_cfg).expect("failed to init telemetry");

    // Config
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let config_content = std::fs::read_to_string(&config_path)?;
    let cfg: Config = serde_yaml::from_str(&config_content)?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting saga server"
    );

    // Database pool (optional)
    let db_pool = if let Some(ref db_config) = cfg.database {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_config.connection_url());
        info!("connecting to database");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(db_config.max_open_conns)
            .connect(&url)
            .await?;
        info!("database connection pool established");
        Some(pool)
    } else if let Ok(url) = std::env::var("DATABASE_URL") {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(25)
            .connect(&url)
            .await?;
        info!("database connection pool established from DATABASE_URL");
        Some(pool)
    } else {
        info!("no database configured, using in-memory repositories");
        None
    };

    // Saga repository
    let saga_repo: Arc<dyn domain::repository::SagaRepository> = if let Some(ref pool) = db_pool {
        Arc::new(SagaPostgresRepository::new(pool.clone()))
    } else {
        Arc::new(InMemorySagaRepository::new())
    };

    // Workflow repository
    let workflow_loader = infrastructure::workflow_loader::WorkflowLoader::new(&cfg.saga.workflow_dir);
    let loaded_definitions = workflow_loader.load_all().await?;
    let workflow_repo = Arc::new(InMemoryWorkflowRepository::new());
    for workflow in &loaded_definitions {
        workflow_repo.register(workflow.clone()).await?;
    }
    info!(count = loaded_definitions.len(), "workflow definitions loaded from directory via WorkflowLoader");

    // Service registry + gRPC caller
    let registry = Arc::new(ServiceRegistry::new(cfg.services.clone()));
    let grpc_caller: Arc<dyn infrastructure::grpc_caller::GrpcStepCaller> =
        Arc::new(TonicGrpcCaller::new(registry));

    // Kafka publisher (optional)
    let publisher: Option<Arc<dyn infrastructure::kafka_producer::SagaEventPublisher>> =
        if let Some(ref kafka_config) = cfg.kafka {
            match infrastructure::kafka_producer::KafkaProducer::new(kafka_config) {
                Ok(producer) => {
                    info!("kafka producer initialized");
                    Some(Arc::new(producer))
                }
                Err(e) => {
                    tracing::warn!(error = %e, "failed to create kafka producer, events will not be published");
                    None
                }
            }
        } else {
            info!("no kafka configured, saga events will not be published");
            None
        };

    // Use cases
    let execute_saga_uc = Arc::new(usecase::ExecuteSagaUseCase::new(
        saga_repo.clone(),
        grpc_caller.clone(),
        publisher,
    ));

    let start_saga_uc = Arc::new(usecase::StartSagaUseCase::new(
        saga_repo.clone(),
        workflow_repo.clone() as Arc<dyn domain::repository::WorkflowRepository>,
        execute_saga_uc.clone(),
    ));

    let get_saga_uc = Arc::new(usecase::GetSagaUseCase::new(saga_repo.clone()));
    let list_sagas_uc = Arc::new(usecase::ListSagasUseCase::new(saga_repo.clone()));
    let cancel_saga_uc = Arc::new(usecase::CancelSagaUseCase::new(saga_repo.clone()));
    let register_workflow_uc = Arc::new(usecase::RegisterWorkflowUseCase::new(
        workflow_repo.clone() as Arc<dyn domain::repository::WorkflowRepository>,
    ));
    let list_workflows_uc = Arc::new(usecase::ListWorkflowsUseCase::new(
        workflow_repo.clone() as Arc<dyn domain::repository::WorkflowRepository>
    ));

    // Startup recovery
    let recover_uc = usecase::RecoverSagasUseCase::new(
        saga_repo.clone(),
        workflow_repo.clone() as Arc<dyn domain::repository::WorkflowRepository>,
        execute_saga_uc.clone(),
    );
    let recovered = recover_uc.execute().await?;
    if recovered > 0 {
        info!(count = recovered, "sagas recovered at startup");
    }

    // AppState (REST handler用)
    let state = AppState {
        start_saga_uc,
        get_saga_uc,
        list_sagas_uc,
        cancel_saga_uc,
        register_workflow_uc,
        list_workflows_uc,
        metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-saga-server")),
    };

    // gRPC service
    let saga_grpc_svc = Arc::new(SagaGrpcService::new(
        state.start_saga_uc.clone(),
        state.get_saga_uc.clone(),
        state.list_sagas_uc.clone(),
        state.cancel_saga_uc.clone(),
        state.register_workflow_uc.clone(),
        state.list_workflows_uc.clone(),
    ));
    use proto::k1s0::system::saga::v1::saga_service_server::SagaServiceServer;

    let saga_tonic = SagaServiceTonic::new(saga_grpc_svc);

    // Router
    let app = handler::router(state);

    // gRPC server (port 50051)
    let grpc_addr: SocketAddr = ([0, 0, 0, 0], 50051).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_future = async move {
        tonic::transport::Server::builder()
            .add_service(SagaServiceServer::new(saga_tonic))
            .serve(grpc_addr)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    // REST server
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    let rest_future = axum::serve(listener, app);

    // REST と gRPC を並行起動
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

// --- In-memory Saga Repository for dev mode ---

struct InMemorySagaRepository {
    states: tokio::sync::RwLock<Vec<domain::entity::saga_state::SagaState>>,
    step_logs: tokio::sync::RwLock<Vec<domain::entity::saga_step_log::SagaStepLog>>,
}

impl InMemorySagaRepository {
    fn new() -> Self {
        Self {
            states: tokio::sync::RwLock::new(Vec::new()),
            step_logs: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl domain::repository::SagaRepository for InMemorySagaRepository {
    async fn create(&self, state: &domain::entity::saga_state::SagaState) -> anyhow::Result<()> {
        self.states.write().await.push(state.clone());
        Ok(())
    }

    async fn update_with_step_log(
        &self,
        state: &domain::entity::saga_state::SagaState,
        log: &domain::entity::saga_step_log::SagaStepLog,
    ) -> anyhow::Result<()> {
        let mut states = self.states.write().await;
        if let Some(s) = states.iter_mut().find(|s| s.saga_id == state.saga_id) {
            *s = state.clone();
        }
        self.step_logs.write().await.push(log.clone());
        Ok(())
    }

    async fn update_status(
        &self,
        saga_id: uuid::Uuid,
        status: &domain::entity::saga_state::SagaStatus,
        error_message: Option<String>,
    ) -> anyhow::Result<()> {
        let mut states = self.states.write().await;
        if let Some(s) = states.iter_mut().find(|s| s.saga_id == saga_id) {
            s.status = status.clone();
            s.error_message = error_message;
            s.updated_at = chrono::Utc::now();
        }
        Ok(())
    }

    async fn find_by_id(
        &self,
        saga_id: uuid::Uuid,
    ) -> anyhow::Result<Option<domain::entity::saga_state::SagaState>> {
        let states = self.states.read().await;
        Ok(states.iter().find(|s| s.saga_id == saga_id).cloned())
    }

    async fn find_step_logs(
        &self,
        saga_id: uuid::Uuid,
    ) -> anyhow::Result<Vec<domain::entity::saga_step_log::SagaStepLog>> {
        let logs = self.step_logs.read().await;
        Ok(logs
            .iter()
            .filter(|l| l.saga_id == saga_id)
            .cloned()
            .collect())
    }

    async fn list(
        &self,
        params: &domain::repository::saga_repository::SagaListParams,
    ) -> anyhow::Result<(Vec<domain::entity::saga_state::SagaState>, i64)> {
        let states = self.states.read().await;
        let filtered: Vec<_> = states
            .iter()
            .filter(|s| {
                if let Some(ref wn) = params.workflow_name {
                    if s.workflow_name != *wn {
                        return false;
                    }
                }
                if let Some(ref st) = params.status {
                    if s.status != *st {
                        return false;
                    }
                }
                if let Some(ref ci) = params.correlation_id {
                    if s.correlation_id.as_deref() != Some(ci.as_str()) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        let total = filtered.len() as i64;
        let page = params.page.max(1);
        let page_size = params.page_size.max(1);
        let offset = ((page - 1) * page_size) as usize;
        let limit = page_size as usize;
        let paged: Vec<_> = filtered.into_iter().skip(offset).take(limit).collect();

        Ok((paged, total))
    }

    async fn find_incomplete(&self) -> anyhow::Result<Vec<domain::entity::saga_state::SagaState>> {
        let states = self.states.read().await;
        Ok(states
            .iter()
            .filter(|s| {
                matches!(
                    s.status,
                    domain::entity::saga_state::SagaStatus::Started
                        | domain::entity::saga_state::SagaStatus::Running
                        | domain::entity::saga_state::SagaStatus::Compensating
                )
            })
            .cloned()
            .collect())
    }
}
