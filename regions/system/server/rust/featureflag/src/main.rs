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

use adapter::grpc::FeatureFlagGrpcService;
use domain::entity::feature_flag::FeatureFlag;
use domain::repository::FeatureFlagRepository;
use infrastructure::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Telemetry
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-featureflag-server".to_string(),
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
    let cfg = Config::load(&config_path)?;

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
    let flag_repo: Arc<dyn FeatureFlagRepository> =
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            info!("connecting to PostgreSQL...");
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(cfg.database.as_ref().map_or(25, |db| db.max_open_conns))
                .connect(&database_url)
                .await?;
            info!("connected to PostgreSQL");
            let pg_repo = Arc::new(
                adapter::repository::featureflag_postgres::FeatureFlagPostgresRepository::new(
                    Arc::new(pool),
                ),
            );
            // キャッシュでラップ（設定から TTL と最大エントリ数を読み取り）
            let cache = Arc::new(infrastructure::cache::FlagCache::new(
                cfg.cache.max_entries,
                cfg.cache.ttl_seconds,
            ));
            info!(
                max_entries = cfg.cache.max_entries,
                ttl_seconds = cfg.cache.ttl_seconds,
                "flag cache initialized"
            );
            Arc::new(
                adapter::repository::cached_featureflag_repository::CachedFeatureFlagRepository::with_metrics(
                    pg_repo,
                    cache,
                    metrics.clone(),
                ),
            )
        } else if let Some(ref db_cfg) = cfg.database {
            info!("connecting to PostgreSQL via config...");
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(db_cfg.max_open_conns)
                .connect(&db_cfg.connection_url())
                .await?;
            info!("connected to PostgreSQL");
            let pg_repo = Arc::new(
                adapter::repository::featureflag_postgres::FeatureFlagPostgresRepository::new(
                    Arc::new(pool),
                ),
            );
            // キャッシュでラップ
            let cache = Arc::new(infrastructure::cache::FlagCache::new(
                cfg.cache.max_entries,
                cfg.cache.ttl_seconds,
            ));
            info!(
                max_entries = cfg.cache.max_entries,
                ttl_seconds = cfg.cache.ttl_seconds,
                "flag cache initialized"
            );
            Arc::new(
                adapter::repository::cached_featureflag_repository::CachedFeatureFlagRepository::with_metrics(
                    pg_repo,
                    cache,
                    metrics.clone(),
                ),
            )
        } else {
            info!("no database configured, using in-memory repository");
            Arc::new(InMemoryFeatureFlagRepository::new())
        };

    // Kafka producer (optional)
    let _kafka_producer: Arc<dyn infrastructure::kafka_producer::FlagEventPublisher> =
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

    // Use cases
    let evaluate_flag_uc = Arc::new(usecase::EvaluateFlagUseCase::new(flag_repo.clone()));
    let get_flag_uc = Arc::new(usecase::GetFlagUseCase::new(flag_repo.clone()));
    let create_flag_uc = Arc::new(usecase::CreateFlagUseCase::new(flag_repo.clone()));
    let update_flag_uc = Arc::new(usecase::UpdateFlagUseCase::new(flag_repo.clone()));
    let delete_flag_uc = Arc::new(usecase::DeleteFlagUseCase::new(flag_repo.clone()));

    let grpc_svc = Arc::new(FeatureFlagGrpcService::new(
        evaluate_flag_uc.clone(),
        get_flag_uc.clone(),
        create_flag_uc.clone(),
        update_flag_uc.clone(),
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

// --- InMemoryFeatureFlagRepository ---

struct InMemoryFeatureFlagRepository {
    flags: tokio::sync::RwLock<HashMap<String, FeatureFlag>>,
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
