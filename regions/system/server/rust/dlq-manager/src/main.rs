use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod infrastructure;
mod proto;
mod usecase;

use adapter::grpc::{dlq_grpc::DlqGrpcService, tonic_service::DlqServiceTonic};
use adapter::handler::{self, AppState};
use infrastructure::config::Config;
use infrastructure::persistence::DlqPostgresRepository;
use proto::k1s0::system::dlq::v1::dlq_service_server::DlqServiceServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Telemetry
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-dlq-manager".to_string(),
        version: "0.1.0".to_string(),
        tier: "system".to_string(),
        environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string()),
        trace_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
        sample_rate: 1.0,
        log_level: "info".to_string(),
        log_format: "json".to_string(),
    };
    k1s0_telemetry::init_telemetry(&telemetry_cfg).expect("failed to init telemetry");

    // Config
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let config_content = std::fs::read_to_string(&config_path)?;
    let cfg: Config = serde_yaml::from_str(&config_content)?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting dlq-manager server"
    );

    // Database pool (optional)
    let db_pool = if let Some(ref db_config) = cfg.database {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_config.connection_url());
        info!("connecting to database");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(db_config.max_open_conns)
            .connect(&url)
            .await?;
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
        info!("no database configured, using in-memory repository");
        None
    };

    // Metrics (shared across layers and Kafka producers/consumers)
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-dlq-manager"));

    // DLQ message repository
    let dlq_repo: Arc<dyn domain::repository::DlqMessageRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(DlqPostgresRepository::new(pool.clone()))
        } else {
            Arc::new(InMemoryDlqMessageRepository::new())
        };

    // Kafka producer (optional)
    let publisher: Option<Arc<dyn infrastructure::kafka::producer::DlqEventPublisher>> =
        if let Some(ref kafka_config) = cfg.kafka {
            match infrastructure::kafka::producer::DlqKafkaProducer::new(kafka_config) {
                Ok(producer) => {
                    info!("kafka producer initialized");
                    Some(Arc::new(producer.with_metrics(metrics.clone())))
                }
                Err(e) => {
                    tracing::warn!(error = %e, "failed to create kafka producer, retries will resolve messages without republishing");
                    None
                }
            }
        } else {
            info!("no kafka configured, retries will resolve messages without republishing");
            None
        };

    // Kafka consumer (optional, background task)
    if let Some(ref kafka_config) = cfg.kafka {
        match infrastructure::kafka::consumer::DlqKafkaConsumer::new(kafka_config, dlq_repo.clone())
        {
            Ok(consumer) => {
                let consumer = consumer.with_metrics(metrics.clone());
                info!("kafka consumer initialized, starting background ingestion");
                tokio::spawn(async move {
                    if let Err(e) = consumer.run().await {
                        tracing::error!(error = %e, "kafka consumer stopped with error");
                    }
                });
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to create kafka consumer, DLQ ingestion disabled");
            }
        }
    }

    // Use cases
    let list_messages_uc = Arc::new(usecase::ListMessagesUseCase::new(dlq_repo.clone()));
    let get_message_uc = Arc::new(usecase::GetMessageUseCase::new(dlq_repo.clone()));
    let retry_message_uc = Arc::new(usecase::RetryMessageUseCase::new(
        dlq_repo.clone(),
        publisher.clone(),
    ));
    let delete_message_uc = Arc::new(usecase::DeleteMessageUseCase::new(dlq_repo.clone()));
    let retry_all_uc = Arc::new(usecase::RetryAllUseCase::new(dlq_repo.clone(), publisher));

    // gRPC service
    let dlq_grpc_service = Arc::new(DlqGrpcService::new(
        list_messages_uc.clone(),
        get_message_uc.clone(),
        retry_message_uc.clone(),
        delete_message_uc.clone(),
        retry_all_uc.clone(),
    ));
    let dlq_tonic = DlqServiceTonic::new(dlq_grpc_service);

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = if let Some(ref auth_cfg) = cfg.auth {
        info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for dlq-manager");
        let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
            &auth_cfg.jwks_url,
            &auth_cfg.issuer,
            &auth_cfg.audience,
            std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
        ));
        Some(adapter::middleware::auth::DlqAuthState {
            verifier: jwks_verifier,
        })
    } else {
        info!("no auth configured, dlq-manager running without authentication");
        None
    };

    // AppState
    let mut state = AppState {
        list_messages_uc,
        get_message_uc,
        retry_message_uc,
        delete_message_uc,
        retry_all_uc,
        metrics: metrics.clone(),
        auth_state: None,
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    // Router
    let app = handler::router(state).layer(k1s0_telemetry::MetricsLayer::new(metrics));

    // REST server
    let rest_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port).parse()?;
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;

    // gRPC server
    let grpc_addr: std::net::SocketAddr = ([0, 0, 0, 0], 50051).into();
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .add_service(DlqServiceServer::new(dlq_tonic))
            .serve(grpc_addr)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    let rest_future = async move {
        axum::serve(listener, app)
            .await
            .map_err(|e| anyhow::anyhow!("REST server error: {}", e))
    };

    info!("gRPC server starting on {}", grpc_addr);
    tokio::select! {
        result = rest_future => {
            if let Err(e) = result {
                tracing::error!(error = %e, "REST server stopped");
            }
        }
        result = grpc_future => {
            if let Err(e) = result {
                tracing::error!(error = %e, "gRPC server stopped");
            }
        }
    }

    Ok(())
}

// --- In-memory DLQ Message Repository for dev mode ---

struct InMemoryDlqMessageRepository {
    messages: tokio::sync::RwLock<Vec<domain::entity::DlqMessage>>,
}

impl InMemoryDlqMessageRepository {
    fn new() -> Self {
        Self {
            messages: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl domain::repository::DlqMessageRepository for InMemoryDlqMessageRepository {
    async fn find_by_id(
        &self,
        id: uuid::Uuid,
    ) -> anyhow::Result<Option<domain::entity::DlqMessage>> {
        let messages = self.messages.read().await;
        Ok(messages.iter().find(|m| m.id == id).cloned())
    }

    async fn find_by_topic(
        &self,
        topic: &str,
        page: i32,
        page_size: i32,
    ) -> anyhow::Result<(Vec<domain::entity::DlqMessage>, i64)> {
        let messages = self.messages.read().await;
        let filtered: Vec<_> = messages
            .iter()
            .filter(|m| m.original_topic == topic)
            .cloned()
            .collect();

        let total = filtered.len() as i64;
        let page = page.max(1);
        let page_size = page_size.max(1);
        let offset = ((page - 1) * page_size) as usize;
        let limit = page_size as usize;
        let paged: Vec<_> = filtered.into_iter().skip(offset).take(limit).collect();

        Ok((paged, total))
    }

    async fn create(&self, message: &domain::entity::DlqMessage) -> anyhow::Result<()> {
        self.messages.write().await.push(message.clone());
        Ok(())
    }

    async fn update(&self, message: &domain::entity::DlqMessage) -> anyhow::Result<()> {
        let mut messages = self.messages.write().await;
        if let Some(m) = messages.iter_mut().find(|m| m.id == message.id) {
            *m = message.clone();
        }
        Ok(())
    }

    async fn delete(&self, id: uuid::Uuid) -> anyhow::Result<()> {
        let mut messages = self.messages.write().await;
        messages.retain(|m| m.id != id);
        Ok(())
    }

    async fn count_by_topic(&self, topic: &str) -> anyhow::Result<i64> {
        let messages = self.messages.read().await;
        let count = messages
            .iter()
            .filter(|m| m.original_topic == topic)
            .count();
        Ok(count as i64)
    }
}
