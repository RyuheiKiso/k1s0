#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod usecase;

use domain::entity::event::{EventStream, Snapshot, StoredEvent};
use domain::repository::{EventRepository, EventStreamRepository, SnapshotRepository};

#[derive(Debug, Clone, serde::Deserialize)]
struct Config {
    app: AppConfig,
    server: ServerConfig,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AppConfig {
    name: String,
    #[serde(default = "default_version")]
    version: String,
    #[serde(default = "default_environment")]
    environment: String,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_environment() -> String {
    "dev".to_string()
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ServerConfig {
    #[serde(default = "default_host")]
    host: String,
    #[serde(default = "default_port")]
    port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8099
}

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
    let config_content = std::fs::read_to_string(&config_path)?;
    let cfg: Config = serde_yaml::from_str(&config_content)?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting event-store server"
    );

    let stream_repo: Arc<dyn EventStreamRepository> =
        Arc::new(InMemoryEventStreamRepository::new());
    let event_repo: Arc<dyn EventRepository> = Arc::new(InMemoryEventRepository::new());
    let snapshot_repo: Arc<dyn SnapshotRepository> =
        Arc::new(InMemorySnapshotRepository::new());

    let _append_events_uc = Arc::new(usecase::AppendEventsUseCase::new(
        stream_repo.clone(),
        event_repo.clone(),
    ));
    let _read_events_uc = Arc::new(usecase::ReadEventsUseCase::new(
        stream_repo.clone(),
        event_repo.clone(),
    ));
    let _read_event_by_sequence_uc = Arc::new(usecase::ReadEventBySequenceUseCase::new(
        stream_repo.clone(),
        event_repo.clone(),
    ));
    let _create_snapshot_uc = Arc::new(usecase::CreateSnapshotUseCase::new(
        stream_repo.clone(),
        snapshot_repo.clone(),
    ));
    let _get_latest_snapshot_uc = Arc::new(usecase::GetLatestSnapshotUseCase::new(
        stream_repo.clone(),
        snapshot_repo.clone(),
    ));
    let _delete_stream_uc = Arc::new(usecase::DeleteStreamUseCase::new(
        stream_repo,
        event_repo,
        snapshot_repo,
    ));

    let app = axum::Router::new()
        .route(
            "/healthz",
            axum::routing::get(adapter::handler::health::healthz),
        )
        .route(
            "/readyz",
            axum::routing::get(adapter::handler::health::readyz),
        );

    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    axum::serve(listener, app).await?;

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
