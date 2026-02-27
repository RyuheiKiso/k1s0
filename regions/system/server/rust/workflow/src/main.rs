#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod infrastructure;
mod proto;
mod usecase;

use adapter::grpc::WorkflowGrpcService;
use domain::entity::workflow_definition::WorkflowDefinition;
use domain::entity::workflow_instance::WorkflowInstance;
use domain::entity::workflow_task::WorkflowTask;
use domain::repository::WorkflowDefinitionRepository;
use domain::repository::WorkflowInstanceRepository;
use domain::repository::WorkflowTaskRepository;
use infrastructure::config::Config;
use infrastructure::kafka_producer::{
    KafkaWorkflowEventPublisher, NoopWorkflowEventPublisher, WorkflowEventPublisher,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-workflow-server".to_string(),
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
        "starting workflow server"
    );

    // --- Repository / Event Publisher 初期化 ---
    // database 設定がある場合は PostgreSQL、なければ InMemory フォールバック
    let (def_repo, inst_repo, task_repo): (
        Arc<dyn WorkflowDefinitionRepository>,
        Arc<dyn WorkflowInstanceRepository>,
        Arc<dyn WorkflowTaskRepository>,
    ) = if let Some(ref db_cfg) = cfg.database {
        let pool = Arc::new(
            infrastructure::database::create_pool(
                &db_cfg.connection_url(),
                db_cfg.max_open_conns,
            )
            .await?,
        );
        info!("connected to PostgreSQL database");

        (
            Arc::new(adapter::repository::DefinitionPostgresRepository::new(pool.clone())),
            Arc::new(adapter::repository::InstancePostgresRepository::new(pool.clone())),
            Arc::new(adapter::repository::TaskPostgresRepository::new(pool)),
        )
    } else {
        info!("no database config found, using in-memory repositories");
        (
            Arc::new(InMemoryWorkflowDefinitionRepository::new()),
            Arc::new(InMemoryWorkflowInstanceRepository::new()),
            Arc::new(InMemoryWorkflowTaskRepository::new()),
        )
    };

    // Kafka event publisher
    let _event_publisher: Arc<dyn WorkflowEventPublisher> =
        if let Some(ref kafka_cfg) = cfg.kafka {
            let publisher = KafkaWorkflowEventPublisher::new(kafka_cfg)?;
            info!(topic = %publisher.topic(), "Kafka event publisher initialized");
            Arc::new(publisher)
        } else {
            info!("no Kafka config found, using noop event publisher");
            Arc::new(NoopWorkflowEventPublisher)
        };

    let create_wf_uc = Arc::new(usecase::CreateWorkflowUseCase::new(def_repo.clone()));
    let update_wf_uc = Arc::new(usecase::UpdateWorkflowUseCase::new(def_repo.clone()));
    let delete_wf_uc = Arc::new(usecase::DeleteWorkflowUseCase::new(def_repo.clone()));
    let get_wf_uc = Arc::new(usecase::GetWorkflowUseCase::new(def_repo.clone()));
    let list_wf_uc = Arc::new(usecase::ListWorkflowsUseCase::new(def_repo.clone()));
    let start_inst_uc = Arc::new(usecase::StartInstanceUseCase::new(
        def_repo.clone(),
        inst_repo.clone(),
        task_repo.clone(),
    ));
    let get_inst_uc = Arc::new(usecase::GetInstanceUseCase::new(inst_repo.clone()));
    let list_inst_uc = Arc::new(usecase::ListInstancesUseCase::new(inst_repo.clone()));
    let cancel_inst_uc = Arc::new(usecase::CancelInstanceUseCase::new(inst_repo.clone()));
    let list_tasks_uc = Arc::new(usecase::ListTasksUseCase::new(task_repo.clone()));
    let approve_task_uc = Arc::new(usecase::ApproveTaskUseCase::new(
        task_repo.clone(),
        inst_repo.clone(),
        def_repo.clone(),
    ));
    let reject_task_uc = Arc::new(usecase::RejectTaskUseCase::new(
        task_repo.clone(),
        inst_repo.clone(),
        def_repo.clone(),
    ));
    let reassign_task_uc = Arc::new(usecase::ReassignTaskUseCase::new(task_repo.clone()));
    let _check_overdue_uc = Arc::new(usecase::CheckOverdueTasksUseCase::new(task_repo));

    let grpc_svc = Arc::new(WorkflowGrpcService::new(
        start_inst_uc.clone(),
        get_inst_uc.clone(),
        approve_task_uc.clone(),
        reject_task_uc.clone(),
    ));

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-workflow-server",
    ));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = if let Some(ref auth_cfg) = cfg.auth {
        info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for workflow-server");
        let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
            &auth_cfg.jwks_url,
            &auth_cfg.issuer,
            &auth_cfg.audience,
            std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
        ));
        Some(adapter::middleware::auth::WorkflowAuthState {
            verifier: jwks_verifier,
        })
    } else {
        info!("no auth configured, workflow-server running without authentication");
        None
    };

    let mut handler_state = adapter::handler::AppState {
        create_workflow_uc: create_wf_uc,
        update_workflow_uc: update_wf_uc,
        delete_workflow_uc: delete_wf_uc,
        get_workflow_uc: get_wf_uc,
        list_workflows_uc: list_wf_uc,
        start_instance_uc: start_inst_uc,
        get_instance_uc: get_inst_uc,
        list_instances_uc: list_inst_uc,
        cancel_instance_uc: cancel_inst_uc,
        list_tasks_uc,
        approve_task_uc,
        reject_task_uc,
        reassign_task_uc,
        metrics: metrics.clone(),
        auth_state: None,
    };
    if let Some(auth_st) = auth_state {
        handler_state = handler_state.with_auth(auth_st);
    }

    let app = adapter::handler::router(handler_state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    // gRPC service
    use proto::k1s0::system::workflow::v1::workflow_service_server::WorkflowServiceServer;

    let workflow_tonic = adapter::grpc::WorkflowServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = "0.0.0.0:9090".parse()?;
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(WorkflowServiceServer::new(workflow_tonic))
            .serve(grpc_addr)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

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

struct InMemoryWorkflowDefinitionRepository {
    definitions: tokio::sync::RwLock<HashMap<String, WorkflowDefinition>>,
}

impl InMemoryWorkflowDefinitionRepository {
    fn new() -> Self {
        Self {
            definitions: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl WorkflowDefinitionRepository for InMemoryWorkflowDefinitionRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<WorkflowDefinition>> {
        let defs = self.definitions.read().await;
        Ok(defs.get(id).cloned())
    }

    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<WorkflowDefinition>> {
        let defs = self.definitions.read().await;
        Ok(defs.values().find(|d| d.name == name).cloned())
    }

    async fn find_all(
        &self,
        enabled_only: bool,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowDefinition>, u64)> {
        let defs = self.definitions.read().await;
        let results: Vec<_> = if enabled_only {
            defs.values().filter(|d| d.enabled).cloned().collect()
        } else {
            defs.values().cloned().collect()
        };
        let total = results.len() as u64;
        Ok((results, total))
    }

    async fn create(&self, definition: &WorkflowDefinition) -> anyhow::Result<()> {
        let mut defs = self.definitions.write().await;
        defs.insert(definition.id.clone(), definition.clone());
        Ok(())
    }

    async fn update(&self, definition: &WorkflowDefinition) -> anyhow::Result<()> {
        let mut defs = self.definitions.write().await;
        defs.insert(definition.id.clone(), definition.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        let mut defs = self.definitions.write().await;
        Ok(defs.remove(id).is_some())
    }
}

struct InMemoryWorkflowInstanceRepository {
    instances: tokio::sync::RwLock<HashMap<String, WorkflowInstance>>,
}

impl InMemoryWorkflowInstanceRepository {
    fn new() -> Self {
        Self {
            instances: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl WorkflowInstanceRepository for InMemoryWorkflowInstanceRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<WorkflowInstance>> {
        let instances = self.instances.read().await;
        Ok(instances.get(id).cloned())
    }

    async fn find_all(
        &self,
        status: Option<String>,
        workflow_id: Option<String>,
        initiator_id: Option<String>,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowInstance>, u64)> {
        let instances = self.instances.read().await;
        let results: Vec<_> = instances
            .values()
            .filter(|i| {
                status.as_deref().map_or(true, |s| i.status == s)
                    && workflow_id.as_deref().map_or(true, |w| i.workflow_id == w)
                    && initiator_id.as_deref().map_or(true, |init| i.initiator_id == init)
            })
            .cloned()
            .collect();
        let total = results.len() as u64;
        Ok((results, total))
    }

    async fn create(&self, instance: &WorkflowInstance) -> anyhow::Result<()> {
        let mut instances = self.instances.write().await;
        instances.insert(instance.id.clone(), instance.clone());
        Ok(())
    }

    async fn update(&self, instance: &WorkflowInstance) -> anyhow::Result<()> {
        let mut instances = self.instances.write().await;
        instances.insert(instance.id.clone(), instance.clone());
        Ok(())
    }
}

struct InMemoryWorkflowTaskRepository {
    tasks: tokio::sync::RwLock<HashMap<String, WorkflowTask>>,
}

impl InMemoryWorkflowTaskRepository {
    fn new() -> Self {
        Self {
            tasks: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl WorkflowTaskRepository for InMemoryWorkflowTaskRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<WorkflowTask>> {
        let tasks = self.tasks.read().await;
        Ok(tasks.get(id).cloned())
    }

    async fn find_all(
        &self,
        assignee_id: Option<String>,
        status: Option<String>,
        instance_id: Option<String>,
        overdue_only: bool,
        _page: u32,
        _page_size: u32,
    ) -> anyhow::Result<(Vec<WorkflowTask>, u64)> {
        let tasks = self.tasks.read().await;
        let results: Vec<_> = tasks
            .values()
            .filter(|t| {
                assignee_id.as_deref().map_or(true, |a| t.assignee_id.as_deref() == Some(a))
                    && status.as_deref().map_or(true, |s| t.status == s)
                    && instance_id.as_deref().map_or(true, |i| t.instance_id == i)
                    && (!overdue_only || t.is_overdue())
            })
            .cloned()
            .collect();
        let total = results.len() as u64;
        Ok((results, total))
    }

    async fn find_overdue(&self) -> anyhow::Result<Vec<WorkflowTask>> {
        let tasks = self.tasks.read().await;
        Ok(tasks.values().filter(|t| t.is_overdue()).cloned().collect())
    }

    async fn create(&self, task: &WorkflowTask) -> anyhow::Result<()> {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id.clone(), task.clone());
        Ok(())
    }

    async fn update(&self, task: &WorkflowTask) -> anyhow::Result<()> {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id.clone(), task.clone());
        Ok(())
    }
}
