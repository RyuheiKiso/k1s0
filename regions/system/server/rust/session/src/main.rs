#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod error;
mod infrastructure;
mod proto;
mod usecase;

use adapter::grpc::SessionGrpcService;
use adapter::repository::session_metadata_postgres::{
    NoopSessionMetadataRepository, SessionMetadataPostgresRepository, SessionMetadataRepository,
};
use adapter::repository::session_redis::RedisSessionRepository;
use domain::entity::session::Session;
use domain::repository::SessionRepository;
use error::SessionError;
use infrastructure::config::Config;
use infrastructure::kafka_producer::{
    KafkaSessionProducer, NoopSessionEventPublisher, SessionEventPublisher,
};

async fn metrics_handler(
    axum::extract::State(state): axum::extract::State<adapter::handler::session_handler::AppState>,
) -> impl axum::response::IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        axum::http::StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

// --- InMemory Repository ---

struct InMemorySessionRepository {
    sessions: tokio::sync::RwLock<HashMap<String, Session>>,
}

impl InMemorySessionRepository {
    fn new() -> Self {
        Self {
            sessions: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl SessionRepository for InMemorySessionRepository {
    async fn save(&self, session: &Session) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(id).cloned())
    }

    async fn find_by_token(&self, token: &str) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().find(|s| s.token == token).cloned())
    }

    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Session>, SessionError> {
        let sessions = self.sessions.read().await;
        Ok(sessions
            .values()
            .filter(|s| s.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn delete(&self, id: &str) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(id);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse().unwrap()),
        )
        .json()
        .init();

    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    info!(port = cfg.server.port, "starting session server");

    // --- Session Repository: Redis or InMemory fallback ---
    let repo: Arc<dyn SessionRepository> = if let Some(ref redis_cfg) = cfg.redis {
        info!(url = %redis_cfg.url, "connecting to Redis");
        let client = redis::Client::open(redis_cfg.url.as_str())
            .map_err(|e| anyhow::anyhow!("failed to create Redis client: {}", e))?;
        let conn = redis::aio::ConnectionManager::new(client)
            .await
            .map_err(|e| anyhow::anyhow!("failed to connect to Redis: {}", e))?;
        info!("Redis connection established");
        Arc::new(RedisSessionRepository::new(conn))
    } else {
        info!("Redis not configured, using InMemory session repository");
        Arc::new(InMemorySessionRepository::new())
    };

    // --- Session Metadata Repository: PostgreSQL or Noop fallback ---
    let metadata_repo: Arc<dyn SessionMetadataRepository> = if let Some(ref db_cfg) = cfg.database
    {
        info!("connecting to PostgreSQL for session metadata");
        let pool = infrastructure::database::create_pool(&db_cfg.url, db_cfg.max_connections).await?;
        info!("PostgreSQL connection pool established");
        Arc::new(SessionMetadataPostgresRepository::new(Arc::new(pool)))
    } else {
        info!("Database not configured, using Noop session metadata repository");
        Arc::new(NoopSessionMetadataRepository)
    };

    // --- Event Publisher: Kafka or Noop fallback ---
    let event_publisher: Arc<dyn SessionEventPublisher> = if let Some(ref kafka_cfg) = cfg.kafka {
        info!(brokers = ?kafka_cfg.brokers, "connecting to Kafka");
        let producer = KafkaSessionProducer::new(kafka_cfg)?;
        info!("Kafka producer initialized");
        Arc::new(producer)
    } else {
        info!("Kafka not configured, using Noop event publisher");
        Arc::new(NoopSessionEventPublisher)
    };

    let create_uc = Arc::new(usecase::CreateSessionUseCase::new(
        repo.clone(),
        cfg.session.default_ttl_seconds,
        cfg.session.max_ttl_seconds,
    ));
    let get_uc = Arc::new(usecase::GetSessionUseCase::new(repo.clone()));
    let refresh_uc = Arc::new(usecase::RefreshSessionUseCase::new(
        repo.clone(),
        cfg.session.max_ttl_seconds,
    ));
    let revoke_uc = Arc::new(usecase::RevokeSessionUseCase::new(repo.clone()));
    let list_uc = Arc::new(usecase::ListUserSessionsUseCase::new(repo.clone()));
    let revoke_all_uc = Arc::new(usecase::RevokeAllSessionsUseCase::new(repo));

    let grpc_svc = Arc::new(SessionGrpcService::new(
        create_uc.clone(),
        get_uc.clone(),
        refresh_uc.clone(),
        revoke_uc.clone(),
        revoke_all_uc.clone(),
        list_uc.clone(),
        cfg.session.default_ttl_seconds,
    ));

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-session-server",
    ));

    // Log metadata and event publisher status
    info!(
        has_metadata_repo = cfg.database.is_some(),
        has_event_publisher = cfg.kafka.is_some(),
        "infrastructure components initialized"
    );

    // Store metadata_repo and event_publisher for future use by handlers
    let _metadata_repo = metadata_repo;
    let _event_publisher = event_publisher;

    let state = adapter::handler::session_handler::AppState {
        create_uc,
        get_uc,
        refresh_uc,
        revoke_uc,
        list_uc,
        revoke_all_uc,
        metrics,
    };

    let app = axum::Router::new()
        .route("/healthz", axum::routing::get(adapter::handler::health::healthz))
        .route("/readyz", axum::routing::get(adapter::handler::health::readyz))
        .route("/metrics", axum::routing::get(metrics_handler))
        .route(
            "/api/v1/sessions",
            axum::routing::post(adapter::handler::session_handler::create_session),
        )
        .route(
            "/api/v1/sessions/:id",
            axum::routing::get(adapter::handler::session_handler::get_session)
                .delete(adapter::handler::session_handler::revoke_session),
        )
        .route(
            "/api/v1/sessions/:id/refresh",
            axum::routing::post(adapter::handler::session_handler::refresh_session),
        )
        .route(
            "/api/v1/users/:user_id/sessions",
            axum::routing::get(adapter::handler::session_handler::list_user_sessions)
                .delete(adapter::handler::session_handler::revoke_all_sessions),
        )
        .with_state(state);

    // gRPC service
    use proto::k1s0::system::session::v1::session_service_server::SessionServiceServer;

    let session_tonic = adapter::grpc::SessionServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], 9090).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_future = async move {
        tonic::transport::Server::builder()
            .add_service(SessionServiceServer::new(session_tonic))
            .serve(grpc_addr)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let rest_future = axum::serve(listener, app);

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
