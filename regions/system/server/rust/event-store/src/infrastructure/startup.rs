use anyhow::Context;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use k1s0_server_common::middleware::grpc_auth::GrpcAuthLayer;
use k1s0_server_common::middleware::rbac::Tier;

use super::config::Config;
use super::in_memory::{
    InMemoryEventRepository, InMemoryEventStreamRepository, InMemorySnapshotRepository,
};
use super::kafka::EventPublisher;
use super::persistence::{
    EventPostgresRepository, SnapshotPostgresRepository, StreamPostgresRepository,
    TransactionalAppendAdapter,
};
use crate::adapter::grpc::EventStoreGrpcService;
use crate::adapter::handler::{self, AppState};
use crate::domain::repository::{EventRepository, EventStreamRepository, SnapshotRepository};
use crate::usecase;

pub async fn run() -> anyhow::Result<()> {
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-event-store-server".to_string(),
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
        .map_err(|e| anyhow::anyhow!("テレメトリの初期化に失敗: {}", e))?;

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
        // infra_guard: stable サービスでは DB 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "event-store",
            k1s0_server_common::InfraKind::Database,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no database configured, using in-memory repositories (dev/test bypass)");
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
        // infra_guard: stable サービスでは Kafka 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "event-store",
            k1s0_server_common::InfraKind::Kafka,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no kafka configured, using noop publisher (dev/test bypass)");
        Arc::new(super::kafka::NoopEventPublisher)
    };

    // Use cases
    // PgPool がある場合は TransactionalAppendAdapter（TransactionalAppendPort の実装）を
    // 経由して REPEATABLE READ トランザクションで原子性を保証する。
    // usecase 層は domain トレイト（TransactionalAppendPort）に依存し、
    // infrastructure 具体型（TransactionalAppendAdapter）には依存しない。
    let append_events_uc = Arc::new(if let Some(ref pool) = db_pool {
        let transactional_port = Arc::new(TransactionalAppendAdapter::new(pool.clone()));
        usecase::AppendEventsUseCase::new_with_transactional_port(
            stream_repo.clone(),
            event_repo.clone(),
            transactional_port,
        )
    } else {
        usecase::AppendEventsUseCase::new(stream_repo.clone(), event_repo.clone())
    });
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

    // gRPC service（event_publisher を渡して REST と同様に Kafka publish を行う）
    let grpc_svc = Arc::new(EventStoreGrpcService::new(
        append_events_uc.clone(),
        read_events_uc.clone(),
        read_event_by_sequence_uc.clone(),
        create_snapshot_uc.clone(),
        get_latest_snapshot_uc.clone(),
        delete_stream_uc.clone(),
        stream_repo.clone(),
        event_publisher.clone(),
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
        cfg.auth
            .as_ref()
            .map(|auth_cfg| -> anyhow::Result<_> {
                info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for event-store");
                let jwks_verifier = Arc::new(
                    k1s0_auth::JwksVerifier::new(
                        &auth_cfg.jwks_url,
                        &auth_cfg.issuer,
                        &auth_cfg.audience,
                        std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
                    )
                    .context("JWKS 検証器の作成に失敗")?,
                );
                Ok(crate::adapter::middleware::auth::AuthState {
                    verifier: jwks_verifier,
                })
            })
            .transpose()?,
    )?;
    // gRPC 認証レイヤー: メソッド名をアクション（read/write）にマッピングして RBAC チェックを行う
    let grpc_auth_layer =
        GrpcAuthLayer::new(auth_state.clone(), Tier::System, event_store_grpc_action);

    let grpc_auth_state =
        auth_state
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
    let event_store_tonic =
        crate::adapter::grpc::EventStoreServiceTonic::new(grpc_svc, grpc_auth_state);

    // Router (CorrelationLayer で相関IDを伝播)
    let app = handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

    // gRPC server
    // gRPC グレースフルシャットダウン用シグナル
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();
    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(EventStoreServiceServer::new(event_store_tonic))
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
    // REST グレースフルシャットダウン付きサーバー
    let rest_future = axum::serve(listener, app).with_graceful_shutdown(async {
        let _ = k1s0_server_common::shutdown::shutdown_signal().await;
    });

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

    // テレメトリのシャットダウン処理
    k1s0_telemetry::shutdown();

    Ok(())
}

/// gRPC メソッド名を RBAC アクション（read/write）にマッピングする。
/// イベントの追記（AppendEvent/AppendEvents）は write、それ以外は read とする。
fn event_store_grpc_action(method: &str) -> &'static str {
    match method {
        "AppendEvent" | "AppendEvents" => "write",
        _ => "read",
    }
}
