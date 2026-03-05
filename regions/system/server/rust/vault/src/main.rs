#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tracing::info;

mod adapter;
mod domain;
mod infrastructure;
mod proto;
mod usecase;

use adapter::grpc::VaultGrpcService;
use adapter::handler::{self, AppState};
use domain::entity::secret::Secret;
use domain::repository::{AccessLogRepository, SecretStore};
use infrastructure::database::DatabaseConfig;
use infrastructure::encryption::MasterKey;
use infrastructure::kafka_producer::KafkaConfig;

#[derive(Debug, Clone, serde::Deserialize)]
struct AuthConfig {
    jwks_url: String,
    issuer: String,
    audience: String,
    #[serde(default = "default_jwks_cache_ttl_secs")]
    jwks_cache_ttl_secs: u64,
}

fn default_jwks_cache_ttl_secs() -> u64 {
    3600
}

/// Application configuration.
#[derive(Debug, Clone, serde::Deserialize)]
struct Config {
    app: AppConfig,
    server: ServerConfig,
    #[serde(default)]
    observability: ObservabilityConfig,
    #[serde(default)]
    auth: Option<AuthConfig>,
    #[serde(default)]
    database: Option<DatabaseConfig>,
    #[serde(default)]
    kafka: Option<KafkaConfig>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct AppConfig {
    name: String,
    #[serde(default = "default_version")]
    version: String,
    #[serde(default = "default_environment")]
    environment: String,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_environment() -> String {
    "dev".to_string()
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ServerConfig {
    #[serde(default = "default_host")]
    host: String,
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_grpc_port")]
    grpc_port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8090
}

fn default_grpc_port() -> u16 {
    50051
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ObservabilityConfig {
    #[serde(default)]
    log: LogConfig,
    #[serde(default)]
    trace: TraceConfig,
    #[serde(default)]
    metrics: MetricsConfig,
}
impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            log: LogConfig::default(),
            trace: TraceConfig::default(),
            metrics: MetricsConfig::default(),
        }
    }
}
#[derive(Debug, Clone, serde::Deserialize)]
struct LogConfig {
    #[serde(default = "default_log_level")]
    level: String,
    #[serde(default = "default_log_format")]
    format: String,
}
impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}
#[derive(Debug, Clone, serde::Deserialize)]
struct TraceConfig {
    #[serde(default = "default_trace_enabled")]
    enabled: bool,
    #[serde(default = "default_trace_endpoint")]
    endpoint: String,
    #[serde(default = "default_trace_sample_rate")]
    sample_rate: f64,
}
impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            enabled: default_trace_enabled(),
            endpoint: default_trace_endpoint(),
            sample_rate: default_trace_sample_rate(),
        }
    }
}
#[derive(Debug, Clone, serde::Deserialize)]
struct MetricsConfig {
    #[serde(default = "default_metrics_enabled")]
    enabled: bool,
    #[serde(default = "default_metrics_path")]
    path: String,
}
impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: default_metrics_enabled(),
            path: default_metrics_path(),
        }
    }
}
fn default_trace_enabled() -> bool {
    true
}
fn default_trace_endpoint() -> String {
    "http://otel-collector.observability:4317".to_string()
}
fn default_trace_sample_rate() -> f64 {
    1.0
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_format() -> String {
    "json".to_string()
}
fn default_metrics_enabled() -> bool {
    true
}
fn default_metrics_path() -> String {
    "/metrics".to_string()
}

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
    let config_content = std::fs::read_to_string(&config_path)?;
    let cfg: Config = serde_yaml::from_str(&config_content)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-vault-server".to_string(),
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
        "starting vault server"
    );

    // MasterKey for encryption
    let master_key = Arc::new(MasterKey::from_env()?);
    info!("master key loaded");

    // Cache (max 10000 entries, TTL 48 min = 2880 seconds)
    let secret_cache = Arc::new(infrastructure::cache::SecretCache::new(10_000, 2880));

    // Secret store + audit repository (Vault KV v2 / PG / InMemory)
    let vault_addr = std::env::var("VAULT_ADDR").ok();
    let vault_token = std::env::var("VAULT_TOKEN").ok();

    let (secret_store_base, audit_repo, db_pool): (
        Arc<dyn SecretStore>,
        Arc<dyn AccessLogRepository>,
        Option<sqlx::PgPool>,
    ) = if let (Some(addr), Some(token)) = (vault_addr, vault_token) {
        info!(vault_addr = %addr, "connecting to HashiCorp Vault KV v2");
        let vault_client = adapter::gateway::VaultKvClient::new(&addr, &token)?;
        let vault_client = Arc::new(vault_client);
        let store: Arc<dyn SecretStore> =
            Arc::new(adapter::repository::vault_secret_store::VaultSecretStore::new(vault_client));
        let audit: Arc<dyn AccessLogRepository> = Arc::new(NoopAccessLogRepository);
        info!("HashiCorp Vault backend ready");
        (store, audit, None)
    } else if let Some(ref db_config) = cfg.database {
        info!("connecting to PostgreSQL for vault storage");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(db_config.max_open_conns)
            .min_connections(db_config.max_idle_conns.min(db_config.max_open_conns))
            .max_lifetime(parse_pool_duration(&db_config.conn_max_lifetime))
            .connect(&db_config.connection_url())
            .await?;
        let pool = Arc::new(pool);
        info!("PostgreSQL connection pool established");

        let store: Arc<dyn SecretStore> = Arc::new(
            adapter::repository::secret_store_postgres::SecretStorePostgresRepository::new(
                pool.clone(),
                master_key.clone(),
            ),
        );
        let audit: Arc<dyn AccessLogRepository> = Arc::new(
            adapter::repository::access_log_postgres::AccessLogPostgresRepository::new(
                pool.clone(),
            ),
        );

        (store, audit, Some(pool.as_ref().clone()))
    } else {
        info!("using in-memory secret store (dev mode)");
        let store: Arc<dyn SecretStore> = Arc::new(InMemorySecretStore::new());
        let audit: Arc<dyn AccessLogRepository> = Arc::new(NoopAccessLogRepository);
        (store, audit, None)
    };
    let secret_store: Arc<dyn SecretStore> = Arc::new(
        adapter::repository::cached_secret_store::CachedSecretStore::new(
            secret_store_base,
            secret_cache.clone(),
        ),
    );

    // Kafka event publisher
    let event_publisher: Arc<dyn infrastructure::kafka_producer::VaultEventPublisher> =
        if let Some(ref kafka_config) = cfg.kafka {
            info!("connecting to Kafka for vault events");
            let producer = infrastructure::kafka_producer::KafkaProducer::new(kafka_config)?;
            info!(topic = producer.topic(), "Kafka producer ready");
            Arc::new(producer)
        } else {
            info!("using noop vault event publisher (dev mode)");
            Arc::new(infrastructure::kafka_producer::NoopVaultEventPublisher)
        };

    // Use cases
    let get_secret_uc = Arc::new(usecase::GetSecretUseCase::new(
        secret_store.clone(),
        audit_repo.clone(),
        event_publisher.clone(),
    ));
    let set_secret_uc = Arc::new(usecase::SetSecretUseCase::new(
        secret_store.clone(),
        audit_repo.clone(),
        event_publisher.clone(),
    ));
    let rotate_secret_uc = Arc::new(usecase::RotateSecretUseCase::new(
        get_secret_uc.clone(),
        set_secret_uc.clone(),
        event_publisher.clone(),
    ));
    let delete_secret_uc = Arc::new(usecase::DeleteSecretUseCase::new(
        secret_store.clone(),
        audit_repo.clone(),
        event_publisher.clone(),
    ));
    let list_secrets_uc = Arc::new(usecase::ListSecretsUseCase::new(secret_store));
    let list_audit_logs_uc = Arc::new(usecase::ListAuditLogsUseCase::new(audit_repo));

    // gRPC service
    let vault_grpc_svc = Arc::new(VaultGrpcService::new(
        get_secret_uc.clone(),
        set_secret_uc.clone(),
        rotate_secret_uc.clone(),
        delete_secret_uc.clone(),
        list_secrets_uc.clone(),
        list_audit_logs_uc.clone(),
    ));

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-vault-server"));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = if let Some(ref auth_cfg) = cfg.auth {
        info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for vault-server");
        let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
            &auth_cfg.jwks_url,
            &auth_cfg.issuer,
            &auth_cfg.audience,
            std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
        ));
        Some(adapter::middleware::auth::VaultAuthState {
            verifier: jwks_verifier,
        })
    } else {
        info!("no auth configured, vault-server running without authentication");
        None
    };

    // SPIFFE access policies (empty = permissive mode; loaded from DB in production)
    let spiffe_state = adapter::middleware::spiffe::SpiffeAuthState {
        policies: Arc::new(vec![]),
    };

    // AppState (REST)
    let mut state = AppState {
        get_secret_uc,
        set_secret_uc,
        rotate_secret_uc,
        delete_secret_uc,
        list_secrets_uc,
        list_audit_logs_uc,
        db_pool,
        metrics: metrics.clone(),
        auth_state: None,
        spiffe_state: Some(spiffe_state),
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    // REST Router
    let app = handler::router(state).layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    // gRPC tonic service
    use proto::k1s0::system::vault::v1::vault_service_server::VaultServiceServer;

    let vault_tonic = adapter::grpc::VaultServiceTonic::new(vault_grpc_svc);

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(VaultServiceServer::new(vault_tonic))
            .serve(grpc_addr)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    // REST server
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
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

// --- InMemory SecretStore (dev fallback) ---

struct InMemorySecretStore {
    secrets: tokio::sync::RwLock<HashMap<String, Secret>>,
}

impl InMemorySecretStore {
    fn new() -> Self {
        Self {
            secrets: tokio::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl domain::repository::SecretStore for InMemorySecretStore {
    async fn get(&self, path: &str, version: Option<i64>) -> anyhow::Result<Secret> {
        let store = self.secrets.read().await;
        let secret = store
            .get(path)
            .ok_or_else(|| anyhow::anyhow!("secret not found: {}", path))?;
        if let Some(v) = version {
            secret
                .versions
                .iter()
                .find(|sv| sv.version == v && !sv.destroyed)
                .ok_or_else(|| anyhow::anyhow!("version {} not found", v))?;
        }
        Ok(secret.clone())
    }

    async fn set(&self, path: &str, data: HashMap<String, String>) -> anyhow::Result<i64> {
        let mut store = self.secrets.write().await;
        let version = if let Some(existing) = store.get_mut(path) {
            let updated = existing.clone().update(data);
            let v = updated.current_version;
            *existing = updated;
            v
        } else {
            let secret = Secret::new(path.to_string(), data);
            store.insert(path.to_string(), secret);
            1
        };
        Ok(version)
    }

    async fn delete(&self, path: &str, versions: Vec<i64>) -> anyhow::Result<()> {
        let mut store = self.secrets.write().await;
        if let Some(secret) = store.get_mut(path) {
            for sv in &mut secret.versions {
                if versions.is_empty() || versions.contains(&sv.version) {
                    sv.destroyed = true;
                }
            }
        }
        Ok(())
    }

    async fn list(&self, path_prefix: &str) -> anyhow::Result<Vec<String>> {
        let store = self.secrets.read().await;
        Ok(store
            .keys()
            .filter(|k| k.starts_with(path_prefix))
            .cloned()
            .collect())
    }

    async fn exists(&self, path: &str) -> anyhow::Result<bool> {
        Ok(self.secrets.read().await.contains_key(path))
    }
}

// --- Noop AccessLogRepository (dev fallback) ---

struct NoopAccessLogRepository;

#[async_trait::async_trait]
impl domain::repository::AccessLogRepository for NoopAccessLogRepository {
    async fn record(
        &self,
        _log: &domain::entity::access_log::SecretAccessLog,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn list(
        &self,
        _offset: u32,
        _limit: u32,
    ) -> anyhow::Result<Vec<domain::entity::access_log::SecretAccessLog>> {
        Ok(vec![])
    }
}
