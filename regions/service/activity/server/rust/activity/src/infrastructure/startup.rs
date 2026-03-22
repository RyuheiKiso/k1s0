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
use k1s0_server_common::middleware::grpc_auth::GrpcAuthLayer;
use k1s0_server_common::middleware::rbac::Tier;
use k1s0_server_common::shutdown::shutdown_signal;

pub async fn run() -> anyhow::Result<()> {
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/default.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-activity-server".to_string(),
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
        sqlx::query("SELECT pg_advisory_lock(1000000012)")
            .execute(&mut *migration_conn)
            .await
            .context("advisory lock acquire")?;
        let migrate_result = MIGRATOR.run(&db_pool).await;
        sqlx::query("SELECT pg_advisory_unlock(1000000012)")
            .execute(&mut *migration_conn)
            .await
            .context("advisory lock release")?;
        migrate_result.context("migration failed")?;
    }

    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("activity"));

    let activity_repo: Arc<dyn crate::domain::repository::activity_repository::ActivityRepository> = Arc::new(
        infrastructure::database::activity_repository::ActivityPostgresRepository::new(db_pool.clone()),
    );

    let create_activity_uc = Arc::new(usecase::create_activity::CreateActivityUseCase::new(activity_repo.clone()));
    let get_activity_uc = Arc::new(usecase::get_activity::GetActivityUseCase::new(activity_repo.clone()));
    let list_activities_uc = Arc::new(usecase::list_activities::ListActivitiesUseCase::new(activity_repo.clone()));
    let submit_activity_uc = Arc::new(usecase::submit_activity::SubmitActivityUseCase::new(activity_repo.clone()));
    let approve_activity_uc = Arc::new(usecase::approve_activity::ApproveActivityUseCase::new(activity_repo.clone()));
    let reject_activity_uc = Arc::new(usecase::reject_activity::RejectActivityUseCase::new(activity_repo.clone()));

    if let Some(ref kafka_cfg) = cfg.kafka {
        if let Ok(producer) = infrastructure::kafka::activity_producer::ActivityKafkaProducer::new(kafka_cfg) {
            let producer = Arc::new(producer);
            let poller = infrastructure::outbox_poller::OutboxPoller::new(db_pool.clone(), producer);
            tokio::spawn(poller.run());
        }
    }

    let state = AppState {
        create_activity_uc: create_activity_uc.clone(),
        get_activity_uc: get_activity_uc.clone(),
        list_activities_uc: list_activities_uc.clone(),
        submit_activity_uc: submit_activity_uc.clone(),
        approve_activity_uc: approve_activity_uc.clone(),
        reject_activity_uc: reject_activity_uc.clone(),
        metrics: metrics.clone(),
    };
    let app = handler::router(state);

    let grpc_service = adapter::grpc::activity_grpc::ActivityGrpcService::new(
        create_activity_uc, get_activity_uc, list_activities_uc,
        submit_activity_uc, approve_activity_uc, reject_activity_uc,
    );

    let grpc_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.grpc_port).parse()?;
    let grpc_auth_layer = GrpcAuthLayer::new(None, Tier::Service, required_action);

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let mut rest_shutdown_rx = shutdown_rx.clone();
    let mut grpc_shutdown_rx = shutdown_rx.clone();

    let grpc_future = async move {
        use crate::proto::k1s0::service::activity::v1::activity_service_server::ActivityServiceServer;
        Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(metrics))
            .add_service(ActivityServiceServer::new(grpc_service))
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
        "GetActivity" | "ListActivities" => "read",
        "ApproveActivity" | "RejectActivity" => "admin",
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
