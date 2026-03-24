// タスクサーバーの起動処理。
// DB 接続・マイグレーション・認証初期化・ユースケース構築・REST/gRPC サーバー起動を行う。
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;
use anyhow::Context;
use tonic::transport::Server;

use crate::adapter;
use crate::infrastructure;
use crate::usecase;
use crate::MIGRATOR;

use super::config::{Config, DatabaseConfig};
use crate::adapter::handler::{self, AppState};
use crate::adapter::middleware::auth::AuthState;
use k1s0_server_common::middleware::grpc_auth::GrpcAuthLayer;
use k1s0_server_common::middleware::rbac::Tier;
use k1s0_server_common::shutdown::shutdown_signal;
use k1s0_server_common::require_auth_state;

pub async fn run() -> anyhow::Result<()> {
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/default.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-task-server".to_string(),
        version: "0.1.0".to_string(),
        tier: "service".to_string(),
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
    match k1s0_telemetry::init_telemetry(&telemetry_cfg) {
        Ok(()) => {}
        Err(e) => tracing::warn!("telemetry init failed: {}", e),
    }

    info!("starting {}", cfg.app.name);

    let db_cfg = cfg
        .database
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("database configuration is required"))?;
    let db_pool = connect_database(db_cfg).await?;

    {
        let mut migration_conn = db_pool.acquire().await.context("advisory lock connection")?;
        // Advisory Lock ID: 1000000010 (task-service)
        // ID割り当て規則: docs/architecture/conventions/advisory-lock-ids.md 参照
        // 各サービスのID: task=1000000010, board=1000000011, activity=1000000012
        sqlx::query("SELECT pg_advisory_lock(1000000010)")
            .execute(&mut *migration_conn)
            .await
            .context("advisory lock acquire")?;
        let migrate_result = MIGRATOR.run(&db_pool).await;
        sqlx::query("SELECT pg_advisory_unlock(1000000010)")
            .execute(&mut *migration_conn)
            .await
            .context("advisory lock release")?;
        migrate_result.context("migration failed")?;
    }

    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("task"));

    let task_repo: Arc<dyn crate::domain::repository::task_repository::TaskRepository> = Arc::new(
        infrastructure::database::task_repository::TaskPostgresRepository::new(db_pool.clone()),
    );

    let create_task_uc = Arc::new(usecase::create_task::CreateTaskUseCase::new(task_repo.clone()));
    let get_task_uc = Arc::new(usecase::get_task::GetTaskUseCase::new(task_repo.clone()));
    let list_tasks_uc = Arc::new(usecase::list_tasks::ListTasksUseCase::new(task_repo.clone()));
    let update_task_status_uc = Arc::new(usecase::update_task_status::UpdateTaskStatusUseCase::new(task_repo.clone()));
    let update_task_uc = Arc::new(usecase::update_task::UpdateTaskUseCase::new(task_repo.clone()));
    let create_checklist_item_uc = Arc::new(usecase::create_checklist_item::CreateChecklistItemUseCase::new(task_repo.clone()));
    let update_checklist_item_uc = Arc::new(usecase::update_checklist_item::UpdateChecklistItemUseCase::new(task_repo.clone()));
    let delete_checklist_item_uc = Arc::new(usecase::delete_checklist_item::DeleteChecklistItemUseCase::new(task_repo.clone()));

    // Outbox ポーラー（Kafka 設定がある場合のみ起動）
    if let Some(ref kafka_cfg) = cfg.kafka {
        if let Ok(producer) = infrastructure::kafka::task_producer::TaskKafkaProducer::new(kafka_cfg) {
            let producer = Arc::new(producer);
            let poller = infrastructure::outbox_poller::OutboxPoller::new(db_pool.clone(), producer);
            tokio::spawn(poller.run());
        }
    }

    // 認証状態の初期化（auth 設定がある場合は JWKS 検証器を生成する）
    let auth_state_opt = cfg.auth
        .as_ref()
        .map(|auth_cfg| -> anyhow::Result<AuthState> {
            info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for task-server");
            let verifier = Arc::new(
                k1s0_auth::JwksVerifier::new(
                    &auth_cfg.jwks_url,
                    &auth_cfg.issuer,
                    &auth_cfg.audience,
                    std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
                )
                .context("JWKS 検証器の作成に失敗")?,
            );
            Ok(AuthState { verifier })
        })
        .transpose()?;

    // 認証設定が未指定の場合は dev/test 環境かつ ALLOW_INSECURE_NO_AUTH=true のみ許可する
    let auth_state = require_auth_state(
        "task-server",
        &cfg.app.environment,
        auth_state_opt,
    )?;

    let state = AppState {
        create_task_uc: create_task_uc.clone(),
        get_task_uc: get_task_uc.clone(),
        list_tasks_uc: list_tasks_uc.clone(),
        update_task_status_uc: update_task_status_uc.clone(),
        update_task_uc,
        create_checklist_item_uc,
        update_checklist_item_uc,
        delete_checklist_item_uc,
        metrics: metrics.clone(),
        auth_state: auth_state.clone(),
    };
    let app = handler::router(state);

    let grpc_service = adapter::grpc::task_grpc::TaskGrpcService::new(
        create_task_uc.clone(),
        get_task_uc.clone(),
        list_tasks_uc.clone(),
        update_task_status_uc.clone(),
    );
    let grpc_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.grpc_port).parse()?;
    // gRPC 認証レイヤーに auth_state を渡す（REST と同じ認証設定を共有する）
    let grpc_auth_layer = GrpcAuthLayer::new(auth_state, Tier::Service, required_action);

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let mut rest_shutdown_rx = shutdown_rx.clone();
    let mut grpc_shutdown_rx = shutdown_rx.clone();

    let grpc_future = async move {
        use crate::proto::k1s0::service::task::v1::task_service_server::TaskServiceServer;
        Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(metrics))
            .add_service(TaskServiceServer::new(grpc_service))
            .serve_with_shutdown(grpc_addr, async move {
                let _ = grpc_shutdown_rx.changed().await;
            })
            .await
            .map_err(|e| anyhow::anyhow!("gRPC error: {}", e))
    };

    let rest_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port).parse()?;
    info!("REST server listening on {}", rest_addr);
    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    let rest_future = axum::serve(listener, app).with_graceful_shutdown(async move {
        let _ = rest_shutdown_rx.changed().await;
    });

    let shutdown_future = async move {
        shutdown_signal().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        let _ = shutdown_tx.send(true);
        Ok::<(), anyhow::Error>(())
    };

    tokio::select! {
        result = shutdown_future => { result?; }
        result = rest_future => { if let Err(e) = result { return Err(anyhow::anyhow!("REST error: {}", e)); } }
        result = grpc_future => { result?; }
    }

    k1s0_telemetry::shutdown();
    Ok(())
}

fn required_action(method: &str) -> &'static str {
    match method {
        "GetTask" | "ListTasks" => "read",
        _ => "write",
    }
}

async fn connect_database(db_cfg: &DatabaseConfig) -> anyhow::Result<sqlx::PgPool> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_cfg.connection_url());
    let lifetime = Duration::from_secs(db_cfg.conn_max_lifetime);
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(db_cfg.max_connections)
        .min_connections(db_cfg.max_idle_conns.min(db_cfg.max_connections))
        .idle_timeout(Some(lifetime))
        .max_lifetime(Some(lifetime))
        .connect(&url)
        .await
        .map_err(|e| anyhow::anyhow!("database connection failed: {}", e))
}
