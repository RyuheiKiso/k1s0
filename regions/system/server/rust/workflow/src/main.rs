use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod infrastructure;
mod proto;
mod usecase;

use adapter::grpc::WorkflowGrpcService;
use domain::repository::WorkflowDefinitionRepository;
use domain::repository::WorkflowInstanceRepository;
use domain::repository::WorkflowTaskRepository;
use infrastructure::config::Config;
use infrastructure::in_memory::{
    InMemoryWorkflowDefinitionRepository, InMemoryWorkflowInstanceRepository,
    InMemoryWorkflowTaskRepository,
};
use infrastructure::kafka_producer::{
    KafkaWorkflowEventPublisher, NoopWorkflowEventPublisher, WorkflowEventPublisher,
};
use infrastructure::notification_request_producer::{
    KafkaNotificationRequestPublisher, NoopNotificationRequestPublisher,
    NotificationRequestPublisher,
};
use infrastructure::scheduler_registration::register_overdue_check_job;

async fn resolve_bind_addr(host: &str, port: u16) -> anyhow::Result<SocketAddr> {
    tokio::net::lookup_host((host, port))
        .await?
        .next()
        .ok_or_else(|| anyhow::anyhow!("failed to resolve bind address {host}:{port}"))
}

async fn shutdown_signal() -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut terminate = signal(SignalKind::terminate())?;
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = terminate.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-workflow-server".to_string(),
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
        "starting workflow server"
    );

    // --- Repository / Event Publisher 蛻晄悄蛹・---
    // database 險ｭ螳壹′縺ゅｋ蝣ｴ蜷医・ PostgreSQL縲√↑縺代ｌ縺ｰ InMemory 繝輔か繝ｼ繝ｫ繝舌ャ繧ｯ
    let (def_repo, inst_repo, task_repo, workflow_pool): (
        Arc<dyn WorkflowDefinitionRepository>,
        Arc<dyn WorkflowInstanceRepository>,
        Arc<dyn WorkflowTaskRepository>,
        Option<Arc<sqlx::PgPool>>,
    ) = if let Some(ref db_cfg) = cfg.database {
        let pool = Arc::new(
            infrastructure::database::create_pool(
                &db_cfg.connection_url(),
                db_cfg.max_open_conns,
                db_cfg.max_idle_conns,
                &db_cfg.conn_max_lifetime,
            )
            .await?,
        );
        info!("connected to PostgreSQL database");

        (
            Arc::new(adapter::repository::DefinitionPostgresRepository::new(
                pool.clone(),
            )),
            Arc::new(adapter::repository::InstancePostgresRepository::new(
                pool.clone(),
            )),
            Arc::new(adapter::repository::TaskPostgresRepository::new(
                pool.clone(),
            )),
            Some(pool),
        )
    } else {
        info!("no database config found, using in-memory repositories");
        (
            Arc::new(InMemoryWorkflowDefinitionRepository::new()),
            Arc::new(InMemoryWorkflowInstanceRepository::new()),
            Arc::new(InMemoryWorkflowTaskRepository::new()),
            None,
        )
    };

    // Kafka event publisher
    let event_publisher: Arc<dyn WorkflowEventPublisher> = if let Some(ref kafka_cfg) = cfg.kafka {
        let publisher = KafkaWorkflowEventPublisher::new(kafka_cfg)?;
        info!(topic = %publisher.topic(), "Kafka event publisher initialized");
        Arc::new(publisher)
    } else {
        info!("no Kafka config found, using noop event publisher");
        Arc::new(NoopWorkflowEventPublisher)
    };

    let notification_request_publisher: Arc<dyn NotificationRequestPublisher> =
        if let Some(ref kafka_cfg) = cfg.kafka {
            let publisher = KafkaNotificationRequestPublisher::new(kafka_cfg)?;
            info!(
                topic = %kafka_cfg.notification_topic,
                "Kafka notification request publisher initialized"
            );
            Arc::new(publisher)
        } else {
            Arc::new(NoopNotificationRequestPublisher)
        };

    if cfg.scheduler.is_some() {
        register_overdue_check_job(&cfg).await?;
    }

    let create_wf_uc = Arc::new(usecase::CreateWorkflowUseCase::new(def_repo.clone()));
    let update_wf_uc = Arc::new(usecase::UpdateWorkflowUseCase::new(def_repo.clone()));
    let delete_wf_uc = Arc::new(usecase::DeleteWorkflowUseCase::new(def_repo.clone()));
    let get_wf_uc = Arc::new(usecase::GetWorkflowUseCase::new(def_repo.clone()));
    let list_wf_uc = Arc::new(usecase::ListWorkflowsUseCase::new(def_repo.clone()));
    let start_inst_uc = Arc::new(if let Some(pool) = workflow_pool.clone() {
        usecase::StartInstanceUseCase::with_pool(
            def_repo.clone(),
            inst_repo.clone(),
            task_repo.clone(),
            event_publisher.clone(),
            pool,
        )
    } else {
        usecase::StartInstanceUseCase::new(
            def_repo.clone(),
            inst_repo.clone(),
            task_repo.clone(),
            event_publisher.clone(),
        )
    });
    let get_inst_uc = Arc::new(usecase::GetInstanceUseCase::new(inst_repo.clone()));
    let list_inst_uc = Arc::new(usecase::ListInstancesUseCase::new(inst_repo.clone()));
    let cancel_inst_uc = Arc::new(usecase::CancelInstanceUseCase::new(inst_repo.clone()));
    let list_tasks_uc = Arc::new(usecase::ListTasksUseCase::new(task_repo.clone()));
    let approve_task_uc = Arc::new(if let Some(pool) = workflow_pool.clone() {
        usecase::ApproveTaskUseCase::with_pool(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo.clone(),
            event_publisher.clone(),
            pool,
        )
    } else {
        usecase::ApproveTaskUseCase::new(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo.clone(),
            event_publisher.clone(),
        )
    });
    let reject_task_uc = Arc::new(if let Some(pool) = workflow_pool {
        usecase::RejectTaskUseCase::with_pool(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo.clone(),
            event_publisher.clone(),
            pool,
        )
    } else {
        usecase::RejectTaskUseCase::new(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo.clone(),
            event_publisher.clone(),
        )
    });
    let reassign_task_uc = Arc::new(usecase::ReassignTaskUseCase::new(task_repo.clone()));
    let check_overdue_uc = Arc::new(usecase::CheckOverdueTasksUseCase::new(
        task_repo.clone(),
        notification_request_publisher.clone(),
    ));

    let grpc_svc = Arc::new(WorkflowGrpcService::new(
        list_wf_uc.clone(),
        create_wf_uc.clone(),
        get_wf_uc.clone(),
        update_wf_uc.clone(),
        delete_wf_uc.clone(),
        start_inst_uc.clone(),
        get_inst_uc.clone(),
        list_inst_uc.clone(),
        cancel_inst_uc.clone(),
        list_tasks_uc.clone(),
        reassign_task_uc.clone(),
        approve_task_uc.clone(),
        reject_task_uc.clone(),
    ));

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-workflow-server",
    ));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = k1s0_server_common::require_auth_state(
        "workflow-server",
        &cfg.app.environment,
        cfg.auth.as_ref().map(|auth_cfg| {
            info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for workflow-server");
            let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
                &auth_cfg.jwks_url,
                &auth_cfg.issuer,
                &auth_cfg.audience,
                std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
            ));
            adapter::middleware::auth::WorkflowAuthState {
                verifier: jwks_verifier,
            }
        }),
    )?;

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
        check_overdue_tasks_uc: check_overdue_uc,
        metrics: metrics.clone(),
        auth_state: None,
    };
    if let Some(auth_st) = auth_state {
        handler_state = handler_state.with_auth(auth_st);
    }
    let grpc_auth_state = handler_state.auth_state.clone();

    let app = adapter::handler::router(
        handler_state,
        cfg.observability.metrics.enabled,
        &cfg.observability.metrics.path,
    )
    .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    let rest_addr = resolve_bind_addr(&cfg.server.host, cfg.server.port).await?;
    info!("REST server starting on {}", rest_addr);

    // gRPC service
    use proto::k1s0::system::workflow::v1::workflow_service_server::WorkflowServiceServer;

    let workflow_tonic = adapter::grpc::WorkflowServiceTonic::new(grpc_svc);

    let grpc_addr = resolve_bind_addr(&cfg.server.host, cfg.server.grpc_port).await?;
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_auth_layer = adapter::middleware::grpc_auth::GrpcAuthLayer::new(grpc_auth_state);
    let grpc_shutdown = shutdown_signal();
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(WorkflowServiceServer::new(workflow_tonic))
            .serve_with_shutdown(grpc_addr, async move {
                let _ = grpc_shutdown.await;
            })
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    let rest_future = axum::serve(listener, app).with_graceful_shutdown(async {
        let _ = shutdown_signal().await;
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

    event_publisher.close().await?;
    notification_request_publisher.close().await?;

    Ok(())
}
