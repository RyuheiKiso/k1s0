use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod infrastructure;
mod usecase;

use adapter::handler::{self, AppState, ValidateTokenUseCase};
use adapter::repository::app_postgres::AppPostgresRepository;
use adapter::repository::download_stats_postgres::DownloadStatsPostgresRepository;
use adapter::repository::version_postgres::VersionPostgresRepository;
use infrastructure::database::DatabaseConfig;
use infrastructure::s3_client::S3Client;

/// Application configuration.
#[derive(Debug, Clone, serde::Deserialize)]
struct Config {
    app: AppConfig,
    server: ServerConfig,
    #[serde(default)]
    observability: ObservabilityConfig,
    auth: AuthConfig,
    #[serde(default)]
    database: Option<DatabaseConfig>,
    #[serde(default)]
    s3: S3Config,
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

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Deserialize)]
struct ServerConfig {
    #[serde(default = "default_host")]
    host: String,
    #[serde(default = "default_port")]
    port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

#[allow(dead_code)]
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
#[allow(dead_code)]
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
    false
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

#[derive(Debug, Clone, serde::Deserialize)]
struct AuthConfig {
    jwt: JwtConfig,
    #[serde(default)]
    jwks: Option<JwksConfig>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct JwtConfig {
    issuer: String,
    audience: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct JwksConfig {
    url: String,
    #[serde(default = "default_cache_ttl_secs")]
    cache_ttl_secs: u64,
}

fn default_cache_ttl_secs() -> u64 {
    600
}

#[derive(Debug, Clone, serde::Deserialize)]
struct S3Config {
    #[serde(default = "default_s3_endpoint")]
    endpoint: String,
    #[serde(default = "default_s3_bucket")]
    bucket: String,
    #[serde(default = "default_s3_region")]
    region: String,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            endpoint: default_s3_endpoint(),
            bucket: default_s3_bucket(),
            region: default_s3_region(),
        }
    }
}

fn default_s3_endpoint() -> String {
    "http://localhost:7480".to_string()
}

fn default_s3_bucket() -> String {
    "app-registry".to_string()
}

fn default_s3_region() -> String {
    "us-east-1".to_string()
}

fn parse_pool_duration(input: &str) -> Option<std::time::Duration> {
    let value = input.trim();
    if value.is_empty() {
        return None;
    }
    if let Some(raw) = value.strip_suffix("ms") {
        return raw
            .trim()
            .parse::<u64>()
            .ok()
            .map(std::time::Duration::from_millis);
    }
    if let Some(raw) = value.strip_suffix('s') {
        return raw
            .trim()
            .parse::<u64>()
            .ok()
            .map(std::time::Duration::from_secs);
    }
    if let Some(raw) = value.strip_suffix('m') {
        return raw
            .trim()
            .parse::<u64>()
            .ok()
            .map(|v| std::time::Duration::from_secs(v * 60));
    }
    if let Some(raw) = value.strip_suffix('h') {
        return raw
            .trim()
            .parse::<u64>()
            .ok()
            .map(|v| std::time::Duration::from_secs(v * 60 * 60));
    }
    value
        .parse::<u64>()
        .ok()
        .map(std::time::Duration::from_secs)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let config_content = std::fs::read_to_string(&config_path)?;
    let cfg: Config = serde_yaml::from_str(&config_content)?;

    // Telemetry
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-app-registry".to_string(),
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
        "starting app-registry server"
    );

    // Token verifier (JWKS verifier if configured, stub otherwise)
    let token_verifier: Arc<dyn infrastructure::TokenVerifier> =
        if let Some(jwks_config) = &cfg.auth.jwks {
            let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
                &jwks_config.url,
                &cfg.auth.jwt.issuer,
                &cfg.auth.jwt.audience,
                std::time::Duration::from_secs(jwks_config.cache_ttl_secs),
            ));
            Arc::new(infrastructure::JwksVerifierAdapter::new(jwks_verifier))
        } else {
            Arc::new(StubTokenVerifier)
        };

    // Database pool (optional)
    let db_pool = if let Some(ref db_config) = cfg.database {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_config.connection_url());
        info!("connecting to database");
        let lifetime = parse_pool_duration(&db_config.conn_max_lifetime)
            .unwrap_or_else(|| std::time::Duration::from_secs(300));
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(db_config.max_open_conns)
            .min_connections(db_config.max_idle_conns.min(db_config.max_open_conns))
            .idle_timeout(Some(lifetime))
            .max_lifetime(Some(lifetime))
            .connect(&url)
            .await?;
        info!("database connection pool established");
        Some(pool)
    } else if let Ok(url) = std::env::var("DATABASE_URL") {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(25)
            .min_connections(5)
            .idle_timeout(Some(std::time::Duration::from_secs(300)))
            .max_lifetime(Some(std::time::Duration::from_secs(300)))
            .connect(&url)
            .await?;
        info!("database connection pool established from DATABASE_URL");
        Some(pool)
    } else {
        info!("no database configured, using in-memory/stub repositories");
        None
    };

    // S3 client (for Ceph RGW presigned URLs)
    let s3_client = Arc::new(
        S3Client::new(&cfg.s3.endpoint, &cfg.s3.bucket, &cfg.s3.region).await,
    );

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-app-registry"));

    // Repositories
    let app_repo: Arc<dyn domain::repository::AppRepository> = if let Some(ref pool) = db_pool {
        Arc::new(AppPostgresRepository::with_metrics(
            pool.clone(),
            metrics.clone(),
        ))
    } else {
        Arc::new(StubAppRepository)
    };

    let version_repo: Arc<dyn domain::repository::VersionRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(VersionPostgresRepository::with_metrics(
                pool.clone(),
                metrics.clone(),
            ))
        } else {
            Arc::new(StubVersionRepository)
        };

    let download_stats_repo: Arc<dyn domain::repository::DownloadStatsRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(DownloadStatsPostgresRepository::with_metrics(
                pool.clone(),
                metrics.clone(),
            ))
        } else {
            Arc::new(StubDownloadStatsRepository)
        };

    // Use cases
    let list_apps_uc = Arc::new(usecase::ListAppsUseCase::new(app_repo.clone()));
    let get_app_uc = Arc::new(usecase::GetAppUseCase::new(app_repo.clone()));
    let list_versions_uc = Arc::new(usecase::ListVersionsUseCase::new(version_repo.clone()));
    let create_version_uc = Arc::new(usecase::CreateVersionUseCase::new(version_repo.clone()));
    let delete_version_uc = Arc::new(usecase::DeleteVersionUseCase::new(version_repo.clone()));
    let get_latest_uc = Arc::new(usecase::GetLatestUseCase::new(version_repo.clone()));
    let generate_download_url_uc = Arc::new(usecase::GenerateDownloadUrlUseCase::new(
        version_repo.clone(),
        download_stats_repo.clone(),
        s3_client.clone(),
    ));

    let validate_token_uc = Arc::new(ValidateTokenUseCase::new(
        token_verifier,
        cfg.auth.jwt.issuer,
        cfg.auth.jwt.audience,
    ));

    // AppState
    let state = AppState {
        list_apps_uc,
        get_app_uc,
        list_versions_uc,
        create_version_uc,
        delete_version_uc,
        get_latest_uc,
        generate_download_url_uc,
        validate_token_uc,
        metrics: metrics.clone(),
        db_pool,
    };

    // Router
    let app = handler::router(state).layer(k1s0_telemetry::MetricsLayer::new(metrics));

    // REST server
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// --- Stub implementations for dev mode ---

struct StubTokenVerifier;

#[async_trait::async_trait]
impl infrastructure::TokenVerifier for StubTokenVerifier {
    async fn verify_token(
        &self,
        _token: &str,
    ) -> anyhow::Result<domain::entity::claims::Claims> {
        anyhow::bail!("stub token verifier: not implemented")
    }
}

struct StubAppRepository;

#[async_trait::async_trait]
impl domain::repository::AppRepository for StubAppRepository {
    async fn list(
        &self,
        _category: Option<&str>,
        _search: Option<&str>,
    ) -> anyhow::Result<Vec<domain::entity::app::App>> {
        Ok(vec![])
    }

    async fn find_by_id(
        &self,
        _id: &str,
    ) -> anyhow::Result<Option<domain::entity::app::App>> {
        Ok(None)
    }
}

struct StubVersionRepository;

#[async_trait::async_trait]
impl domain::repository::VersionRepository for StubVersionRepository {
    async fn list_by_app(
        &self,
        _app_id: &str,
    ) -> anyhow::Result<Vec<domain::entity::version::AppVersion>> {
        Ok(vec![])
    }

    async fn find_latest(
        &self,
        _app_id: &str,
        _platform: &domain::entity::platform::Platform,
        _arch: &str,
    ) -> anyhow::Result<Option<domain::entity::version::AppVersion>> {
        Ok(None)
    }

    async fn create(
        &self,
        version: &domain::entity::version::AppVersion,
    ) -> anyhow::Result<domain::entity::version::AppVersion> {
        Ok(version.clone())
    }

    async fn delete(
        &self,
        _app_id: &str,
        _version: &str,
        _platform: &domain::entity::platform::Platform,
        _arch: &str,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

struct StubDownloadStatsRepository;

#[async_trait::async_trait]
impl domain::repository::DownloadStatsRepository for StubDownloadStatsRepository {
    async fn record(
        &self,
        _stat: &domain::entity::download_stat::DownloadStat,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn count_by_app(&self, _app_id: &str) -> anyhow::Result<i64> {
        Ok(0)
    }

    async fn count_by_version(&self, _app_id: &str, _version: &str) -> anyhow::Result<i64> {
        Ok(0)
    }
}
