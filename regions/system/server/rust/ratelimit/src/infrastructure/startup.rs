use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use crate::adapter;
use crate::domain;
use crate::proto;
use crate::usecase;

use super::cache::RuleCache;
use super::config::Config;
use super::redis_store::RedisRateLimitStore;
use crate::adapter::grpc::RateLimitGrpcService;
use crate::adapter::handler::{self, AppState};
use crate::adapter::repository::cached_ratelimit_repository::CachedRateLimitRepository;
use crate::adapter::repository::ratelimit_postgres::RateLimitPostgresRepository;

fn parse_pool_duration(value: &str) -> Option<std::time::Duration> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(v) = trimmed.strip_suffix("ms") {
        return v.parse::<u64>().ok().map(std::time::Duration::from_millis);
    }
    if let Some(v) = trimmed.strip_suffix('s') {
        return v.parse::<u64>().ok().map(std::time::Duration::from_secs);
    }
    if let Some(v) = trimmed.strip_suffix('m') {
        return v
            .parse::<u64>()
            .ok()
            .map(|mins| std::time::Duration::from_secs(mins * 60));
    }
    if let Some(v) = trimmed.strip_suffix('h') {
        return v
            .parse::<u64>()
            .ok()
            .map(|hours| std::time::Duration::from_secs(hours * 3600));
    }
    trimmed
        .parse::<u64>()
        .ok()
        .map(std::time::Duration::from_secs)
}

pub async fn run() -> anyhow::Result<()> {
    // Telemetry
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let config_content = std::fs::read_to_string(&config_path)?;
    let cfg: Config = serde_yaml::from_str(&config_content)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-ratelimit-server".to_string(),
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

    // Config

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
            .min_connections(db_config.max_idle_conns.min(db_config.max_open_conns))
            .max_lifetime(parse_pool_duration(&db_config.conn_max_lifetime))
            .connect(&url)
            .await?;
        info!("database connection pool established");
        Some(pool)
    } else if let Ok(url) = std::env::var("DATABASE_URL") {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(25)
            .min_connections(5)
            .max_lifetime(parse_pool_duration("5m"))
            .connect(&url)
            .await?;
        info!("database connection pool established from DATABASE_URL");
        Some(pool)
    } else {
        // infra_guard: stable サービスでは DB 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "ratelimit",
            k1s0_server_common::InfraKind::Database,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no database configured, using in-memory repositories (dev/test bypass)");
        None
    };

    // Redis connection (optional)
    let redis_conns = if let Some(ref redis_config) = cfg.redis {
        let url = std::env::var("REDIS_URL").unwrap_or_else(|_| redis_config.url.clone());
        let client = redis::Client::open(url.as_str())?;
        let manager_config = redis::aio::ConnectionManagerConfig::new()
            .set_connection_timeout(std::time::Duration::from_millis(redis_config.timeout_ms))
            .set_response_timeout(std::time::Duration::from_millis(redis_config.timeout_ms));

        let mut conns = Vec::with_capacity(redis_config.pool_size.max(1));
        for _ in 0..redis_config.pool_size.max(1) {
            let conn = redis::aio::ConnectionManager::new_with_config(
                client.clone(),
                manager_config.clone(),
            )
            .await?;
            conns.push(conn);
        }
        info!(
            pool_size = redis_config.pool_size,
            timeout_ms = redis_config.timeout_ms,
            "Redis connection pool established"
        );
        Some(conns)
    } else if let Ok(url) = std::env::var("REDIS_URL") {
        let client = redis::Client::open(url.as_str())?;
        let conn = redis::aio::ConnectionManager::new(client).await?;
        info!("Redis connection established from REDIS_URL");
        Some(vec![conn])
    } else {
        // infra_guard: stable サービスでは Redis 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "ratelimit",
            k1s0_server_common::InfraKind::Redis,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no Redis configured, using in-memory state store (dev/test bypass)");
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
        if let Some(conns) = redis_conns {
            Arc::new(RedisRateLimitStore::new(conns))
        } else {
            Arc::new(InMemoryRateLimitStateStore::new())
        };

    // Use cases
    let check_uc = Arc::new(usecase::CheckRateLimitUseCase::with_fallback_policy(
        rule_repo.clone(),
        state_store.clone(),
        cfg.ratelimit.fail_open,
        cfg.ratelimit.default_limit,
        cfg.ratelimit.default_window_seconds,
    ));
    let create_uc = Arc::new(usecase::CreateRuleUseCase::new(rule_repo.clone()));
    let get_uc = Arc::new(usecase::GetRuleUseCase::new(rule_repo.clone()));
    let list_uc = Arc::new(usecase::ListRulesUseCase::new(rule_repo.clone()));
    let update_uc = Arc::new(usecase::UpdateRuleUseCase::new(rule_repo.clone()));
    let delete_uc = Arc::new(usecase::DeleteRuleUseCase::new(rule_repo.clone()));
    let get_usage_uc = Arc::new(usecase::GetUsageUseCase::with_state_store(
        rule_repo,
        state_store.clone(),
    ));
    let reset_uc = Arc::new(usecase::ResetRateLimitUseCase::new(state_store));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = k1s0_server_common::require_auth_state(
        "ratelimit-server",
        &cfg.app.environment,
        cfg.auth.as_ref().map(|auth_cfg| {
            info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for ratelimit-server");
            let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
                &auth_cfg.jwks_url,
                &auth_cfg.issuer,
                &auth_cfg.audience,
                std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
            ));
            adapter::middleware::auth::RatelimitAuthState {
                verifier: jwks_verifier,
            }
        }),
    )?;

    // AppState (REST handler 用)
    let mut state = AppState::new(
        check_uc.clone(),
        create_uc.clone(),
        get_uc.clone(),
        list_uc.clone(),
        update_uc.clone(),
        delete_uc.clone(),
        get_usage_uc.clone(),
        reset_uc.clone(),
        db_pool,
    );
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    // gRPC service
    let grpc_svc = Arc::new(RateLimitGrpcService::new(
        check_uc,
        create_uc,
        get_uc,
        update_uc,
        delete_uc,
        list_uc,
        get_usage_uc,
        reset_uc,
    ));

    use proto::k1s0::system::ratelimit::v1::rate_limit_service_server::RateLimitServiceServer;
    let tonic_svc = adapter::grpc::RateLimitServiceTonic::new(grpc_svc);

    // Router
    let app = handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

    // gRPC server (port 50051)
    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    // gRPC グレースフルシャットダウン用シグナル
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(RateLimitServiceServer::new(tonic_svc))
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

    async fn find_by_scope(
        &self,
        scope: &str,
    ) -> anyhow::Result<Vec<domain::entity::RateLimitRule>> {
        let rules = self.rules.read().await;
        Ok(rules.iter().filter(|r| r.scope == scope).cloned().collect())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<domain::entity::RateLimitRule>> {
        let rules = self.rules.read().await;
        Ok(rules.clone())
    }

    async fn find_page(
        &self,
        page: u32,
        page_size: u32,
        scope: Option<String>,
        enabled_only: bool,
    ) -> anyhow::Result<(Vec<domain::entity::RateLimitRule>, u64)> {
        let rules = self.rules.read().await;
        let scope = scope.as_deref();
        let mut filtered: Vec<_> = rules
            .iter()
            .filter(|r| scope.is_none_or(|s| r.scope == s))
            .filter(|r| !enabled_only || r.enabled)
            .cloned()
            .collect();
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let page = page.max(1);
        let page_size = page_size.clamp(1, 200);
        let total = filtered.len() as u64;
        let start = ((page - 1) * page_size) as usize;
        let paged = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((paged, total))
    }

    async fn update(&self, rule: &domain::entity::RateLimitRule) -> anyhow::Result<()> {
        let mut rules = self.rules.write().await;
        if let Some(existing) = rules.iter_mut().find(|r| r.id == rule.id) {
            *existing = rule.clone();
        }
        Ok(())
    }

    async fn delete(&self, id: &uuid::Uuid) -> anyhow::Result<bool> {
        let mut rules = self.rules.write().await;
        let len_before = rules.len();
        rules.retain(|r| r.id != *id);
        Ok(rules.len() < len_before)
    }

    async fn reset_state(&self, _key: &str) -> anyhow::Result<()> {
        // インメモリ実装ではRedis状態のリセットは行わない
        Ok(())
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
        let now = chrono::Utc::now();
        Ok(domain::entity::RateLimitDecision::allowed(
            limit,
            limit - 1,
            now + chrono::Duration::seconds(window_secs),
        ))
    }

    async fn check_fixed_window(
        &self,
        _key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<domain::entity::RateLimitDecision> {
        let now = chrono::Utc::now();
        Ok(domain::entity::RateLimitDecision::allowed(
            limit,
            limit - 1,
            now + chrono::Duration::seconds(window_secs),
        ))
    }

    async fn check_sliding_window(
        &self,
        _key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<domain::entity::RateLimitDecision> {
        let now = chrono::Utc::now();
        Ok(domain::entity::RateLimitDecision::allowed(
            limit,
            limit - 1,
            now + chrono::Duration::seconds(window_secs),
        ))
    }

    async fn check_leaky_bucket(
        &self,
        _key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<domain::entity::RateLimitDecision> {
        let now = chrono::Utc::now();
        Ok(domain::entity::RateLimitDecision::allowed(
            limit,
            limit - 1,
            now + chrono::Duration::seconds(window_secs),
        ))
    }

    async fn reset(&self, _key: &str) -> anyhow::Result<()> {
        // インメモリ実装ではリセットは何もしない
        Ok(())
    }

    async fn get_usage(
        &self,
        _key: &str,
        _limit: i64,
        _window_secs: i64,
    ) -> anyhow::Result<Option<domain::repository::UsageSnapshot>> {
        Ok(None)
    }
}
