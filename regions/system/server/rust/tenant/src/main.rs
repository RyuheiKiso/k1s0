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

use adapter::handler::{self, AppState};
use domain::entity::{ProvisioningJob, Tenant, TenantMember, TenantStatus};
use domain::repository::{MemberRepository, TenantRepository};

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
    kafka: Option<infrastructure::kafka_producer::KafkaConfig>,
    #[serde(default)]
    keycloak: Option<infrastructure::keycloak_admin::KeycloakAdminConfig>,
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
    #[serde(default = "default_http_port")]
    http_port: u16,
    #[serde(default = "default_grpc_port")]
    grpc_port: u16,
}

fn default_http_port() -> u16 {
    8080
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

#[derive(Debug, Clone, serde::Deserialize)]
struct DatabaseConfig {
    #[serde(default = "default_db_host")]
    host: String,
    #[serde(default = "default_db_port")]
    port: u16,
    #[serde(default = "default_db_name")]
    name: String,
    #[serde(default = "default_db_user")]
    user: String,
    #[serde(default)]
    password: String,
    #[serde(default = "default_db_ssl_mode")]
    ssl_mode: String,
    #[serde(default = "default_max_connections")]
    max_connections: u32,
    #[serde(default = "default_max_idle_conns")]
    max_idle_conns: u32,
    #[serde(default = "default_conn_max_lifetime")]
    conn_max_lifetime: String,
}

impl DatabaseConfig {
    fn connection_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode={}",
            self.user, self.password, self.host, self.port, self.name, self.ssl_mode
        )
    }
}

fn default_db_host() -> String {
    "localhost".to_string()
}

fn default_db_port() -> u16 {
    5432
}

fn default_db_name() -> String {
    "k1s0_system".to_string()
}

fn default_db_user() -> String {
    "app".to_string()
}

fn default_db_ssl_mode() -> String {
    "disable".to_string()
}

fn default_max_connections() -> u32 {
    25
}

fn default_max_idle_conns() -> u32 {
    5
}

fn default_conn_max_lifetime() -> String {
    "5m".to_string()
}

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

// --- InMemory Repository ---

struct InMemoryTenantRepository {
    tenants: tokio::sync::RwLock<Vec<Tenant>>,
}

impl InMemoryTenantRepository {
    fn new() -> Self {
        Self {
            tenants: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl TenantRepository for InMemoryTenantRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Tenant>> {
        let tenants = self.tenants.read().await;
        Ok(tenants.iter().find(|t| t.id == *id).cloned())
    }

    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<Tenant>> {
        let tenants = self.tenants.read().await;
        Ok(tenants.iter().find(|t| t.name == name).cloned())
    }

    async fn list(&self, page: i32, page_size: i32) -> anyhow::Result<(Vec<Tenant>, i64)> {
        let tenants = self.tenants.read().await;
        let total = tenants.len() as i64;
        let offset = ((page - 1) * page_size) as usize;
        let result: Vec<_> = tenants
            .iter()
            .skip(offset)
            .take(page_size as usize)
            .cloned()
            .collect();
        Ok((result, total))
    }

    async fn create(&self, tenant: &Tenant) -> anyhow::Result<()> {
        let mut tenants = self.tenants.write().await;
        tenants.push(tenant.clone());
        Ok(())
    }

    async fn update(&self, tenant: &Tenant) -> anyhow::Result<()> {
        let mut tenants = self.tenants.write().await;
        if let Some(existing) = tenants.iter_mut().find(|t| t.id == tenant.id) {
            *existing = tenant.clone();
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Telemetry
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let config_content = std::fs::read_to_string(&config_path)?;
    let cfg: Config = serde_yaml::from_str(&config_content)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-tenant-server".to_string(),
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
        http_port = cfg.server.http_port,
        grpc_port = cfg.server.grpc_port,
        "starting tenant server"
    );

    let mut db_pool_for_health: Option<Arc<sqlx::PgPool>> = None;

    // Repository: config.database (DATABASE_URL fallback) -> DATABASE_URL only -> InMemory
    let (tenant_repo, member_repo): (Arc<dyn TenantRepository>, Arc<dyn MemberRepository>) =
        if let Some(ref db_cfg) = cfg.database {
            let database_url =
                std::env::var("DATABASE_URL").unwrap_or_else(|_| db_cfg.connection_url());
            info!("connecting to PostgreSQL...");
            let pool = Arc::new(
                sqlx::postgres::PgPoolOptions::new()
                    .max_connections(db_cfg.max_connections)
                    .min_connections(db_cfg.max_idle_conns.min(db_cfg.max_connections))
                    .max_lifetime(parse_pool_duration(&db_cfg.conn_max_lifetime))
                    .connect(&database_url)
                    .await?,
            );
            db_pool_for_health = Some(pool.clone());
            info!("connected to PostgreSQL");
            (
                Arc::new(
                    adapter::repository::tenant_postgres::TenantPostgresRepository::new(
                        pool.clone(),
                    ),
                ),
                Arc::new(adapter::repository::member_postgres::MemberPostgresRepository::new(pool)),
            )
        } else if let Ok(database_url) = std::env::var("DATABASE_URL") {
            info!("connecting to PostgreSQL with DATABASE_URL fallback...");
            let pool = Arc::new(
                sqlx::postgres::PgPoolOptions::new()
                    .max_connections(default_max_connections())
                    .min_connections(default_max_idle_conns())
                    .max_lifetime(parse_pool_duration(&default_conn_max_lifetime()))
                    .connect(&database_url)
                    .await?,
            );
            db_pool_for_health = Some(pool.clone());
            info!("connected to PostgreSQL");
            (
                Arc::new(
                    adapter::repository::tenant_postgres::TenantPostgresRepository::new(
                        pool.clone(),
                    ),
                ),
                Arc::new(adapter::repository::member_postgres::MemberPostgresRepository::new(pool)),
            )
        } else {
            info!("database not configured, using in-memory repositories");
            (
                Arc::new(InMemoryTenantRepository::new()),
                Arc::new(InMemoryMemberRepository::new()),
            )
        };

    let kafka_brokers_for_health = cfg.kafka.as_ref().map(|k| k.brokers.clone()).or_else(|| {
        std::env::var("KAFKA_BROKERS").ok().map(|brokers| {
            brokers
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
    });

    // Kafka event publisher: Kafka if config or KAFKA_BROKERS env var is set, otherwise Noop
    let event_publisher: Arc<dyn infrastructure::kafka_producer::TenantEventPublisher> =
        if let Some(ref kafka_cfg) = cfg.kafka {
            info!("initializing Kafka event publisher...");
            let publisher =
                infrastructure::kafka_producer::KafkaTenantEventPublisher::new(kafka_cfg)?;
            info!(topic = %publisher.topic(), "Kafka event publisher initialized");
            Arc::new(publisher)
        } else if let Some(brokers) = kafka_brokers_for_health.clone() {
            info!("initializing Kafka event publisher from KAFKA_BROKERS env...");
            let kafka_cfg = infrastructure::kafka_producer::KafkaConfig {
                brokers,
                consumer_group: String::new(),
                security_protocol: "PLAINTEXT".to_string(),
                sasl: Default::default(),
                topics: Default::default(),
            };
            let publisher =
                infrastructure::kafka_producer::KafkaTenantEventPublisher::new(&kafka_cfg)?;
            info!(topic = %publisher.topic(), "Kafka event publisher initialized");
            Arc::new(publisher)
        } else {
            info!("Kafka not configured, using noop event publisher");
            Arc::new(infrastructure::kafka_producer::NoopTenantEventPublisher)
        };

    // Saga client: HttpSagaClient if SAGA_SERVER_URL is set, otherwise NoopSagaClient
    let saga_client: Arc<dyn infrastructure::saga_client::SagaClient> =
        if let Ok(saga_url) = std::env::var("SAGA_SERVER_URL") {
            info!(saga_url = %saga_url, "initializing HTTP saga client");
            Arc::new(infrastructure::saga_client::HttpSagaClient::new(&saga_url))
        } else {
            info!("SAGA_SERVER_URL not set, using noop saga client");
            Arc::new(infrastructure::saga_client::NoopSagaClient)
        };

    let keycloak_admin: Arc<dyn infrastructure::keycloak_admin::KeycloakAdmin> =
        if let Some(ref keycloak_cfg) = cfg.keycloak {
            info!(
                base_url = %keycloak_cfg.base_url,
                realm = %keycloak_cfg.realm,
                "initializing keycloak admin client"
            );
            Arc::new(infrastructure::keycloak_admin::KeycloakAdminClient::new(
                keycloak_cfg.clone(),
            ))
        } else {
            info!("keycloak config not set, using noop keycloak admin client");
            Arc::new(infrastructure::keycloak_admin::NoopKeycloakAdmin)
        };

    let keycloak_health_url = cfg
        .keycloak
        .as_ref()
        .map(|k| format!("{}/realms/{}", k.base_url.trim_end_matches('/'), k.realm));

    let create_tenant_uc = Arc::new(
        usecase::CreateTenantUseCase::new(tenant_repo.clone())
            .with_saga_client(saga_client.clone())
            .with_event_publisher(event_publisher.clone())
            .with_keycloak_admin(keycloak_admin.clone()),
    );
    let get_tenant_uc = Arc::new(usecase::GetTenantUseCase::new(tenant_repo.clone()));
    let list_tenants_uc = Arc::new(usecase::ListTenantsUseCase::new(tenant_repo.clone()));
    let update_tenant_uc = Arc::new(
        usecase::UpdateTenantUseCase::new(tenant_repo.clone())
            .with_event_publisher(event_publisher.clone()),
    );
    let delete_tenant_uc = Arc::new(
        usecase::DeleteTenantUseCase::new(tenant_repo.clone())
            .with_saga_client(saga_client)
            .with_keycloak_admin(keycloak_admin)
            .with_event_publisher(event_publisher.clone()),
    );
    let suspend_tenant_uc = Arc::new(
        usecase::SuspendTenantUseCase::new(tenant_repo.clone())
            .with_event_publisher(event_publisher),
    );
    let activate_tenant_uc = Arc::new(usecase::ActivateTenantUseCase::new(tenant_repo));
    let add_member_uc = Arc::new(usecase::AddMemberUseCase::new(member_repo.clone()));
    let remove_member_uc = Arc::new(usecase::RemoveMemberUseCase::new(member_repo.clone()));
    let list_members_uc = Arc::new(usecase::ListMembersUseCase::new(member_repo.clone()));
    let get_provisioning_status_uc =
        Arc::new(usecase::GetProvisioningStatusUseCase::new(member_repo));

    // gRPC service
    use adapter::grpc::TenantGrpcService;
    use proto::k1s0::system::tenant::v1::tenant_service_server::TenantServiceServer;

    let tenant_grpc_svc = Arc::new(TenantGrpcService::new(
        create_tenant_uc.clone(),
        get_tenant_uc.clone(),
        list_tenants_uc.clone(),
        update_tenant_uc.clone(),
        suspend_tenant_uc.clone(),
        activate_tenant_uc.clone(),
        delete_tenant_uc.clone(),
        add_member_uc.clone(),
        list_members_uc.clone(),
        remove_member_uc.clone(),
        get_provisioning_status_uc,
    ));
    let tenant_tonic = adapter::grpc::TenantServiceTonic::new(tenant_grpc_svc);

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-tenant-server"));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = if let Some(ref auth_cfg) = cfg.auth {
        info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for tenant-server");
        let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
            &auth_cfg.jwks_url,
            &auth_cfg.issuer,
            &auth_cfg.audience,
            std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
        ));
        Some(adapter::middleware::auth::TenantAuthState {
            verifier: jwks_verifier,
        })
    } else {
        info!("no auth configured, tenant-server running without authentication");
        None
    };

    let mut state = AppState {
        create_tenant_uc,
        get_tenant_uc,
        list_tenants_uc,
        update_tenant_uc,
        delete_tenant_uc,
        suspend_tenant_uc,
        activate_tenant_uc,
        list_members_uc,
        add_member_uc,
        remove_member_uc,
        metrics: metrics.clone(),
        auth_state: None,
        db_pool: db_pool_for_health,
        kafka_brokers: kafka_brokers_for_health,
        keycloak_health_url,
        http_client: reqwest::Client::new(),
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }
    let app = handler::router(state).layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(TenantServiceServer::new(tenant_tonic))
            .serve(grpc_addr)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.http_port));
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

// --- InMemory MemberRepository ---

struct InMemoryMemberRepository {
    members: tokio::sync::RwLock<Vec<TenantMember>>,
    jobs: tokio::sync::RwLock<Vec<ProvisioningJob>>,
}

impl InMemoryMemberRepository {
    fn new() -> Self {
        Self {
            members: tokio::sync::RwLock::new(Vec::new()),
            jobs: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl MemberRepository for InMemoryMemberRepository {
    async fn find_by_tenant(&self, tenant_id: &Uuid) -> anyhow::Result<Vec<TenantMember>> {
        let members = self.members.read().await;
        Ok(members
            .iter()
            .filter(|m| m.tenant_id == *tenant_id)
            .cloned()
            .collect())
    }

    async fn find_member(
        &self,
        tenant_id: &Uuid,
        user_id: &Uuid,
    ) -> anyhow::Result<Option<TenantMember>> {
        let members = self.members.read().await;
        Ok(members
            .iter()
            .find(|m| m.tenant_id == *tenant_id && m.user_id == *user_id)
            .cloned())
    }

    async fn add(&self, member: &TenantMember) -> anyhow::Result<()> {
        let mut members = self.members.write().await;
        members.push(member.clone());
        Ok(())
    }

    async fn remove(&self, tenant_id: &Uuid, user_id: &Uuid) -> anyhow::Result<bool> {
        let mut members = self.members.write().await;
        let len_before = members.len();
        members.retain(|m| !(m.tenant_id == *tenant_id && m.user_id == *user_id));
        Ok(members.len() < len_before)
    }

    async fn find_job(&self, job_id: &Uuid) -> anyhow::Result<Option<ProvisioningJob>> {
        let jobs = self.jobs.read().await;
        Ok(jobs.iter().find(|j| j.id == *job_id).cloned())
    }
}
