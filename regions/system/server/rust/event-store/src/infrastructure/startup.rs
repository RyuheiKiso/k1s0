use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use crate::adapter;
use crate::adapter::grpc::EventStoreGrpcService;
use crate::adapter::handler::{self, AppState};
use crate::domain::repository::{EventRepository, EventStreamRepository, SnapshotRepository};
use super::config::Config;
use super::in_memory::{
    InMemoryEventRepository, InMemoryEventStreamRepository, InMemorySnapshotRepository,
};
use super::kafka::EventPublisher;
use super::persistence::{
    EventPostgresRepository, SnapshotPostgresRepository, StreamPostgresRepository,
};
use crate::usecase;

pub async fn run() -> anyhow::Result<()> {
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-event-store-server".to_string(),
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
        "starting event-store server"
    );

    // Database pool (optional)
    let db_pool = if let Some(ref db_config) = cfg.database {
        let _url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_config.url.clone());
        info!("connecting to database");
        let pool = super::database::connect(db_config).await?;
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

    // Repositories
    let stream_repo: Arc<dyn EventStreamRepository> = if let Some(ref pool) = db_pool {
        Arc::new(StreamPostgresRepository::new(pool.clone()))
    } else {
        Arc::new(InMemoryEventStreamRepository::new())
    };

    let event_repo: Arc<dyn EventRepository> = if let Some(ref pool) = db_pool {
        Arc::new(EventPostgresRepository::new(pool.clone()))
    } else {
        Arc::new(InMemoryEventRepository::new())
    };

    let snapshot_repo: Arc<dyn SnapshotRepository> = if let Some(ref pool) = db_pool {
        Arc::new(SnapshotPostgresRepository::new(pool.clone()))
    } else {
        Arc::new(InMemorySnapshotRepository::new())
    };

    // Kafka producer (optional)
    let event_publisher: Arc<dyn EventPublisher> = if let Some(ref kafka_config) = cfg.kafka {
        match super::kafka::EventStoreKafkaProducer::new(kafka_config) {
            Ok(producer) => {
                info!("kafka producer initialized");
                Arc::new(producer)
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to create kafka producer, using noop publisher");
                Arc::new(super::kafka::NoopEventPublisher)
            }
        }
    } else {
        info!("no kafka configured, using noop publisher");
        Arc::new(super::kafka::NoopEventPublisher)
    };

    // Use cases
    let append_events_uc = Arc::new(usecase::AppendEventsUseCase::new(
        stream_repo.clone(),
        event_repo.clone(),
    ));
    let read_events_uc = Arc::new(usecase::ReadEventsUseCase::new(
        stream_repo.clone(),
        event_repo.clone(),
    ));
    let read_event_by_sequence_uc = Arc::new(usecase::ReadEventBySequenceUseCase::new(
        stream_repo.clone(),
        event_repo.clone(),
    ));
    let create_snapshot_uc = Arc::new(usecase::CreateSnapshotUseCase::new(
        stream_repo.clone(),
        snapshot_repo.clone(),
    ));
    let get_latest_snapshot_uc = Arc::new(usecase::GetLatestSnapshotUseCase::new(
        stream_repo.clone(),
        snapshot_repo.clone(),
    ));
    let delete_stream_uc = Arc::new(usecase::DeleteStreamUseCase::new(
        stream_repo.clone(),
        event_repo.clone(),
        snapshot_repo.clone(),
    ));

    // gRPC service
    let grpc_svc = Arc::new(EventStoreGrpcService::new(
        append_events_uc.clone(),
        read_events_uc.clone(),
        read_event_by_sequence_uc.clone(),
        create_snapshot_uc.clone(),
        get_latest_snapshot_uc.clone(),
        delete_stream_uc.clone(),
        stream_repo.clone(),
    ));

    let grpc_addr: std::net::SocketAddr =
        format!("{}:{}", cfg.server.host, cfg.server.grpc_port).parse()?;
    info!("gRPC server starting on {}", grpc_addr);

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-event-store-server",
    ));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = k1s0_server_common::require_auth_state(
        "event-store",
        &cfg.app.environment,
        cfg.auth.as_ref().map(|auth_cfg| {
            info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for event-store");
            let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
                &auth_cfg.jwks_url,
                &auth_cfg.issuer,
                &auth_cfg.audience,
                std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
            ));
            crate::adapter::middleware::auth::EventStoreAuthState {
                verifier: jwks_verifier,
            }
        }),
    )?;
    let grpc_auth_state = auth_state
        .as_ref()
        .map(|s| crate::adapter::grpc::EventStoreGrpcAuthState {
            verifier: s.verifier.clone(),
        });

    // List use cases
    let list_events_uc = Arc::new(usecase::ListEventsUseCase::new(event_repo.clone()));
    let list_streams_uc = Arc::new(usecase::ListStreamsUseCase::new(stream_repo.clone()));

    // REST AppState
    let mut state = AppState {
        append_events_uc,
        read_events_uc,
        read_event_by_sequence_uc,
        list_events_uc,
        list_streams_uc,
        create_snapshot_uc,
        get_latest_snapshot_uc,
        delete_stream_uc,
        stream_repo,
        event_publisher,
        metrics: metrics.clone(),
        auth_state: None,
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    // tonic wrapper
    use crate::proto::k1s0::system::eventstore::v1::event_store_service_server::EventStoreServiceServer;
    let event_store_tonic = crate::adapter::grpc::EventStoreServiceTonic::new(grpc_svc, grpc_auth_state);

    // Router
    let app = handler::router(state).layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    // gRPC server
    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(EventStoreServiceServer::new(event_store_tonic))
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
