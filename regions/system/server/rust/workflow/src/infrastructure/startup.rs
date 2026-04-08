use anyhow::Context;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use super::config::Config;
use super::in_memory::{
    InMemoryWorkflowDefinitionRepository, InMemoryWorkflowInstanceRepository,
    InMemoryWorkflowTaskRepository,
};
use super::kafka_producer::{
    KafkaWorkflowEventPublisher, NoopWorkflowEventPublisher, WorkflowEventPublisher,
};
use super::notification_request_producer::{
    KafkaNotificationRequestPublisher, NoopNotificationRequestPublisher,
    NotificationRequestPublisher,
};
use super::scheduler_registration::register_overdue_check_job;
use crate::adapter::grpc::WorkflowGrpcService;
use crate::domain::repository::WorkflowDefinitionRepository;
use crate::domain::repository::WorkflowInstanceRepository;
use crate::domain::repository::WorkflowTaskRepository;

async fn resolve_bind_addr(host: &str, port: u16) -> anyhow::Result<SocketAddr> {
    tokio::net::lookup_host((host, port))
        .await?
        .next()
        .ok_or_else(|| anyhow::anyhow!("failed to resolve bind address {host}:{port}"))
}

// HIGH-001 監査対応: 起動処理は構造上行数が多くなるため許容する
#[allow(clippy::too_many_lines, clippy::items_after_statements)]
pub async fn run() -> anyhow::Result<()> {
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-workflow-server".to_string(),
        // Cargo.toml の package.version を使用する（M-16 監査対応: ハードコード解消）
        version: env!("CARGO_PKG_VERSION").to_string(),
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
    k1s0_telemetry::init_telemetry(&telemetry_cfg)
        .map_err(|e| anyhow::anyhow!("テレメトリの初期化に失敗: {e}"))?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting workflow server"
    );

    // --- Repository / Event Publisher ---
    #[allow(clippy::type_complexity)]
    let (def_repo, inst_repo, task_repo, workflow_pool): (
        Arc<dyn WorkflowDefinitionRepository>,
        Arc<dyn WorkflowInstanceRepository>,
        Arc<dyn WorkflowTaskRepository>,
        Option<Arc<sqlx::PgPool>>,
    ) = if let Some(ref db_cfg) = cfg.database {
        // DATABASE_URL 環境変数が設定されている場合は優先する（serde_yaml はシェル変数を展開しないため）
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_cfg.connection_url());
        let pool = Arc::new(
            super::database::create_pool(
                &db_url,
                db_cfg.max_open_conns,
                db_cfg.max_idle_conns,
                &db_cfg.conn_max_lifetime,
            )
            .await?,
        );
        info!("connected to PostgreSQL database");
        // 起動時に workflow-db マイグレーションを適用する（C-01 対応）
        crate::MIGRATOR
            .run(pool.as_ref())
            .await
            .map_err(|e| anyhow::anyhow!("workflow-db migration failed: {e}"))?;
        tracing::info!("workflow-db migrations applied successfully");

        (
            Arc::new(crate::adapter::repository::DefinitionPostgresRepository::new(pool.clone())),
            Arc::new(crate::adapter::repository::InstancePostgresRepository::new(
                pool.clone(),
            )),
            Arc::new(crate::adapter::repository::TaskPostgresRepository::new(
                pool.clone(),
            )),
            Some(pool),
        )
    } else {
        // infra_guard: stable サービスでは DB 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "workflow",
            k1s0_server_common::InfraKind::Database,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no database config found, using in-memory repositories (dev/test bypass)");
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
        // infra_guard: stable サービスでは Kafka 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "workflow",
            k1s0_server_common::InfraKind::Kafka,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no Kafka config found, using noop event publisher (dev/test bypass)");
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

    // db_pool_for_health は /healthz エンドポイントの DB 接続確認に使用する（C-02 対応）
    // workflow_pool は後段で所有権が移動するため、ここで先に PgPool を clone しておく
    let db_pool_for_health = workflow_pool.as_ref().map(|p| p.as_ref().clone());

    let create_wf_uc = Arc::new(crate::usecase::CreateWorkflowUseCase::new(def_repo.clone()));
    let update_wf_uc = Arc::new(crate::usecase::UpdateWorkflowUseCase::new(def_repo.clone()));
    let delete_wf_uc = Arc::new(crate::usecase::DeleteWorkflowUseCase::new(def_repo.clone()));
    let get_wf_uc = Arc::new(crate::usecase::GetWorkflowUseCase::new(def_repo.clone()));
    let list_wf_uc = Arc::new(crate::usecase::ListWorkflowsUseCase::new(def_repo.clone()));
    let start_inst_uc = Arc::new(if let Some(pool) = workflow_pool.clone() {
        crate::usecase::StartInstanceUseCase::with_pool(
            def_repo.clone(),
            inst_repo.clone(),
            task_repo.clone(),
            event_publisher.clone(),
            pool,
        )
    } else {
        crate::usecase::StartInstanceUseCase::new(
            def_repo.clone(),
            inst_repo.clone(),
            task_repo.clone(),
            event_publisher.clone(),
        )
    });
    let get_inst_uc = Arc::new(crate::usecase::GetInstanceUseCase::new(inst_repo.clone()));
    let list_inst_uc = Arc::new(crate::usecase::ListInstancesUseCase::new(inst_repo.clone()));
    let cancel_inst_uc = Arc::new(crate::usecase::CancelInstanceUseCase::new(
        inst_repo.clone(),
    ));
    let list_tasks_uc = Arc::new(crate::usecase::ListTasksUseCase::new(task_repo.clone()));
    let approve_task_uc = Arc::new(if let Some(pool) = workflow_pool.clone() {
        crate::usecase::ApproveTaskUseCase::with_pool(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo.clone(),
            event_publisher.clone(),
            pool,
        )
    } else {
        crate::usecase::ApproveTaskUseCase::new(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo.clone(),
            event_publisher.clone(),
        )
    });
    let reject_task_uc = Arc::new(if let Some(pool) = workflow_pool {
        crate::usecase::RejectTaskUseCase::with_pool(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo.clone(),
            event_publisher.clone(),
            pool,
        )
    } else {
        crate::usecase::RejectTaskUseCase::new(
            task_repo.clone(),
            inst_repo.clone(),
            def_repo.clone(),
            event_publisher.clone(),
        )
    });
    let reassign_task_uc = Arc::new(crate::usecase::ReassignTaskUseCase::new(task_repo.clone()));
    let check_overdue_uc = Arc::new(crate::usecase::CheckOverdueTasksUseCase::new(
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
        cfg.auth
            .as_ref()
            .map(|auth_cfg| -> anyhow::Result<_> {
                // nested 形式の AuthConfig から JWKS URL を取得する
                let jwks_url = auth_cfg
                    .jwks
                    .as_ref()
                    .map(|j| j.url.as_str())
                    .unwrap_or_default();
                let cache_ttl = auth_cfg
                    .jwks
                    .as_ref()
                    .map_or(300, |j| j.cache_ttl_secs);
                info!(jwks_url = %jwks_url, "initializing JWKS verifier for workflow-server");
                let jwks_verifier = Arc::new(
                    k1s0_auth::JwksVerifier::new(
                        jwks_url,
                        &auth_cfg.jwt.issuer,
                        &auth_cfg.jwt.audience,
                        std::time::Duration::from_secs(cache_ttl),
                    )
                    .context("JWKS 検証器の作成に失敗")?,
                );
                Ok(crate::adapter::middleware::auth::AuthState {
                    verifier: jwks_verifier,
                })
            })
            .transpose()?,
    )?;

    let mut handler_state = crate::adapter::handler::AppState {
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
        db_pool: db_pool_for_health,
    };
    if let Some(auth_st) = auth_state {
        handler_state = handler_state.with_auth(auth_st);
    }
    let grpc_auth_state = handler_state.auth_state.clone();

    // CorrelationLayerを追加してリクエスト間の相関IDを伝播する
    let app = crate::adapter::handler::router(
        handler_state,
        cfg.observability.metrics.enabled,
        &cfg.observability.metrics.path,
    )
    .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
    .layer(k1s0_correlation::layer::CorrelationLayer::new());

    let rest_addr = resolve_bind_addr(&cfg.server.host, cfg.server.port).await?;
    info!("REST server starting on {}", rest_addr);

    // gRPC service
    use crate::proto::k1s0::system::workflow::v1::workflow_service_server::WorkflowServiceServer;

    let workflow_tonic = crate::adapter::grpc::WorkflowServiceTonic::new(grpc_svc);

    let grpc_addr = resolve_bind_addr(&cfg.server.host, cfg.server.grpc_port).await?;
    info!("gRPC server starting on {}", grpc_addr);

    // gRPC Health Check Protocol サービスを登録する。
    // readyz エンドポイントや Kubernetes の livenessProbe/readinessProbe が
    // Bearer token なしでヘルスチェックできるようにするため。
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<WorkflowServiceServer<crate::adapter::grpc::WorkflowServiceTonic>>()
        .await;

    let grpc_metrics = metrics;
    let grpc_auth_layer =
        crate::adapter::middleware::grpc_auth::GrpcAuthLayer::new(grpc_auth_state);
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(health_service)
            .add_service(WorkflowServiceServer::new(workflow_tonic))
            .serve_with_shutdown(grpc_addr, async move {
                let _ = grpc_shutdown.await;
            })
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {e}"))
    };

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
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

    // Kafkaパブリッシャーをクローズしてからテレメトリをシャットダウンする
    event_publisher.close().await?;
    notification_request_publisher.close().await?;

    // テレメトリのエクスポーターをフラッシュしてシャットダウンする
    k1s0_telemetry::shutdown();

    Ok(())
}
