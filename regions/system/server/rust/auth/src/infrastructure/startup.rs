use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use super::config::{default_cache_ttl_secs, parse_pool_duration, Config};
use super::keycloak_client::KeycloakClient;
use super::keycloak_role_permission_source::KeycloakRolePermissionSource;
use super::user_cache::UserCache;
use crate::adapter;
use crate::adapter::grpc::{AuditGrpcService, AuthGrpcService};
use crate::adapter::handler::{self, AppState};
use crate::adapter::repository::api_key_postgres::ApiKeyPostgresRepository;
use crate::adapter::repository::audit_log_postgres::AuditLogPostgresRepository;
use crate::adapter::repository::cached_user_repository::CachedUserRepository;
use crate::adapter::repository::user_postgres::UserPostgresRepository;
use crate::usecase;

pub async fn run() -> anyhow::Result<()> {
    // Telemetry
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-auth-server".to_string(),
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
        "starting auth server"
    );

    // Token verifier (JWKS verifier if configured, stub otherwise)
    let token_verifier: Arc<dyn super::TokenVerifier> = if let Some(jwks_config) = &cfg.auth.jwks {
        let jwks_verifier = Arc::new(
            k1s0_auth::JwksVerifier::new(
                &jwks_config.url,
                &cfg.auth.jwt.issuer,
                &cfg.auth.jwt.audience,
                std::time::Duration::from_secs(jwks_config.cache_ttl_secs),
            )
            .expect("Failed to create JWKS verifier"),
        );
        Arc::new(super::JwksVerifierAdapter::new(jwks_verifier))
    } else {
        Arc::new(StubTokenVerifier)
    };

    // Database pool (optional)
    let db_pool = if let Some(ref db_config) = cfg.database {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_config.connection_url());
        info!(url = %url.replace(|c: char| c == ':' && url.contains("@"), "*"), "connecting to database");
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
        // infra_guard: stable サービスでは DB 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "auth",
            k1s0_server_common::InfraKind::Database,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no database configured, using in-memory/stub repositories (dev/test bypass)");
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
        super::jwks_provider::JwksProvider::new(url, std::time::Duration::from_secs(ttl_secs))
    });

    // Metrics (shared across layers and repositories)
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-auth-server"));

    // User cache (max 5000 entries, TTL 300 seconds)
    let user_cache = Arc::new(UserCache::new(5000, 300));

    // User repository (PostgreSQL > Keycloak > Stub)
    let keycloak_config = cfg.keycloak.clone();
    let user_repo: Arc<dyn crate::domain::repository::UserRepository> =
        if let Some(ref pool) = db_pool {
            let inner: Arc<dyn crate::domain::repository::UserRepository> = Arc::new(
                UserPostgresRepository::with_metrics(pool.clone(), metrics.clone()),
            );
            Arc::new(CachedUserRepository::with_metrics(
                inner,
                user_cache,
                metrics.clone(),
            ))
        } else if let Some(kc_config) = keycloak_config.clone() {
            let inner: Arc<dyn crate::domain::repository::UserRepository> =
                Arc::new(KeycloakClient::new(kc_config));
            Arc::new(CachedUserRepository::with_metrics(
                inner,
                user_cache,
                metrics.clone(),
            ))
        } else {
            Arc::new(StubUserRepository)
        };

    // Keycloak Admin API role-permission table (cached + periodic refresh)
    let role_permission_table = if let Some(kc_config) = keycloak_config {
        let source = Arc::new(KeycloakRolePermissionSource::new(
            kc_config,
            cfg.keycloak_admin.token_cache_ttl_secs,
        ));
        let table = Arc::new(crate::domain::service::RolePermissionTable::new(
            source,
            cfg.permission_cache.ttl_secs,
            2_048,
        ));

        if let Err(err) = table.sync_once().await {
            tracing::warn!(
                error = %err,
                "initial keycloak role-permission sync failed; static RBAC fallback will be used"
            );
        } else {
            info!("initial keycloak role-permission table sync completed");
        }

        table
            .clone()
            .start_background_refresh(std::time::Duration::from_secs(
                cfg.keycloak_admin.refresh_interval_secs,
            ));
        Some(table)
    } else {
        None
    };

    // Audit log repository (PostgreSQL or in-memory)
    let audit_repo: Arc<dyn crate::domain::repository::AuditLogRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(AuditLogPostgresRepository::with_metrics(
                pool.clone(),
                metrics.clone(),
            ))
        } else {
            Arc::new(InMemoryAuditLogRepository::new())
        };

    // API key repository (PostgreSQL or in-memory)
    let api_key_repo: Arc<dyn crate::domain::repository::ApiKeyRepository> =
        if let Some(ref pool) = db_pool {
            Arc::new(ApiKeyPostgresRepository::with_metrics(
                pool.clone(),
                metrics.clone(),
            ))
        } else {
            Arc::new(InMemoryApiKeyRepository::new())
        };

    // Kafka producer (conditional on audit.kafka_enabled)
    let kafka_publisher: Option<Arc<dyn super::kafka_producer::AuditEventPublisher>> =
        if cfg.audit.kafka_enabled {
            if let Some(ref kafka_config) = cfg.kafka {
                match super::kafka_producer::KafkaProducer::new(kafka_config) {
                    Ok(producer) => {
                        info!("Kafka audit event publisher enabled");
                        Some(Arc::new(producer))
                    }
                    Err(e) => {
                        tracing::warn!(
                        "Failed to create Kafka producer, audit events will not be published: {}",
                        e
                    );
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
    let check_permission_uc = Arc::new(if let Some(table) = role_permission_table.clone() {
        usecase::CheckPermissionUseCase::with_user_repo_and_role_table(user_repo.clone(), table)
    } else {
        usecase::CheckPermissionUseCase::with_user_repo(user_repo.clone())
    });
    let record_audit_log_uc = Arc::new(if let Some(ref publisher) = kafka_publisher {
        usecase::RecordAuditLogUseCase::with_publisher(audit_repo.clone(), publisher.clone())
    } else {
        usecase::RecordAuditLogUseCase::new(audit_repo.clone())
    });
    let search_audit_logs_uc = Arc::new(usecase::SearchAuditLogsUseCase::new(audit_repo.clone()));

    // AppState (REST handler 用)
    let mut state = AppState::new(
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
    state.permission_cache =
        super::permission_cache::PermissionCache::new(cfg.permission_cache.ttl_secs, 10_000);
    state.permission_cache_refresh_on_miss = cfg.permission_cache.refresh_on_miss;
    state.check_permission_uc = check_permission_uc.clone();
    state.role_permission_table = role_permission_table;

    let auth_grpc_svc = Arc::new(AuthGrpcService::new(
        validate_token_uc,
        get_user_uc,
        get_user_roles_uc,
        list_users_uc,
        check_permission_uc,
    ));
    let audit_grpc_svc = Arc::new(AuditGrpcService::new(
        record_audit_log_uc,
        search_audit_logs_uc,
    ));

    use crate::proto::k1s0::system::auth::v1::{
        audit_service_server::AuditServiceServer, auth_service_server::AuthServiceServer,
    };

    let auth_tonic = adapter::grpc::AuthServiceTonic::new(auth_grpc_svc);
    let audit_tonic = adapter::grpc::AuditServiceTonic::new(audit_grpc_svc);

    // Router
    let app = handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

    // gRPC server
    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(AuthServiceServer::new(auth_tonic))
            .add_service(AuditServiceServer::new(audit_tonic))
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

    k1s0_telemetry::shutdown();

    Ok(())
}

// --- Stub implementations for dev mode ---

struct StubTokenVerifier;

#[async_trait::async_trait]
impl super::TokenVerifier for StubTokenVerifier {
    async fn verify_token(&self, _token: &str) -> anyhow::Result<crate::domain::entity::Claims> {
        anyhow::bail!("stub token verifier: not implemented")
    }
}

struct StubUserRepository;

#[async_trait::async_trait]
impl crate::domain::repository::UserRepository for StubUserRepository {
    async fn find_by_id(&self, user_id: &str) -> anyhow::Result<crate::domain::entity::user::User> {
        anyhow::bail!("stub user repository: user not found: {}", user_id)
    }

    async fn list(
        &self,
        page: i32,
        page_size: i32,
        _search: Option<String>,
        _enabled: Option<bool>,
    ) -> anyhow::Result<crate::domain::entity::user::UserListResult> {
        Ok(crate::domain::entity::user::UserListResult {
            users: vec![],
            pagination: crate::domain::entity::user::Pagination {
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
    ) -> anyhow::Result<crate::domain::entity::user::UserRoles> {
        anyhow::bail!("stub user repository: user not found: {}", user_id)
    }
}

/// InMemoryApiKeyRepository は開発用のインメモリ API キーリポジトリ。
struct InMemoryApiKeyRepository {
    keys: tokio::sync::RwLock<Vec<crate::domain::entity::api_key::ApiKey>>,
}

impl InMemoryApiKeyRepository {
    fn new() -> Self {
        Self {
            keys: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl crate::domain::repository::ApiKeyRepository for InMemoryApiKeyRepository {
    async fn create(&self, api_key: &crate::domain::entity::api_key::ApiKey) -> anyhow::Result<()> {
        self.keys.write().await.push(api_key.clone());
        Ok(())
    }

    async fn find_by_id(
        &self,
        id: uuid::Uuid,
    ) -> anyhow::Result<Option<crate::domain::entity::api_key::ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys.iter().find(|k| k.id == id).cloned())
    }

    async fn find_by_prefix(
        &self,
        prefix: &str,
    ) -> anyhow::Result<Option<crate::domain::entity::api_key::ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys.iter().find(|k| k.prefix == prefix).cloned())
    }

    async fn list_by_tenant(
        &self,
        tenant_id: &str,
    ) -> anyhow::Result<Vec<crate::domain::entity::api_key::ApiKey>> {
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
    logs: tokio::sync::RwLock<Vec<crate::domain::entity::audit_log::AuditLog>>,
}

impl InMemoryAuditLogRepository {
    fn new() -> Self {
        Self {
            logs: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl crate::domain::repository::AuditLogRepository for InMemoryAuditLogRepository {
    async fn create(&self, log: &crate::domain::entity::audit_log::AuditLog) -> anyhow::Result<()> {
        self.logs.write().await.push(log.clone());
        Ok(())
    }

    async fn search(
        &self,
        params: &crate::domain::entity::audit_log::AuditLogSearchParams,
    ) -> anyhow::Result<(Vec<crate::domain::entity::audit_log::AuditLog>, i64)> {
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
