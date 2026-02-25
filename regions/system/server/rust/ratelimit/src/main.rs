#![allow(dead_code, unused_imports)]

use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod infrastructure;
mod proto;
mod usecase;

use adapter::grpc::RateLimitGrpcService;
use adapter::handler::{self, AppState};
use adapter::repository::cached_ratelimit_repository::CachedRateLimitRepository;
use adapter::repository::ratelimit_postgres::RateLimitPostgresRepository;
use infrastructure::cache::RuleCache;
use infrastructure::config::Config;
use infrastructure::redis_store::RedisRateLimitStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Telemetry
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-ratelimit-server".to_string(),
        version: "0.1.0".to_string(),
        tier: "system".to_string(),
        environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string()),
        trace_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
        sample_rate: 1.0,
        log_level: "info".to_string(),
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
        "starting ratelimit server"
    );

    // Database pool (optional)
    let db_pool = if let Some(ref db_config) = cfg.database {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_config.connection_url());
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
        info!("no database configured, using in-memory repositories");
        None
    };

    // Redis connection (optional)
    let redis_conn = if let Some(ref redis_config) = cfg.redis {
        let url = std::env::var("REDIS_URL").unwrap_or_else(|_| redis_config.url.clone());
        let client = redis::Client::open(url.as_str())?;
        let conn = redis::aio::ConnectionManager::new(client).await?;
        info!("Redis connection established");
        Some(conn)
    } else if let Ok(url) = std::env::var("REDIS_URL") {
        let client = redis::Client::open(url.as_str())?;
        let conn = redis::aio::ConnectionManager::new(client).await?;
        info!("Redis connection established from REDIS_URL");
        Some(conn)
    } else {
        info!("no Redis configured, using in-memory state store");
        None
    };

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-ratelimit-server",
    ));

    // Rule cache (max 2000 entries, TTL 120 seconds)
    let rule_cache = Arc::new(RuleCache::new(2000, 120));

    // Repositories
    let rule_repo: Arc<dyn domain::repository::RateLimitRepository> =
        if let Some(ref pool) = db_pool {
            let inner: Arc<dyn domain::repository::RateLimitRepository> =
                Arc::new(RateLimitPostgresRepository::new(pool.clone()));
            Arc::new(CachedRateLimitRepository::with_metrics(
                inner,
                rule_cache,
                metrics.clone(),
            ))
        } else {
            Arc::new(InMemoryRateLimitRepository::new())
        };

    let state_store: Arc<dyn domain::repository::RateLimitStateStore> =
        if let Some(conn) = redis_conn {
            Arc::new(RedisRateLimitStore::new(conn))
        } else {
            Arc::new(InMemoryRateLimitStateStore::new())
        };

    // Use cases
    let check_uc = Arc::new(usecase::CheckRateLimitUseCase::new(
        rule_repo.clone(),
        state_store,
    ));
    let create_uc = Arc::new(usecase::CreateRuleUseCase::new(rule_repo.clone()));
    let get_uc = Arc::new(usecase::GetRuleUseCase::new(rule_repo));

    // AppState (REST handler 用)
    let state = AppState::new(check_uc.clone(), create_uc.clone(), get_uc.clone(), db_pool);

    // gRPC service
    let grpc_svc = Arc::new(RateLimitGrpcService::new(check_uc, create_uc, get_uc));

    use proto::k1s0::system::ratelimit::v1::rate_limit_service_server::RateLimitServiceServer;
    let tonic_svc = adapter::grpc::RateLimitServiceTonic::new(grpc_svc);

    // Router
    let app = handler::router(state).layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    // gRPC server (port 50051)
    let grpc_addr: SocketAddr = ([0, 0, 0, 0], 50051).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(RateLimitServiceServer::new(tonic_svc))
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

// --- In-memory implementations for dev mode ---

struct InMemoryRateLimitRepository {
    rules: tokio::sync::RwLock<Vec<domain::entity::RateLimitRule>>,
}

impl InMemoryRateLimitRepository {
    fn new() -> Self {
        Self {
            rules: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl domain::repository::RateLimitRepository for InMemoryRateLimitRepository {
    async fn create(
        &self,
        rule: &domain::entity::RateLimitRule,
    ) -> anyhow::Result<domain::entity::RateLimitRule> {
        let mut rules = self.rules.write().await;
        rules.push(rule.clone());
        Ok(rule.clone())
    }

    async fn find_by_id(&self, id: &uuid::Uuid) -> anyhow::Result<domain::entity::RateLimitRule> {
        let rules = self.rules.read().await;
        rules
            .iter()
            .find(|r| r.id == *id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("rule not found: {}", id))
    }

    async fn find_by_name(
        &self,
        name: &str,
    ) -> anyhow::Result<Option<domain::entity::RateLimitRule>> {
        let rules = self.rules.read().await;
        Ok(rules.iter().find(|r| r.name == name).cloned())
    }
}

struct InMemoryRateLimitStateStore;

impl InMemoryRateLimitStateStore {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl domain::repository::RateLimitStateStore for InMemoryRateLimitStateStore {
    async fn check_token_bucket(
        &self,
        _key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<domain::entity::RateLimitDecision> {
        let now = chrono::Utc::now().timestamp();
        Ok(domain::entity::RateLimitDecision::allowed(
            limit - 1,
            now + window_secs,
        ))
    }

    async fn check_fixed_window(
        &self,
        _key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<domain::entity::RateLimitDecision> {
        let now = chrono::Utc::now().timestamp();
        Ok(domain::entity::RateLimitDecision::allowed(
            limit - 1,
            now + window_secs,
        ))
    }

    async fn check_sliding_window(
        &self,
        _key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<domain::entity::RateLimitDecision> {
        let now = chrono::Utc::now().timestamp();
        Ok(domain::entity::RateLimitDecision::allowed(
            limit - 1,
            now + window_secs,
        ))
    }
}
