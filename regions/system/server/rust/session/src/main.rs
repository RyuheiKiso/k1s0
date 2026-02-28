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
    // Telemetry
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-session-server".to_string(),
        version: "0.1.0".to_string(),
        tier: "system".to_string(),
        environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string()),
        trace_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
        sample_rate: 1.0,
        log_level: "info".to_string(),
        log_format: "json".to_string(),
    };
    k1s0_telemetry::init_telemetry(&telemetry_cfg).expect("failed to init telemetry");

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

    // --- Kafka consumer (optional, background task) ---
    if let Some(ref kafka_cfg) = cfg.kafka {
        match infrastructure::kafka_consumer::SessionKafkaConsumer::new(
            kafka_cfg,
            revoke_all_uc.clone(),
        ) {
            Ok(consumer) => {
                let consumer = consumer.with_metrics(
                    Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-session-server")),
                );
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

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = if let Some(ref auth_cfg) = cfg.auth {
        info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for session-server");
        let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
            &auth_cfg.jwks_url,
            &auth_cfg.issuer,
            &auth_cfg.audience,
            std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
        ));
        Some(adapter::middleware::auth::SessionAuthState {
            verifier: jwks_verifier,
        })
    } else {
        info!("no auth configured, session-server running without authentication");
        None
    };

    let mut state = adapter::handler::session_handler::AppState {
        create_uc,
        get_uc,
        refresh_uc,
        revoke_uc,
        list_uc,
        revoke_all_uc,
        metrics: metrics.clone(),
        auth_state: None,
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    // 認証不要のエンドポイント
    let public_routes = axum::Router::new()
        .route("/healthz", axum::routing::get(adapter::handler::health::healthz))
        .route("/readyz", axum::routing::get(adapter::handler::health::readyz))
        .route("/metrics", axum::routing::get(metrics_handler));

    // 認証が設定されている場合は RBAC 付きルーティング
    use adapter::middleware::auth::auth_middleware;
    use adapter::middleware::rbac::require_permission;

    let api_routes = if let Some(ref auth_st) = state.auth_state {
        // GET -> sessions/read
        let read_routes = axum::Router::new()
            .route(
                "/api/v1/sessions/:id",
                axum::routing::get(adapter::handler::session_handler::get_session),
            )
            .route(
                "/api/v1/users/:user_id/sessions",
                axum::routing::get(adapter::handler::session_handler::list_user_sessions),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "sessions", "read",
            )));

        // POST/refresh -> sessions/write
        let write_routes = axum::Router::new()
            .route(
                "/api/v1/sessions",
                axum::routing::post(adapter::handler::session_handler::create_session),
            )
            .route(
                "/api/v1/sessions/:id/refresh",
                axum::routing::post(adapter::handler::session_handler::refresh_session),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "sessions", "write",
            )));

        // DELETE -> sessions/admin
        let admin_routes = axum::Router::new()
            .route(
                "/api/v1/sessions/:id",
                axum::routing::delete(adapter::handler::session_handler::revoke_session),
            )
            .route(
                "/api/v1/users/:user_id/sessions",
                axum::routing::delete(adapter::handler::session_handler::revoke_all_sessions),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "sessions", "admin",
            )));

        axum::Router::new()
            .merge(read_routes)
            .merge(write_routes)
            .merge(admin_routes)
            .layer(axum::middleware::from_fn_with_state(
                auth_st.clone(),
                auth_middleware,
            ))
    } else {
        // 認証なし（dev モード / テスト）
        axum::Router::new()
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
    };

    let app = public_routes
        .merge(api_routes)
        .with_state(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    // gRPC service
    use proto::k1s0::system::session::v1::session_service_server::SessionServiceServer;

    let session_tonic = adapter::grpc::SessionServiceTonic::new(grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], 9090).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
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
