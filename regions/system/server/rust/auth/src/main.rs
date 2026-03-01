#![allow(dead_code, unused_imports)]

use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod infrastructure;
mod proto;
mod usecase;

use adapter::grpc::{AuditGrpcService, AuthGrpcService};
use adapter::handler::{self, AppState};
use adapter::repository::api_key_postgres::ApiKeyPostgresRepository;
use adapter::repository::audit_log_postgres::AuditLogPostgresRepository;
use adapter::repository::cached_user_repository::CachedUserRepository;
use adapter::repository::user_postgres::UserPostgresRepository;
use infrastructure::database::DatabaseConfig;
use infrastructure::kafka_producer::KafkaConfig;
use infrastructure::keycloak_client::{KeycloakClient, KeycloakConfig};
use infrastructure::user_cache::UserCache;

/// Application configuration.
#[derive(Debug, Clone, serde::Deserialize)]
struct Config {
    app: AppConfig,
    server: ServerConfig,
    auth: AuthConfig,
    #[serde(default)]
    database: Option<DatabaseConfig>,
    #[serde(default)]
    kafka: Option<KafkaConfig>,
    #[serde(default)]
    keycloak: Option<KeycloakConfig>,
    #[serde(default)]
    permission_cache: PermissionCacheConfig,
    #[serde(default)]
    audit: AuditConfig,
    #[serde(default)]
    keycloak_admin: KeycloakAdminConfig,
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
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
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

/// Permission cache configuration.
#[derive(Debug, Clone, serde::Deserialize)]
struct PermissionCacheConfig {
    #[serde(default = "default_permission_cache_ttl_secs")]
    ttl_secs: u64,
    #[serde(default = "default_refresh_on_miss")]
    refresh_on_miss: bool,
}

impl Default for PermissionCacheConfig {
    fn default() -> Self {
        Self {
            ttl_secs: default_permission_cache_ttl_secs(),
            refresh_on_miss: default_refresh_on_miss(),
        }
    }
}

fn default_permission_cache_ttl_secs() -> u64 {
    300
}

fn default_refresh_on_miss() -> bool {
    true
}

/// Audit configuration.
#[derive(Debug, Clone, serde::Deserialize)]
struct AuditConfig {
    #[serde(default)]
    kafka_enabled: bool,
    #[serde(default = "default_retention_days")]
    retention_days: u32,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            kafka_enabled: false,
            retention_days: default_retention_days(),
        }
    }
}

fn default_retention_days() -> u32 {
    365
}

/// Keycloak admin client configuration.
#[derive(Debug, Clone, serde::Deserialize)]
struct KeycloakAdminConfig {
    #[serde(default = "default_keycloak_admin_token_cache_ttl_secs")]
    token_cache_ttl_secs: u64,
}

impl Default for KeycloakAdminConfig {
    fn default() -> Self {
        Self {
            token_cache_ttl_secs: default_keycloak_admin_token_cache_ttl_secs(),
        }
    }
}

fn default_keycloak_admin_token_cache_ttl_secs() -> u64 {
    300
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Telemetry
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-auth-server".to_string(),
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
    let mut cfg: Config = serde_yaml::from_str(&config_content)?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting auth server"
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
        info!(url = %url.replace(|c: char| c == ':' && url.contains("@"), "*"), "connecting to database");
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
        info!("no database configured, using in-memory/stub repositories");
        None
    };

    // Keycloak health check URL (captured before take())
    let keycloak_health_url = cfg
        .keycloak
        .as_ref()
        .map(|kc| format!("{}/realms/{}", kc.base_url, kc.realm));

    // JWKS proxy provider (Keycloak certs -> auth-server /jwks)
    let jwks_provider = cfg.keycloak.as_ref().map(|kc| {
        let url = format!(
            "{}/realms/{}/protocol/openid-connect/certs",
            kc.base_url, kc.realm
        );
        let ttl_secs = cfg
            .auth
            .jwks
            .as_ref()
            .map(|j| j.cache_ttl_secs)
            .unwrap_or(default_cache_ttl_secs());
        infrastructure::jwks_provider::JwksProvider::new(url, std::time::Duration::from_secs(ttl_secs))
    });

    // Metrics (shared across layers and repositories)
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-auth-server"));

    // User cache (max 5000 entries, TTL 300 seconds)
    let user_cache = Arc::new(UserCache::new(5000, 300));

    // User repository (PostgreSQL > Keycloak > Stub)
    let keycloak_config = cfg.keycloak.take();
    let user_repo: Arc<dyn domain::repository::UserRepository> = if let Some(ref pool) = db_pool {
        let inner: Arc<dyn domain::repository::UserRepository> =
            Arc::new(UserPostgresRepository::with_metrics(
                pool.clone(),
                metrics.clone(),
            ));
        Arc::new(CachedUserRepository::with_metrics(
            inner,
            user_cache,
            metrics.clone(),
        ))
    } else if let Some(kc_config) = keycloak_config {
        let inner: Arc<dyn domain::repository::UserRepository> =
            Arc::new(KeycloakClient::new(kc_config));
        Arc::new(CachedUserRepository::with_metrics(
            inner,
            user_cache,
            metrics.clone(),
        ))
    } else {
        Arc::new(StubUserRepository)
    };

    // Audit log repository (PostgreSQL or in-memory)
    let audit_repo: Arc<dyn domain::repository::AuditLogRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(AuditLogPostgresRepository::with_metrics(
                pool.clone(),
                metrics.clone(),
            ))
        } else {
            Arc::new(InMemoryAuditLogRepository::new())
        };

    // API key repository (PostgreSQL or in-memory)
    let api_key_repo: Arc<dyn domain::repository::ApiKeyRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(ApiKeyPostgresRepository::with_metrics(
                pool.clone(),
                metrics.clone(),
            ))
        } else {
            Arc::new(InMemoryApiKeyRepository::new())
        };

    // Kafka producer (conditional on audit.kafka_enabled)
    let kafka_publisher: Option<Arc<dyn infrastructure::kafka_producer::AuditEventPublisher>> =
        if cfg.audit.kafka_enabled {
            if let Some(ref kafka_config) = cfg.kafka {
                match infrastructure::kafka_producer::KafkaProducer::new(kafka_config) {
                    Ok(producer) => {
                        info!("Kafka audit event publisher enabled");
                        Some(Arc::new(producer))
                    }
                    Err(e) => {
                        tracing::warn!("Failed to create Kafka producer, audit events will not be published: {}", e);
                        None
                    }
                }
            } else {
                tracing::warn!("audit.kafka_enabled=true but no kafka config found");
                None
            }
        } else {
            info!("Kafka audit event publishing disabled");
            None
        };

    // --- gRPC Service ---
    let validate_token_uc = Arc::new(usecase::ValidateTokenUseCase::new(
        token_verifier.clone(),
        cfg.auth.jwt.issuer.clone(),
        cfg.auth.jwt.audience.clone(),
    ));
    let get_user_uc = Arc::new(usecase::GetUserUseCase::new(user_repo.clone()));
    let get_user_roles_uc = Arc::new(usecase::GetUserRolesUseCase::new(user_repo.clone()));
    let list_users_uc = Arc::new(usecase::ListUsersUseCase::new(user_repo.clone()));
    let record_audit_log_uc = Arc::new(if let Some(ref publisher) = kafka_publisher {
        usecase::RecordAuditLogUseCase::with_publisher(audit_repo.clone(), publisher.clone())
    } else {
        usecase::RecordAuditLogUseCase::new(audit_repo.clone())
    });
    let search_audit_logs_uc = Arc::new(usecase::SearchAuditLogsUseCase::new(audit_repo.clone()));

    // AppState (REST handler 用)
    let state = AppState::new(
        token_verifier,
        user_repo,
        audit_repo,
        api_key_repo,
        cfg.auth.jwt.issuer,
        cfg.auth.jwt.audience,
        db_pool.clone(),
        keycloak_health_url,
        jwks_provider,
    );

    let auth_grpc_svc = Arc::new(AuthGrpcService::new(
        validate_token_uc,
        get_user_uc,
        get_user_roles_uc,
        list_users_uc,
    ));
    let audit_grpc_svc = Arc::new(AuditGrpcService::new(
        record_audit_log_uc,
        search_audit_logs_uc,
    ));

    use proto::k1s0::system::auth::v1::{
        audit_service_server::AuditServiceServer, auth_service_server::AuthServiceServer,
    };

    let auth_tonic = adapter::grpc::AuthServiceTonic::new(auth_grpc_svc);
    let audit_tonic = adapter::grpc::AuditServiceTonic::new(audit_grpc_svc);

    // Router
    let app = handler::router(state).layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    // gRPC server (port 50051)
    let grpc_addr: SocketAddr = ([0, 0, 0, 0], 50051).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(AuthServiceServer::new(auth_tonic))
            .add_service(AuditServiceServer::new(audit_tonic))
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

// --- Stub implementations for dev mode ---

struct StubTokenVerifier;

#[async_trait::async_trait]
impl infrastructure::TokenVerifier for StubTokenVerifier {
    async fn verify_token(&self, _token: &str) -> anyhow::Result<domain::entity::Claims> {
        anyhow::bail!("stub token verifier: not implemented")
    }
}

struct StubUserRepository;

#[async_trait::async_trait]
impl domain::repository::UserRepository for StubUserRepository {
    async fn find_by_id(&self, user_id: &str) -> anyhow::Result<domain::entity::user::User> {
        anyhow::bail!("stub user repository: user not found: {}", user_id)
    }

    async fn list(
        &self,
        page: i32,
        page_size: i32,
        _search: Option<String>,
        _enabled: Option<bool>,
    ) -> anyhow::Result<domain::entity::user::UserListResult> {
        Ok(domain::entity::user::UserListResult {
            users: vec![],
            pagination: domain::entity::user::Pagination {
                total_count: 0,
                page,
                page_size,
                has_next: false,
            },
        })
    }

    async fn get_roles(&self, user_id: &str) -> anyhow::Result<domain::entity::user::UserRoles> {
        anyhow::bail!("stub user repository: user not found: {}", user_id)
    }
}

/// InMemoryApiKeyRepository は開発用のインメモリ API キーリポジトリ。
struct InMemoryApiKeyRepository {
    keys: tokio::sync::RwLock<Vec<domain::entity::api_key::ApiKey>>,
}

impl InMemoryApiKeyRepository {
    fn new() -> Self {
        Self {
            keys: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl domain::repository::ApiKeyRepository for InMemoryApiKeyRepository {
    async fn create(&self, api_key: &domain::entity::api_key::ApiKey) -> anyhow::Result<()> {
        self.keys.write().await.push(api_key.clone());
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: uuid::Uuid,
    ) -> anyhow::Result<Option<domain::entity::api_key::ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys.iter().find(|k| k.id == id).cloned())
    }

    async fn find_by_prefix(
        &self,
        prefix: &str,
    ) -> anyhow::Result<Option<domain::entity::api_key::ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys.iter().find(|k| k.prefix == prefix).cloned())
    }

    async fn list_by_tenant(
        &self,
        tenant_id: &str,
    ) -> anyhow::Result<Vec<domain::entity::api_key::ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys
            .iter()
            .filter(|k| k.tenant_id == tenant_id)
            .cloned()
            .collect())
    }

    async fn revoke(&self, id: uuid::Uuid) -> anyhow::Result<()> {
        let mut keys = self.keys.write().await;
        if let Some(key) = keys.iter_mut().find(|k| k.id == id) {
            key.revoked = true;
            Ok(())
        } else {
            anyhow::bail!("api key not found: {}", id)
        }
    }

    async fn delete(&self, id: uuid::Uuid) -> anyhow::Result<()> {
        let mut keys = self.keys.write().await;
        let len_before = keys.len();
        keys.retain(|k| k.id != id);
        if keys.len() == len_before {
            anyhow::bail!("api key not found: {}", id)
        }
        Ok(())
    }
}

/// InMemoryAuditLogRepository は開発用のインメモリ監査ログリポジトリ。
struct InMemoryAuditLogRepository {
    logs: tokio::sync::RwLock<Vec<domain::entity::audit_log::AuditLog>>,
}

impl InMemoryAuditLogRepository {
    fn new() -> Self {
        Self {
            logs: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl domain::repository::AuditLogRepository for InMemoryAuditLogRepository {
    async fn create(&self, log: &domain::entity::audit_log::AuditLog) -> anyhow::Result<()> {
        self.logs.write().await.push(log.clone());
        Ok(())
    }

    async fn search(
        &self,
        params: &domain::entity::audit_log::AuditLogSearchParams,
    ) -> anyhow::Result<(Vec<domain::entity::audit_log::AuditLog>, i64)> {
        let logs = self.logs.read().await;
        let mut filtered: Vec<_> = logs
            .iter()
            .filter(|log| {
                if let Some(ref uid) = params.user_id {
                    if log.user_id != *uid {
                        return false;
                    }
                }
                if let Some(ref et) = params.event_type {
                    if log.event_type != *et {
                        return false;
                    }
                }
                if let Some(ref r) = params.result {
                    if log.result != *r {
                        return false;
                    }
                }
                if let Some(ref from) = params.from {
                    if log.created_at < *from {
                        return false;
                    }
                }
                if let Some(ref to) = params.to {
                    if log.created_at > *to {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        let total = filtered.len() as i64;
        let offset = ((params.page - 1) * params.page_size) as usize;
        let limit = params.page_size as usize;

        filtered = filtered.into_iter().skip(offset).take(limit).collect();

        Ok((filtered, total))
    }
}
