use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod infrastructure;
mod usecase;

use adapter::handler::{self, AppState};
use infrastructure::database::DatabaseConfig;
use infrastructure::kafka_producer::{KafkaConfig, KafkaProducer};
use infrastructure::keycloak_client::{KeycloakClient, KeycloakConfig};

/// アプリケーション設定。
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
}

#[derive(Debug, Clone, serde::Deserialize)]
struct JwtConfig {
    issuer: String,
    audience: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logger
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    // Config
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let config_content = std::fs::read_to_string(&config_path)?;
    let cfg: Config = serde_yaml::from_str(&config_content)?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting auth server"
    );

    // Token verifier (stub for now, real JWKS verifier requires network)
    let token_verifier: Arc<dyn infrastructure::TokenVerifier> =
        Arc::new(StubTokenVerifier);

    // User repository (Keycloak client or stub)
    let user_repo: Arc<dyn domain::repository::UserRepository> = if let Some(kc_config) = cfg.keycloak {
        Arc::new(KeycloakClient::new(kc_config))
    } else {
        Arc::new(StubUserRepository)
    };

    // Audit log repository (in-memory for dev, PostgreSQL for prod)
    let audit_repo: Arc<dyn domain::repository::AuditLogRepository> =
        Arc::new(InMemoryAuditLogRepository::new());

    // AppState
    let state = AppState::new(
        token_verifier,
        user_repo,
        audit_repo,
        cfg.auth.jwt.issuer,
        cfg.auth.jwt.audience,
    );

    // Router
    let app = handler::router(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

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

    async fn get_roles(
        &self,
        user_id: &str,
    ) -> anyhow::Result<domain::entity::user::UserRoles> {
        anyhow::bail!("stub user repository: user not found: {}", user_id)
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
    async fn create(
        &self,
        log: &domain::entity::audit_log::AuditLog,
    ) -> anyhow::Result<()> {
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
                    if log.recorded_at < *from {
                        return false;
                    }
                }
                if let Some(ref to) = params.to {
                    if log.recorded_at > *to {
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

        filtered = filtered
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        Ok((filtered, total))
    }
}
