#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;
use uuid::Uuid;

mod adapter;
mod domain;
mod infrastructure;
mod usecase;

use adapter::grpc::NotificationGrpcService;
use domain::entity::notification_channel::NotificationChannel;
use domain::entity::notification_log::NotificationLog;
use domain::entity::notification_template::NotificationTemplate;
use domain::repository::NotificationChannelRepository;
use domain::repository::NotificationLogRepository;
use domain::repository::NotificationTemplateRepository;
use infrastructure::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-notification-server".to_string(),
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
        "starting notification server"
    );

    let channel_repo: Arc<dyn NotificationChannelRepository> =
        Arc::new(InMemoryNotificationChannelRepository::new());
    let log_repo: Arc<dyn NotificationLogRepository> =
        Arc::new(InMemoryNotificationLogRepository::new());
    let _template_repo: Arc<dyn NotificationTemplateRepository> =
        Arc::new(InMemoryNotificationTemplateRepository::new());

    let _create_channel_uc = Arc::new(usecase::CreateChannelUseCase::new(channel_repo.clone()));
    let _update_channel_uc = Arc::new(usecase::UpdateChannelUseCase::new(channel_repo.clone()));
    let _send_notification_uc =
        Arc::new(usecase::SendNotificationUseCase::new(channel_repo, log_repo.clone()));

    let _grpc_svc = Arc::new(NotificationGrpcService::new(
        _send_notification_uc,
        log_repo,
    ));

    let grpc_addr: std::net::SocketAddr = "0.0.0.0:9090".parse()?;
    info!("gRPC server starting on {}", grpc_addr);

    let app = axum::Router::new()
        .route("/healthz", axum::routing::get(adapter::handler::health::healthz))
        .route("/readyz", axum::routing::get(adapter::handler::health::readyz));

    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// --- InMemory Repositories ---

struct InMemoryNotificationChannelRepository {
    channels: tokio::sync::RwLock<HashMap<Uuid, NotificationChannel>>,
}

impl InMemoryNotificationChannelRepository {
    fn new() -> Self {
        Self {
            channels: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl NotificationChannelRepository for InMemoryNotificationChannelRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<NotificationChannel>> {
        let channels = self.channels.read().await;
        Ok(channels.get(id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<NotificationChannel>> {
        let channels = self.channels.read().await;
        Ok(channels.values().cloned().collect())
    }

    async fn create(&self, channel: &NotificationChannel) -> anyhow::Result<()> {
        let mut channels = self.channels.write().await;
        channels.insert(channel.id, channel.clone());
        Ok(())
    }

    async fn update(&self, channel: &NotificationChannel) -> anyhow::Result<()> {
        let mut channels = self.channels.write().await;
        channels.insert(channel.id, channel.clone());
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut channels = self.channels.write().await;
        Ok(channels.remove(id).is_some())
    }
}

struct InMemoryNotificationTemplateRepository {
    templates: tokio::sync::RwLock<HashMap<Uuid, NotificationTemplate>>,
}

impl InMemoryNotificationTemplateRepository {
    fn new() -> Self {
        Self {
            templates: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl NotificationTemplateRepository for InMemoryNotificationTemplateRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<NotificationTemplate>> {
        let templates = self.templates.read().await;
        Ok(templates.get(id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<NotificationTemplate>> {
        let templates = self.templates.read().await;
        Ok(templates.values().cloned().collect())
    }

    async fn create(&self, template: &NotificationTemplate) -> anyhow::Result<()> {
        let mut templates = self.templates.write().await;
        templates.insert(template.id, template.clone());
        Ok(())
    }

    async fn update(&self, template: &NotificationTemplate) -> anyhow::Result<()> {
        let mut templates = self.templates.write().await;
        templates.insert(template.id, template.clone());
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut templates = self.templates.write().await;
        Ok(templates.remove(id).is_some())
    }
}

struct InMemoryNotificationLogRepository {
    logs: tokio::sync::RwLock<HashMap<Uuid, NotificationLog>>,
}

impl InMemoryNotificationLogRepository {
    fn new() -> Self {
        Self {
            logs: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl NotificationLogRepository for InMemoryNotificationLogRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<NotificationLog>> {
        let logs = self.logs.read().await;
        Ok(logs.get(id).cloned())
    }

    async fn find_by_channel_id(&self, channel_id: &Uuid) -> anyhow::Result<Vec<NotificationLog>> {
        let logs = self.logs.read().await;
        Ok(logs
            .values()
            .filter(|l| l.channel_id == *channel_id)
            .cloned()
            .collect())
    }

    async fn create(&self, log: &NotificationLog) -> anyhow::Result<()> {
        let mut logs = self.logs.write().await;
        logs.insert(log.id, log.clone());
        Ok(())
    }

    async fn update(&self, log: &NotificationLog) -> anyhow::Result<()> {
        let mut logs = self.logs.write().await;
        logs.insert(log.id, log.clone());
        Ok(())
    }
}
