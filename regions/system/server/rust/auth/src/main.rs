use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod infrastructure;
mod usecase;

use adapter::grpc::{AuditGrpcService, AuthGrpcService};
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
    3600
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
    };
    k1s0_telemetry::init_telemetry(&telemetry_cfg)
        .expect("failed to init telemetry");

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

    // User repository (Keycloak client or stub)
    let user_repo: Arc<dyn domain::repository::UserRepository> = if let Some(kc_config) = cfg.keycloak {
        Arc::new(KeycloakClient::new(kc_config))
    } else {
        Arc::new(StubUserRepository)
    };

    // Audit log repository (in-memory for dev, PostgreSQL for prod)
    let audit_repo: Arc<dyn domain::repository::AuditLogRepository> =
        Arc::new(InMemoryAuditLogRepository::new());

    // --- gRPC Service ---
    let validate_token_uc = Arc::new(usecase::ValidateTokenUseCase::new(
        token_verifier.clone(),
        cfg.auth.jwt.issuer.clone(),
        cfg.auth.jwt.audience.clone(),
    ));
    let get_user_uc = Arc::new(usecase::GetUserUseCase::new(user_repo.clone()));
    let list_users_uc = Arc::new(usecase::ListUsersUseCase::new(user_repo.clone()));
    let record_audit_log_uc = Arc::new(usecase::RecordAuditLogUseCase::new(audit_repo.clone()));
    let search_audit_logs_uc = Arc::new(usecase::SearchAuditLogsUseCase::new(audit_repo.clone()));

    // AppState (REST handler 用)
    let state = AppState::new(
        token_verifier,
        user_repo,
        audit_repo,
        cfg.auth.jwt.issuer,
        cfg.auth.jwt.audience,
    );

    let _auth_grpc_svc = AuthGrpcService::new(
        validate_token_uc,
        get_user_uc,
        list_users_uc,
    );
    let _audit_grpc_svc = AuditGrpcService::new(
        record_audit_log_uc,
        search_audit_logs_uc,
    );

    // Router
    let app = handler::router(state);

    // gRPC server (port 50051)
    let grpc_addr: SocketAddr = ([0, 0, 0, 0], 50051).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_future = async move {
        // TODO: tonic::transport::Server::builder()
        //     .add_service(AuthServiceServer::new(auth_grpc_svc))
        //     .add_service(AuditServiceServer::new(audit_grpc_svc))
        //     .serve(grpc_addr)
        //     .await
        // proto 生成後に有効化。現時点ではスタブとして待機。
        let listener = tokio::net::TcpListener::bind(grpc_addr).await?;
        info!("gRPC server listening on {} (stub, awaiting tonic codegen)", grpc_addr);
        // ただリスンするだけ（proto 生成後に tonic Server に置き換え）
        loop {
            let _ = listener.accept().await;
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
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
