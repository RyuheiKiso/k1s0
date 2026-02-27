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

use adapter::grpc::EventStoreGrpcService;
use adapter::handler::{self, AppState};
use domain::entity::event::{EventStream, Snapshot, StoredEvent};
use domain::repository::{EventRepository, EventStreamRepository, SnapshotRepository};
use infrastructure::config::Config;
use infrastructure::kafka::EventPublisher;
use infrastructure::persistence::{
    EventPostgresRepository, SnapshotPostgresRepository, StreamPostgresRepository,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-event-store-server".to_string(),
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
        "starting event-store server"
    );

    // Database pool (optional)
    let db_pool = if let Some(ref db_config) = cfg.database {
        let _url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_config.url.clone());
        info!("connecting to database");
        let pool = infrastructure::database::connect(db_config).await?;
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
        match infrastructure::kafka::EventStoreKafkaProducer::new(kafka_config) {
            Ok(producer) => {
                info!("kafka producer initialized");
                Arc::new(producer)
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to create kafka producer, using noop publisher");
                Arc::new(infrastructure::kafka::NoopEventPublisher)
            }
        }
    } else {
        info!("no kafka configured, using noop publisher");
        Arc::new(infrastructure::kafka::NoopEventPublisher)
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
        read_event_by_sequence_uc,
        create_snapshot_uc.clone(),
        get_latest_snapshot_uc.clone(),
    ));

    let grpc_addr: std::net::SocketAddr =
        format!("{}:{}", cfg.server.host, cfg.server.grpc_port).parse()?;
    info!("gRPC server starting on {}", grpc_addr);

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-event-store-server",
    ));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = if let Some(ref auth_cfg) = cfg.auth {
        info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for event-store");
        let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
            &auth_cfg.jwks_url,
            &auth_cfg.issuer,
            &auth_cfg.audience,
            std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
        ));
        Some(adapter::middleware::auth::EventStoreAuthState {
            verifier: jwks_verifier,
        })
    } else {
        info!("no auth configured, event-store running without authentication");
        None
    };

    // REST AppState
    let mut state = AppState {
        append_events_uc,
        read_events_uc,
        create_snapshot_uc,
        get_latest_snapshot_uc,
        delete_stream_uc,
        stream_repo,
        event_repo,
        event_publisher,
        metrics: metrics.clone(),
        auth_state: None,
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    // tonic wrapper
    use proto::k1s0::system::eventstore::v1::event_store_service_server::EventStoreServiceServer;
    let event_store_tonic = adapter::grpc::EventStoreServiceTonic::new(grpc_svc);

    // Router
    let app = handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

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

// --- InMemory Repositories ---

struct InMemoryEventStreamRepository {
    streams: tokio::sync::RwLock<HashMap<String, EventStream>>,
}

impl InMemoryEventStreamRepository {
    fn new() -> Self {
        Self {
            streams: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl EventStreamRepository for InMemoryEventStreamRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<EventStream>> {
        let streams = self.streams.read().await;
        Ok(streams.get(id).cloned())
    }

    async fn list_all(&self, page: u32, page_size: u32) -> anyhow::Result<(Vec<EventStream>, u64)> {
        let streams = self.streams.read().await;
        let all: Vec<EventStream> = streams.values().cloned().collect();
        let total = all.len() as u64;
        let page = page.max(1);
        let page_size = page_size.max(1).min(200);
        let offset = ((page - 1) * page_size) as usize;
        let paged: Vec<EventStream> = all.into_iter().skip(offset).take(page_size as usize).collect();
        Ok((paged, total))
    }

    async fn create(&self, stream: &EventStream) -> anyhow::Result<()> {
        let mut streams = self.streams.write().await;
        streams.insert(stream.id.clone(), stream.clone());
        Ok(())
    }

    async fn update_version(&self, id: &str, new_version: i64) -> anyhow::Result<()> {
        let mut streams = self.streams.write().await;
        if let Some(stream) = streams.get_mut(id) {
            stream.current_version = new_version;
            stream.updated_at = chrono::Utc::now();
        }
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        let mut streams = self.streams.write().await;
        Ok(streams.remove(id).is_some())
    }
}

struct InMemoryEventRepository {
    events: tokio::sync::RwLock<Vec<StoredEvent>>,
    sequence_counter: tokio::sync::RwLock<u64>,
}

impl InMemoryEventRepository {
    fn new() -> Self {
        Self {
            events: tokio::sync::RwLock::new(Vec::new()),
            sequence_counter: tokio::sync::RwLock::new(0),
        }
    }
}

#[async_trait::async_trait]
impl EventRepository for InMemoryEventRepository {
    async fn append(
        &self,
        _stream_id: &str,
        events: Vec<StoredEvent>,
    ) -> anyhow::Result<Vec<StoredEvent>> {
        let mut all_events = self.events.write().await;
        let mut counter = self.sequence_counter.write().await;
        let mut result = Vec::new();
        for mut event in events {
            *counter += 1;
            event.sequence = *counter;
            result.push(event.clone());
            all_events.push(event);
        }
        Ok(result)
    }

    async fn find_by_stream(
        &self,
        stream_id: &str,
        from_version: i64,
        to_version: Option<i64>,
        event_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<StoredEvent>, u64)> {
        let all_events = self.events.read().await;
        let filtered: Vec<_> = all_events
            .iter()
            .filter(|e| {
                e.stream_id == stream_id
                    && e.version >= from_version
                    && to_version.map_or(true, |tv| e.version <= tv)
                    && event_type.as_ref().map_or(true, |et| e.event_type == *et)
            })
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        let offset = ((page - 1) * page_size) as usize;
        let paged: Vec<_> = filtered.into_iter().skip(offset).take(page_size as usize).collect();
        Ok((paged, total))
    }

    async fn find_all(
        &self,
        event_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<StoredEvent>, u64)> {
        let all_events = self.events.read().await;
        let filtered: Vec<_> = all_events
            .iter()
            .filter(|e| event_type.as_ref().map_or(true, |et| e.event_type == *et))
            .cloned()
            .collect();
        let total = filtered.len() as u64;
        let page = page.max(1);
        let page_size = page_size.max(1).min(200);
        let offset = ((page - 1) * page_size) as usize;
        let paged: Vec<_> = filtered.into_iter().skip(offset).take(page_size as usize).collect();
        Ok((paged, total))
    }

    async fn find_by_sequence(
        &self,
        stream_id: &str,
        sequence: u64,
    ) -> anyhow::Result<Option<StoredEvent>> {
        let all_events = self.events.read().await;
        Ok(all_events
            .iter()
            .find(|e| e.stream_id == stream_id && e.sequence == sequence)
            .cloned())
    }

    async fn delete_by_stream(&self, stream_id: &str) -> anyhow::Result<u64> {
        let mut all_events = self.events.write().await;
        let before = all_events.len();
        all_events.retain(|e| e.stream_id != stream_id);
        Ok((before - all_events.len()) as u64)
    }
}

struct InMemorySnapshotRepository {
    snapshots: tokio::sync::RwLock<Vec<Snapshot>>,
}

impl InMemorySnapshotRepository {
    fn new() -> Self {
        Self {
            snapshots: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl SnapshotRepository for InMemorySnapshotRepository {
    async fn create(&self, snapshot: &Snapshot) -> anyhow::Result<()> {
        let mut snapshots = self.snapshots.write().await;
        snapshots.push(snapshot.clone());
        Ok(())
    }

    async fn find_latest(&self, stream_id: &str) -> anyhow::Result<Option<Snapshot>> {
        let snapshots = self.snapshots.read().await;
        Ok(snapshots
            .iter()
            .filter(|s| s.stream_id == stream_id)
            .max_by_key(|s| s.snapshot_version)
            .cloned())
    }

    async fn delete_by_stream(&self, stream_id: &str) -> anyhow::Result<u64> {
        let mut snapshots = self.snapshots.write().await;
        let before = snapshots.len();
        snapshots.retain(|s| s.stream_id != stream_id);
        Ok((before - snapshots.len()) as u64)
    }
}
