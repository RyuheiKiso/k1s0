use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use crate::adapter;
use crate::infrastructure;
use crate::proto;
use crate::usecase;

use super::config::Config;
use super::delivery::{
    EmailDeliveryClient, PushDeliveryClient, SlackDeliveryClient, SmsDeliveryClient,
    WebhookDeliveryClient,
};
use super::kafka_producer::{
    KafkaNotificationProducer, NoopNotificationEventPublisher, NotificationEventPublisher,
};
use crate::adapter::grpc::NotificationGrpcService;
use crate::adapter::repository::channel_postgres::ChannelPostgresRepository;
use crate::adapter::repository::notification_log_postgres::NotificationLogPostgresRepository;
use crate::adapter::repository::template_postgres::TemplatePostgresRepository;
use crate::domain::entity::notification_channel::NotificationChannel;
use crate::domain::entity::notification_log::NotificationLog;
use crate::domain::entity::notification_template::NotificationTemplate;
use crate::domain::repository::NotificationChannelRepository;
use crate::domain::repository::NotificationLogRepository;
use crate::domain::repository::NotificationTemplateRepository;
use crate::domain::service::DeliveryClient;

pub async fn run() -> anyhow::Result<()> {
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-notification-server".to_string(),
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
        // infra_guard: stable サービスでは DB 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "notification",
            k1s0_server_common::InfraKind::Database,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no database configured, using in-memory repositories (dev/test bypass)");
        (
            Arc::new(InMemoryNotificationChannelRepository::new()),
            Arc::new(InMemoryNotificationLogRepository::new()),
            Arc::new(InMemoryNotificationTemplateRepository::new()),
        )
    };

    // --- Kafka wiring: KafkaProducer or Noop fallback ---
    let event_publisher: Arc<dyn NotificationEventPublisher> =
        if let Some(ref kafka_cfg) = cfg.kafka {
            info!("initializing Kafka producer");
            let producer = KafkaNotificationProducer::new(kafka_cfg)?;
            info!(topic = %producer.topic(), "Kafka producer initialized");
            Arc::new(producer)
        } else {
            // infra_guard: stable サービスでは Kafka 設定を必須化（dev/test 以外はエラー）
            k1s0_server_common::require_infra(
                "notification",
                k1s0_server_common::InfraKind::Kafka,
                &cfg.app.environment,
                None::<String>,
            )?;
            info!("no Kafka configured, using noop event publisher (dev/test bypass)");
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
        let from_address =
            std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@k1s0.dev".to_string());
        match EmailDeliveryClient::new(&smtp_host, smtp_port, &smtp_user, &smtp_pass, &from_address)
        {
            Ok(client) => {
                info!(
                    "Email delivery client initialized (SMTP: {}:{})",
                    smtp_host, smtp_port
                );
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

    if let Ok(sms_endpoint) = std::env::var("SMS_API_ENDPOINT") {
        info!("SMS delivery client initialized");
        delivery_clients.insert(
            "sms".to_string(),
            Arc::new(SmsDeliveryClient::new(
                sms_endpoint,
                std::env::var("SMS_API_KEY").ok(),
            )),
        );
    } else {
        info!("SMS_API_ENDPOINT not configured, skipping SMS delivery client");
    }

    if let Ok(push_endpoint) = std::env::var("PUSH_API_ENDPOINT") {
        info!("Push delivery client initialized");
        delivery_clients.insert(
            "push".to_string(),
            Arc::new(PushDeliveryClient::new(
                push_endpoint,
                std::env::var("PUSH_AUTH_TOKEN").ok(),
            )),
        );
    } else {
        info!("PUSH_API_ENDPOINT not configured, skipping Push delivery client");
    }

    let send_notification_uc = if delivery_clients.is_empty() {
        Arc::new(
            usecase::SendNotificationUseCase::with_template_repo(
                channel_repo.clone(),
                log_repo.clone(),
                template_repo.clone(),
            )
            .with_event_publisher(event_publisher.clone()),
        )
    } else {
        Arc::new(
            usecase::SendNotificationUseCase::with_delivery_clients_and_template_repo(
                channel_repo.clone(),
                log_repo.clone(),
                template_repo.clone(),
                delivery_clients,
            )
            .with_event_publisher(event_publisher.clone()),
        )
    };
    let retry_notification_uc = Arc::new(usecase::RetryNotificationUseCase::new(
        log_repo.clone(),
        channel_repo.clone(),
    ));
    let create_template_uc = Arc::new(usecase::CreateTemplateUseCase::new(template_repo.clone()));
    let list_templates_uc = Arc::new(usecase::ListTemplatesUseCase::new(template_repo.clone()));
    let get_template_uc = Arc::new(usecase::GetTemplateUseCase::new(template_repo.clone()));
    let update_template_uc = Arc::new(usecase::UpdateTemplateUseCase::new(template_repo.clone()));
    let delete_template_uc = Arc::new(usecase::DeleteTemplateUseCase::new(template_repo));

    // --- Kafka consumer (optional, background task) ---
    if let Some(ref kafka_cfg) = cfg.kafka {
        match infrastructure::kafka_consumer::NotificationKafkaConsumer::new(
            kafka_cfg,
            send_notification_uc.clone(),
        ) {
            Ok(consumer) => {
                let consumer = consumer.with_metrics(Arc::new(
                    k1s0_telemetry::metrics::Metrics::new("k1s0-notification-server"),
                ));
                info!("kafka consumer initialized, starting background ingestion");
                tokio::spawn(async move {
                    if let Err(e) = consumer.run().await {
                        tracing::error!(error = %e, "kafka consumer stopped with error");
                    }
                });
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to create kafka consumer");
            }
        }
    }

    let grpc_svc = Arc::new(NotificationGrpcService::with_management(
        send_notification_uc.clone(),
        retry_notification_uc.clone(),
        log_repo.clone(),
        channel_repo.clone(),
        create_channel_uc.clone(),
        list_channels_uc.clone(),
        get_channel_uc.clone(),
        update_channel_uc.clone(),
        delete_channel_uc.clone(),
        create_template_uc.clone(),
        list_templates_uc.clone(),
        get_template_uc.clone(),
        update_template_uc.clone(),
        delete_template_uc.clone(),
    ));

    let grpc_addr: std::net::SocketAddr =
        format!("{}:{}", cfg.server.host, cfg.server.grpc_port).parse()?;
    info!("gRPC server starting on {}", grpc_addr);

    // tonic wrapper
    use proto::k1s0::system::notification::v1::notification_service_server::NotificationServiceServer;
    let notification_tonic = adapter::grpc::NotificationServiceTonic::new(grpc_svc);

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-notification-server",
    ));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = k1s0_server_common::require_auth_state(
        "notification-server",
        &cfg.app.environment,
        cfg.auth.as_ref().map(|auth_cfg| {
        info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for notification-server");
        let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
            &auth_cfg.jwks_url,
            &auth_cfg.issuer,
            &auth_cfg.audience,
            std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
        ).expect("Failed to create JWKS verifier"));
        adapter::middleware::auth::NotificationAuthState {
            verifier: jwks_verifier,
        }
        }),
    )?;

    let mut state = adapter::handler::AppState {
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
        metrics: metrics.clone(),
        auth_state: None,
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    let app = adapter::handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

    // gRPC グレースフルシャットダウン用シグナル
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();

    // gRPC server
    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(NotificationServiceServer::new(notification_tonic))
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
    // REST グレースフルシャットダウンを設定
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

    // テレメトリのシャットダウン処理を実行
    k1s0_telemetry::shutdown();

    Ok(())
}

// --- InMemory Repositories ---

struct InMemoryNotificationChannelRepository {
    channels: tokio::sync::RwLock<HashMap<String, NotificationChannel>>,
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
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<NotificationChannel>> {
        let channels = self.channels.read().await;
        Ok(channels.get(id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<NotificationChannel>> {
        let channels = self.channels.read().await;
        Ok(channels.values().cloned().collect())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        channel_type: Option<String>,
        enabled_only: bool,
    ) -> anyhow::Result<(Vec<NotificationChannel>, u64)> {
        let channels = self.channels.read().await;
        let mut filtered: Vec<NotificationChannel> = channels
            .values()
            .filter(|ch| {
                if enabled_only && !ch.enabled {
                    return false;
                }
                if let Some(ref ct) = channel_type {
                    if ch.channel_type != *ct {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let total = filtered.len() as u64;
        let start = ((page.saturating_sub(1)) * page_size) as usize;
        let items: Vec<NotificationChannel> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((items, total))
    }

    async fn create(&self, channel: &NotificationChannel) -> anyhow::Result<()> {
        let mut channels = self.channels.write().await;
        channels.insert(channel.id.clone(), channel.clone());
        Ok(())
    }

    async fn update(&self, channel: &NotificationChannel) -> anyhow::Result<()> {
        let mut channels = self.channels.write().await;
        channels.insert(channel.id.clone(), channel.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        let mut channels = self.channels.write().await;
        Ok(channels.remove(id).is_some())
    }
}

struct InMemoryNotificationTemplateRepository {
    templates: tokio::sync::RwLock<HashMap<String, NotificationTemplate>>,
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
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<NotificationTemplate>> {
        let templates = self.templates.read().await;
        Ok(templates.get(id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<NotificationTemplate>> {
        let templates = self.templates.read().await;
        Ok(templates.values().cloned().collect())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        channel_type: Option<String>,
    ) -> anyhow::Result<(Vec<NotificationTemplate>, u64)> {
        let templates = self.templates.read().await;
        let mut filtered: Vec<NotificationTemplate> = templates
            .values()
            .filter(|t| {
                if let Some(ref ct) = channel_type {
                    if t.channel_type != *ct {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let total = filtered.len() as u64;
        let start = ((page.saturating_sub(1)) * page_size) as usize;
        let items: Vec<NotificationTemplate> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((items, total))
    }

    async fn create(&self, template: &NotificationTemplate) -> anyhow::Result<()> {
        let mut templates = self.templates.write().await;
        templates.insert(template.id.clone(), template.clone());
        Ok(())
    }

    async fn update(&self, template: &NotificationTemplate) -> anyhow::Result<()> {
        let mut templates = self.templates.write().await;
        templates.insert(template.id.clone(), template.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        let mut templates = self.templates.write().await;
        Ok(templates.remove(id).is_some())
    }
}

struct InMemoryNotificationLogRepository {
    logs: tokio::sync::RwLock<HashMap<String, NotificationLog>>,
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
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<NotificationLog>> {
        let logs = self.logs.read().await;
        Ok(logs.get(id).cloned())
    }

    async fn find_by_channel_id(&self, channel_id: &str) -> anyhow::Result<Vec<NotificationLog>> {
        let logs = self.logs.read().await;
        Ok(logs
            .values()
            .filter(|l| l.channel_id == channel_id)
            .cloned()
            .collect())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        channel_id: Option<String>,
        status: Option<String>,
    ) -> anyhow::Result<(Vec<NotificationLog>, u64)> {
        let logs = self.logs.read().await;
        let mut filtered: Vec<NotificationLog> = logs
            .values()
            .filter(|l| {
                if let Some(ref cid) = channel_id {
                    if l.channel_id != *cid {
                        return false;
                    }
                }
                if let Some(ref s) = status {
                    if l.status != *s {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let total = filtered.len() as u64;
        let start = ((page.saturating_sub(1)) * page_size) as usize;
        let items: Vec<NotificationLog> = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((items, total))
    }

    async fn create(&self, log: &NotificationLog) -> anyhow::Result<()> {
        let mut logs = self.logs.write().await;
        logs.insert(log.id.clone(), log.clone());
        Ok(())
    }

    async fn update(&self, log: &NotificationLog) -> anyhow::Result<()> {
        let mut logs = self.logs.write().await;
        logs.insert(log.id.clone(), log.clone());
        Ok(())
    }
}
