#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;
use uuid::Uuid;

mod adapter;
mod domain;
mod infrastructure;
mod proto;
mod usecase;

use adapter::grpc::NotificationGrpcService;
use adapter::repository::channel_postgres::ChannelPostgresRepository;
use adapter::repository::notification_log_postgres::NotificationLogPostgresRepository;
use adapter::repository::template_postgres::TemplatePostgresRepository;
use domain::entity::notification_channel::NotificationChannel;
use domain::entity::notification_log::NotificationLog;
use domain::entity::notification_template::NotificationTemplate;
use domain::repository::NotificationChannelRepository;
use domain::repository::NotificationLogRepository;
use domain::repository::NotificationTemplateRepository;
use domain::service::DeliveryClient;
use infrastructure::config::Config;
use infrastructure::delivery::{EmailDeliveryClient, SlackDeliveryClient, WebhookDeliveryClient};
use infrastructure::kafka_producer::{
    KafkaNotificationProducer, NoopNotificationEventPublisher, NotificationEventPublisher,
};

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

    // --- Repository wiring: PostgreSQL or InMemory fallback ---
    let (channel_repo, log_repo, template_repo): (
        Arc<dyn NotificationChannelRepository>,
        Arc<dyn NotificationLogRepository>,
        Arc<dyn NotificationTemplateRepository>,
    ) = if let Some(ref db_cfg) = cfg.database {
        info!("connecting to PostgreSQL");
        let pool = Arc::new(infrastructure::database::connect(db_cfg).await?);
        info!("PostgreSQL connection established");

        (
            Arc::new(ChannelPostgresRepository::new(pool.clone())),
            Arc::new(NotificationLogPostgresRepository::new(pool.clone())),
            Arc::new(TemplatePostgresRepository::new(pool)),
        )
    } else {
        info!("no database configured, using in-memory repositories");
        (
            Arc::new(InMemoryNotificationChannelRepository::new()),
            Arc::new(InMemoryNotificationLogRepository::new()),
            Arc::new(InMemoryNotificationTemplateRepository::new()),
        )
    };

    // --- Kafka wiring: KafkaProducer or Noop fallback ---
    let _event_publisher: Arc<dyn NotificationEventPublisher> =
        if let Some(ref kafka_cfg) = cfg.kafka {
            info!("initializing Kafka producer");
            let producer = KafkaNotificationProducer::new(kafka_cfg)?;
            info!(topic = %producer.topic(), "Kafka producer initialized");
            Arc::new(producer)
        } else {
            info!("no Kafka configured, using noop event publisher");
            Arc::new(NoopNotificationEventPublisher)
        };

    let create_channel_uc = Arc::new(usecase::CreateChannelUseCase::new(channel_repo.clone()));
    let list_channels_uc = Arc::new(usecase::ListChannelsUseCase::new(channel_repo.clone()));
    let get_channel_uc = Arc::new(usecase::GetChannelUseCase::new(channel_repo.clone()));
    let update_channel_uc = Arc::new(usecase::UpdateChannelUseCase::new(channel_repo.clone()));
    let delete_channel_uc = Arc::new(usecase::DeleteChannelUseCase::new(channel_repo.clone()));

    // --- Delivery client wiring ---
    let mut delivery_clients: HashMap<String, Arc<dyn DeliveryClient>> = HashMap::new();

    if let (Ok(smtp_host), Ok(smtp_user), Ok(smtp_pass)) = (
        std::env::var("SMTP_HOST"),
        std::env::var("SMTP_USERNAME"),
        std::env::var("SMTP_PASSWORD"),
    ) {
        let smtp_port: u16 = std::env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .unwrap_or(587);
        let from_address = std::env::var("SMTP_FROM")
            .unwrap_or_else(|_| "noreply@k1s0.dev".to_string());
        match EmailDeliveryClient::new(&smtp_host, smtp_port, &smtp_user, &smtp_pass, &from_address) {
            Ok(client) => {
                info!("Email delivery client initialized (SMTP: {}:{})", smtp_host, smtp_port);
                delivery_clients.insert("email".to_string(), Arc::new(client));
            }
            Err(e) => {
                tracing::warn!("Failed to initialize email delivery client: {}", e);
            }
        }
    } else {
        info!("SMTP not configured, skipping email delivery client");
    }

    if let Ok(slack_webhook_url) = std::env::var("SLACK_WEBHOOK_URL") {
        info!("Slack delivery client initialized");
        delivery_clients.insert(
            "slack".to_string(),
            Arc::new(SlackDeliveryClient::new(slack_webhook_url)),
        );
    } else {
        info!("SLACK_WEBHOOK_URL not configured, skipping Slack delivery client");
    }

    if let Ok(webhook_url) = std::env::var("WEBHOOK_URL") {
        info!("Webhook delivery client initialized");
        delivery_clients.insert(
            "webhook".to_string(),
            Arc::new(WebhookDeliveryClient::new(webhook_url, None)),
        );
    } else {
        info!("WEBHOOK_URL not configured, skipping webhook delivery client");
    }

    let send_notification_uc = if delivery_clients.is_empty() {
        Arc::new(usecase::SendNotificationUseCase::new(channel_repo.clone(), log_repo.clone()))
    } else {
        Arc::new(usecase::SendNotificationUseCase::with_delivery_clients(
            channel_repo.clone(),
            log_repo.clone(),
            delivery_clients,
        ))
    };
    let retry_notification_uc =
        Arc::new(usecase::RetryNotificationUseCase::new(log_repo.clone(), channel_repo));
    let create_template_uc = Arc::new(usecase::CreateTemplateUseCase::new(template_repo.clone()));
    let list_templates_uc = Arc::new(usecase::ListTemplatesUseCase::new(template_repo.clone()));
    let get_template_uc = Arc::new(usecase::GetTemplateUseCase::new(template_repo.clone()));
    let update_template_uc = Arc::new(usecase::UpdateTemplateUseCase::new(template_repo.clone()));
    let delete_template_uc = Arc::new(usecase::DeleteTemplateUseCase::new(template_repo));

    let grpc_svc = Arc::new(NotificationGrpcService::new(
        send_notification_uc.clone(),
        log_repo.clone(),
    ));

    let grpc_addr: std::net::SocketAddr = "0.0.0.0:9090".parse()?;
    info!("gRPC server starting on {}", grpc_addr);

    // tonic wrapper
    use proto::k1s0::system::notification::v1::notification_service_server::NotificationServiceServer;
    let notification_tonic = adapter::grpc::NotificationServiceTonic::new(grpc_svc);

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-notification-server",
    ));

    let state = adapter::handler::AppState {
        send_notification_uc,
        retry_notification_uc,
        log_repo,
        create_channel_uc,
        list_channels_uc,
        get_channel_uc,
        update_channel_uc,
        delete_channel_uc,
        create_template_uc,
        list_templates_uc,
        get_template_uc,
        update_template_uc,
        delete_template_uc,
        metrics,
    };

    let app = adapter::handler::router(state);

    // gRPC server
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .add_service(NotificationServiceServer::new(notification_tonic))
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
