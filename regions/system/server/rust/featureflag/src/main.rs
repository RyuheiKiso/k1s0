#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tracing::info;
use uuid::Uuid;

mod adapter;
mod domain;
mod infrastructure;
mod proto;
mod usecase;

use adapter::grpc::FeatureFlagGrpcService;
use domain::entity::feature_flag::FeatureFlag;
use domain::repository::{FeatureFlagRepository, FlagAuditLogRepository};
use infrastructure::config::Config;

fn parse_pool_duration(raw: &str) -> Option<Duration> {
    let s = raw.trim().to_ascii_lowercase();
    if s.is_empty() {
        return None;
    }
    if let Some(v) = s.strip_suffix('s') {
        return v.parse::<u64>().ok().map(Duration::from_secs);
    }
    if let Some(v) = s.strip_suffix('m') {
        return v
            .parse::<u64>()
            .ok()
            .map(|mins| Duration::from_secs(mins * 60));
    }
    if let Some(v) = s.strip_suffix('h') {
        return v
            .parse::<u64>()
            .ok()
            .map(|hours| Duration::from_secs(hours * 60 * 60));
    }
    s.parse::<u64>().ok().map(Duration::from_secs)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Telemetry
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-featureflag-server".to_string(),
        version: "0.1.0".to_string(),
        tier: "system".to_string(),
        environment: cfg.app.environment.clone(),
        trace_endpoint: cfg.observability.trace.enabled.then(|| cfg.observability.trace.endpoint.clone()),
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
        "starting featureflag server"
    );

    // Metrics (shared across layers and repositories)
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-featureflag-server",
    ));

    // Flag repository: PostgreSQL if DATABASE_URL or database config is set, otherwise in-memory
    let (flag_repo, audit_log_repo, local_cache): (
        Arc<dyn FeatureFlagRepository>,
        Arc<dyn FlagAuditLogRepository>,
        Option<Arc<infrastructure::cache::FlagCache>>,
    ) =
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            info!("connecting to PostgreSQL...");
            let max_open_conns = cfg.database.as_ref().map_or(25, |db| db.max_open_conns);
            let max_idle_conns = cfg.database.as_ref().map_or(5, |db| db.max_idle_conns);
            let conn_max_lifetime = cfg
                .database
                .as_ref()
                .map_or("5m", |db| db.conn_max_lifetime.as_str());
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(max_open_conns)
                .min_connections(max_idle_conns.min(max_open_conns))
                .max_lifetime(parse_pool_duration(conn_max_lifetime))
                .connect(&database_url)
                .await?;
            let pool = Arc::new(pool);
            info!("connected to PostgreSQL");
            let pg_repo = Arc::new(
                adapter::repository::featureflag_postgres::FeatureFlagPostgresRepository::new(
                    pool.clone(),
                ),
            );
            let audit_repo: Arc<dyn FlagAuditLogRepository> = Arc::new(
                adapter::repository::flag_audit_log_postgres::FlagAuditLogPostgresRepository::new(
                    pool,
                ),
            );
            // Cache for frequently accessed flags.
            let cache = Arc::new(infrastructure::cache::FlagCache::new(
                cfg.cache.max_entries,
                cfg.cache.ttl_seconds,
            ));
            info!(
                max_entries = cfg.cache.max_entries,
                ttl_seconds = cfg.cache.ttl_seconds,
                "flag cache initialized"
            );
            (
                Arc::new(
                adapter::repository::cached_featureflag_repository::CachedFeatureFlagRepository::with_metrics(
                    pg_repo,
                    cache.clone(),
                    metrics.clone(),
                ),
                ),
                audit_repo,
                Some(cache),
            )
        } else if let Some(ref db_cfg) = cfg.database {
            info!("connecting to PostgreSQL via config...");
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(db_cfg.max_open_conns)
                .min_connections(db_cfg.max_idle_conns.min(db_cfg.max_open_conns))
                .max_lifetime(parse_pool_duration(&db_cfg.conn_max_lifetime))
                .connect(&db_cfg.connection_url())
                .await?;
            let pool = Arc::new(pool);
            info!("connected to PostgreSQL");
            let pg_repo = Arc::new(
                adapter::repository::featureflag_postgres::FeatureFlagPostgresRepository::new(
                    pool.clone(),
                ),
            );
            let audit_repo: Arc<dyn FlagAuditLogRepository> = Arc::new(
                adapter::repository::flag_audit_log_postgres::FlagAuditLogPostgresRepository::new(
                    pool,
                ),
            );
            // 繧ｭ繝｣繝・す繝･縺ｧ繝ｩ繝・・
            let cache = Arc::new(infrastructure::cache::FlagCache::new(
                cfg.cache.max_entries,
                cfg.cache.ttl_seconds,
            ));
            info!(
                max_entries = cfg.cache.max_entries,
                ttl_seconds = cfg.cache.ttl_seconds,
                "flag cache initialized"
            );
            (
                Arc::new(
                adapter::repository::cached_featureflag_repository::CachedFeatureFlagRepository::with_metrics(
                    pg_repo,
                    cache.clone(),
                    metrics.clone(),
                ),
                ),
                audit_repo,
                Some(cache),
            )
        } else {
            info!("no database configured, using in-memory repository");
            (
                Arc::new(InMemoryFeatureFlagRepository::new()),
                Arc::new(NoopFlagAuditLogRepository),
                None,
            )
        };

    // Kafka producer (optional)
    let kafka_producer: Arc<dyn infrastructure::kafka_producer::FlagEventPublisher> =
        if let Some(ref kafka_cfg) = cfg.kafka {
            match infrastructure::kafka_producer::KafkaFlagProducer::new(kafka_cfg) {
                Ok(p) => {
                    info!(
                        topic = %kafka_cfg.topic,
                        "kafka producer initialized for flag change notifications"
                    );
                    Arc::new(p.with_metrics(metrics.clone()))
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "failed to create kafka producer, flag change events will not be published"
                    );
                    Arc::new(infrastructure::kafka_producer::NoopFlagEventPublisher)
                }
            }
        } else {
            Arc::new(infrastructure::kafka_producer::NoopFlagEventPublisher)
        };

    // Kafka consumer for cross-instance cache invalidation.
    if let (Some(kafka_cfg), Some(cache)) = (cfg.kafka.clone(), local_cache.clone()) {
        match infrastructure::kafka_consumer::spawn_flag_cache_invalidator(kafka_cfg, cache) {
            Ok(_) => info!("featureflag cache invalidation consumer started"),
            Err(e) => tracing::warn!(error = %e, "failed to start featureflag cache invalidation consumer"),
        }
    }

    // Use cases
    let list_flags_uc = Arc::new(usecase::ListFlagsUseCase::new(flag_repo.clone()));
    let evaluate_flag_uc = Arc::new(usecase::EvaluateFlagUseCase::new(flag_repo.clone()));
    let get_flag_uc = Arc::new(usecase::GetFlagUseCase::new(flag_repo.clone()));
    let create_flag_uc = Arc::new(usecase::CreateFlagUseCase::new(
        flag_repo.clone(),
        kafka_producer.clone(),
        audit_log_repo.clone(),
    ));
    let update_flag_uc = Arc::new(usecase::UpdateFlagUseCase::new(
        flag_repo.clone(),
        kafka_producer.clone(),
        audit_log_repo.clone(),
    ));
    let delete_flag_uc = Arc::new(usecase::DeleteFlagUseCase::new(
        flag_repo.clone(),
        kafka_producer.clone(),
        audit_log_repo,
    ));

    let grpc_svc = Arc::new(FeatureFlagGrpcService::new(
        list_flags_uc.clone(),
        evaluate_flag_uc.clone(),
        get_flag_uc.clone(),
        create_flag_uc.clone(),
        update_flag_uc.clone(),
        delete_flag_uc.clone(),
    ));

    // tonic wrapper
    use proto::k1s0::system::featureflag::v1::feature_flag_service_server::FeatureFlagServiceServer;
    let featureflag_tonic = adapter::grpc::FeatureFlagServiceTonic::new(grpc_svc);

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = if let Some(ref auth_cfg) = cfg.auth {
        info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for featureflag-server");
        let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
            &auth_cfg.jwks_url,
            &auth_cfg.issuer,
            &auth_cfg.audience,
            std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
        ));
        Some(adapter::middleware::auth::FeatureflagAuthState {
            verifier: jwks_verifier,
        })
    } else {
        info!("no auth configured, featureflag-server running without authentication");
        None
    };

    // AppState for REST handlers
    let mut state = adapter::handler::AppState {
        flag_repo: flag_repo.clone(),
        event_publisher: kafka_producer,
        list_flags_uc,
        evaluate_flag_uc,
        get_flag_uc,
        create_flag_uc,
        update_flag_uc,
        delete_flag_uc,
        metrics: metrics.clone(),
        auth_state: None,
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    // REST router
    let app = adapter::handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    // gRPC server
    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(FeatureFlagServiceServer::new(featureflag_tonic))
            .serve(grpc_addr)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    // REST server
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    let rest_future = axum::serve(listener, app);

    // REST 縺ｨ gRPC 繧剃ｸｦ陦瑚ｵｷ蜍・
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

// --- InMemoryFeatureFlagRepository ---

struct InMemoryFeatureFlagRepository {
    flags: tokio::sync::RwLock<HashMap<String, FeatureFlag>>,
}

struct NoopFlagAuditLogRepository;

#[async_trait::async_trait]
impl FlagAuditLogRepository for NoopFlagAuditLogRepository {
    async fn create(&self, _log: &domain::entity::flag_audit_log::FlagAuditLog) -> anyhow::Result<()> {
        Ok(())
    }

    async fn list_by_flag_id(
        &self,
        _flag_id: &Uuid,
        _limit: i64,
        _offset: i64,
    ) -> anyhow::Result<Vec<domain::entity::flag_audit_log::FlagAuditLog>> {
        Ok(vec![])
    }
}

impl InMemoryFeatureFlagRepository {
    fn new() -> Self {
        Self {
            flags: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl FeatureFlagRepository for InMemoryFeatureFlagRepository {
    async fn find_by_key(&self, flag_key: &str) -> anyhow::Result<FeatureFlag> {
        let flags = self.flags.read().await;
        flags
            .get(flag_key)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("flag not found: {}", flag_key))
    }

    async fn find_all(&self) -> anyhow::Result<Vec<FeatureFlag>> {
        let flags = self.flags.read().await;
        Ok(flags.values().cloned().collect())
    }

    async fn create(&self, flag: &FeatureFlag) -> anyhow::Result<()> {
        let mut flags = self.flags.write().await;
        flags.insert(flag.flag_key.clone(), flag.clone());
        Ok(())
    }

    async fn update(&self, flag: &FeatureFlag) -> anyhow::Result<()> {
        let mut flags = self.flags.write().await;
        flags.insert(flag.flag_key.clone(), flag.clone());
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut flags = self.flags.write().await;
        let key = flags
            .iter()
            .find(|(_, v)| v.id == *id)
            .map(|(k, _)| k.clone());
        if let Some(key) = key {
            flags.remove(&key);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn exists_by_key(&self, flag_key: &str) -> anyhow::Result<bool> {
        let flags = self.flags.read().await;
        Ok(flags.contains_key(flag_key))
    }
}
