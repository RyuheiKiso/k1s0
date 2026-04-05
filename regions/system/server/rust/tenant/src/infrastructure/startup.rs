use anyhow::Context;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;
use uuid::Uuid;

// gRPC 認証レイヤー
use k1s0_server_common::middleware::grpc_auth::GrpcAuthLayer;
use k1s0_server_common::middleware::rbac::Tier;

use super::config::{
    default_conn_max_lifetime, default_max_connections, default_max_idle_conns,
    parse_pool_duration, Config,
};
use crate::adapter::handler::{self, AppState};
use crate::domain::entity::{ProvisioningJob, Tenant, TenantMember};
use crate::domain::repository::{MemberRepository, TenantRepository};

pub async fn run() -> anyhow::Result<()> {
    // Telemetry
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-tenant-server".to_string(),
        // Cargo.toml の package.version を使用する（M-16 監査対応: ハードコード解消）
        version: env!("CARGO_PKG_VERSION").to_string(),
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
    k1s0_telemetry::init_telemetry(&telemetry_cfg)
        .map_err(|e| anyhow::anyhow!("テレメトリの初期化に失敗: {}", e))?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        http_port = cfg.server.http_port,
        grpc_port = cfg.server.grpc_port,
        "starting tenant server"
    );

    let mut db_pool_for_health: Option<Arc<sqlx::PgPool>> = None;

    // テナントキャッシュ設定を Config から取得する
    let cache_config = crate::infrastructure::cache::TenantCacheConfig {
        ttl_secs: cfg.cache.ttl_secs,
        max_entries: cfg.cache.max_entries,
    };
    info!(
        ttl_secs = cache_config.ttl_secs,
        max_entries = cache_config.max_entries,
        "テナントキャッシュ設定を読み込み"
    );

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
            // PostgreSQLリポジトリをキャッシュで包んでDB負荷を軽減する
            let pg_repo: Arc<dyn TenantRepository> = Arc::new(
                crate::adapter::repository::tenant_postgres::TenantPostgresRepository::new(
                    pool.clone(),
                ),
            );
            let cached_repo = Arc::new(crate::infrastructure::cache::CachedTenantRepository::new(
                pg_repo,
                &cache_config,
            ));
            info!("テナントキャッシュを初期化");
            (
                cached_repo as Arc<dyn TenantRepository>,
                Arc::new(
                    crate::adapter::repository::member_postgres::MemberPostgresRepository::new(
                        pool,
                    ),
                ),
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
            // PostgreSQLリポジトリをキャッシュで包んでDB負荷を軽減する
            let pg_repo: Arc<dyn TenantRepository> = Arc::new(
                crate::adapter::repository::tenant_postgres::TenantPostgresRepository::new(
                    pool.clone(),
                ),
            );
            let cached_repo = Arc::new(crate::infrastructure::cache::CachedTenantRepository::new(
                pg_repo,
                &cache_config,
            ));
            info!("テナントキャッシュを初期化");
            (
                cached_repo as Arc<dyn TenantRepository>,
                Arc::new(
                    crate::adapter::repository::member_postgres::MemberPostgresRepository::new(
                        pool,
                    ),
                ),
            )
        } else {
            // infra_guard: stable サービスでは DB 設定を必須化（dev/test 以外はエラー）
            k1s0_server_common::require_infra(
                "tenant",
                k1s0_server_common::InfraKind::Database,
                &cfg.app.environment,
                None::<String>,
            )?;
            info!("database not configured, using in-memory repositories (dev/test bypass)");
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
    let event_publisher: Arc<dyn super::kafka_producer::TenantEventPublisher> =
        if let Some(ref kafka_cfg) = cfg.kafka {
            info!("initializing Kafka event publisher...");
            let publisher = super::kafka_producer::KafkaTenantEventPublisher::new(kafka_cfg)?;
            info!(topic = %publisher.topic(), "Kafka event publisher initialized");
            Arc::new(publisher)
        } else if let Some(brokers) = kafka_brokers_for_health.clone() {
            info!("initializing Kafka event publisher from KAFKA_BROKERS env...");
            let kafka_cfg = super::kafka_producer::KafkaConfig {
                brokers,
                consumer_group: String::new(),
                security_protocol: "PLAINTEXT".to_string(),
                sasl: Default::default(),
                topics: Default::default(),
            };
            let publisher = super::kafka_producer::KafkaTenantEventPublisher::new(&kafka_cfg)?;
            info!(topic = %publisher.topic(), "Kafka event publisher initialized");
            Arc::new(publisher)
        } else {
            // infra_guard: stable サービスでは Kafka 設定を必須化（dev/test 以外はエラー）
            k1s0_server_common::require_infra(
                "tenant",
                k1s0_server_common::InfraKind::Kafka,
                &cfg.app.environment,
                None::<String>,
            )?;
            info!("Kafka not configured, using noop event publisher (dev/test bypass)");
            Arc::new(super::kafka_producer::NoopTenantEventPublisher)
        };

    // Saga client: HttpSagaClient if SAGA_SERVER_URL is set, otherwise NoopSagaClient
    let saga_client: Arc<dyn super::saga_client::SagaClient> =
        if let Ok(saga_url) = std::env::var("SAGA_SERVER_URL") {
            info!(saga_url = %saga_url, "initializing HTTP saga client");
            // new() が Result を返すようになったため ? で伝播する
            Arc::new(super::saga_client::HttpSagaClient::new(&saga_url)?)
        } else {
            info!("SAGA_SERVER_URL not set, using noop saga client");
            Arc::new(super::saga_client::NoopSagaClient)
        };

    let keycloak_admin: Arc<dyn super::keycloak_admin::KeycloakAdmin> =
        if let Some(ref keycloak_cfg) = cfg.keycloak {
            info!(
                base_url = %keycloak_cfg.base_url,
                realm = %keycloak_cfg.realm,
                "initializing keycloak admin client"
            );
            // new() が Result を返すようになったため ? で伝播する
            Arc::new(super::keycloak_admin::KeycloakAdminClient::new(
                keycloak_cfg.clone(),
            )?)
        } else {
            info!("keycloak config not set, using noop keycloak admin client");
            Arc::new(super::keycloak_admin::NoopKeycloakAdmin)
        };

    let keycloak_health_url = cfg
        .keycloak
        .as_ref()
        .map(|k| format!("{}/realms/{}", k.base_url.trim_end_matches('/'), k.realm));

    // Watch broadcast channel for tenant change streaming
    let (_watch_uc, watch_tx) = crate::usecase::WatchTenantUseCase::new();
    info!("watch tenant broadcast channel initialized");

    let create_tenant_uc = Arc::new(
        crate::usecase::CreateTenantUseCase::new(tenant_repo.clone())
            .with_saga_client(saga_client.clone())
            .with_event_publisher(event_publisher.clone())
            .with_keycloak_admin(keycloak_admin.clone())
            .with_watch_sender(watch_tx.clone()),
    );
    let get_tenant_uc = Arc::new(crate::usecase::GetTenantUseCase::new(tenant_repo.clone()));
    let list_tenants_uc = Arc::new(crate::usecase::ListTenantsUseCase::new(tenant_repo.clone()));
    let update_tenant_uc = Arc::new(
        crate::usecase::UpdateTenantUseCase::new(tenant_repo.clone())
            .with_event_publisher(event_publisher.clone())
            .with_watch_sender(watch_tx.clone()),
    );
    let delete_tenant_uc = Arc::new(
        crate::usecase::DeleteTenantUseCase::new(tenant_repo.clone())
            .with_saga_client(saga_client)
            .with_keycloak_admin(keycloak_admin)
            .with_event_publisher(event_publisher.clone())
            .with_watch_sender(watch_tx.clone()),
    );
    let suspend_tenant_uc = Arc::new(
        crate::usecase::SuspendTenantUseCase::new(tenant_repo.clone())
            .with_event_publisher(event_publisher)
            .with_watch_sender(watch_tx.clone()),
    );
    let activate_tenant_uc = Arc::new(
        crate::usecase::ActivateTenantUseCase::new(tenant_repo.clone())
            .with_watch_sender(watch_tx.clone()),
    );
    let add_member_uc = Arc::new(crate::usecase::AddMemberUseCase::new(member_repo.clone()));
    let remove_member_uc = Arc::new(crate::usecase::RemoveMemberUseCase::new(
        member_repo.clone(),
    ));
    let list_members_uc = Arc::new(crate::usecase::ListMembersUseCase::new(member_repo.clone()));
    let update_member_role_uc = Arc::new(crate::usecase::UpdateMemberRoleUseCase::new(
        member_repo.clone(),
        tenant_repo.clone(),
    ));
    let get_provisioning_status_uc = Arc::new(crate::usecase::GetProvisioningStatusUseCase::new(
        member_repo,
    ));

    // gRPC service
    use crate::adapter::grpc::TenantGrpcService;
    use crate::proto::k1s0::system::tenant::v1::tenant_service_server::TenantServiceServer;

    let tenant_grpc_svc = Arc::new(TenantGrpcService::new_with_watch(
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
        // メンバーロール更新ユースケースをgRPCサービスに注入する
        update_member_role_uc.clone(),
        get_provisioning_status_uc,
        watch_tx,
    ));
    let tenant_tonic = crate::adapter::grpc::TenantServiceTonic::new(tenant_grpc_svc);

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-tenant-server"));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = k1s0_server_common::require_auth_state(
        "tenant-server",
        &cfg.app.environment,
        cfg.auth
            .as_ref()
            .map(|auth_cfg| -> anyhow::Result<_> {
                // nested 形式の AuthConfig から JWKS 検証器を初期化する
                let jwks = auth_cfg
                    .jwks
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("auth.jwks configuration is required"))?;
                info!(jwks_url = %jwks.url, "initializing JWKS verifier for tenant-server");
                let jwks_verifier = Arc::new(
                    k1s0_auth::JwksVerifier::new(
                        &jwks.url,
                        &auth_cfg.jwt.issuer,
                        &auth_cfg.jwt.audience,
                        std::time::Duration::from_secs(jwks.cache_ttl_secs),
                    )
                    .context("JWKS 検証器の作成に失敗")?,
                );
                Ok(crate::adapter::middleware::auth::AuthState {
                    verifier: jwks_verifier,
                })
            })
            .transpose()?,
    )?;

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
        update_member_role_uc,
        metrics: metrics.clone(),
        auth_state: None,
        db_pool: db_pool_for_health,
        kafka_brokers: kafka_brokers_for_health,
        keycloak_health_url,
        // ヘルスチェック用HTTPクライアント（タイムアウト10秒）
        http_client: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .context("HTTP client の作成に失敗")?,
    };
    // gRPC 認証レイヤー用に auth_state を REST への移動前にクローンしておく。
    let grpc_auth_layer = GrpcAuthLayer::new(auth_state.clone(), Tier::System, tenant_grpc_action);
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }
    let app = handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    // gRPC Health Check Protocol サービスを登録する。
    // readyz エンドポイントや Kubernetes の livenessProbe/readinessProbe が
    // Bearer token なしでヘルスチェックできるようにするため。
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<TenantServiceServer<crate::adapter::grpc::TenantServiceTonic>>()
        .await;

    let grpc_metrics = metrics;
    // gRPC グレースフルシャットダウン用シグナル
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(health_service)
            .add_service(TenantServiceServer::new(tenant_tonic))
            .serve_with_shutdown(grpc_addr, async move {
                let _ = grpc_shutdown.await;
            })
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.http_port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    // REST グレースフルシャットダウン設定
    let rest_future = axum::serve(listener, app).with_graceful_shutdown(async {
        let _ = k1s0_server_common::shutdown::shutdown_signal().await;
    });

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

    // テレメトリのシャットダウン処理
    k1s0_telemetry::shutdown();

    Ok(())
}

/// gRPC メソッド名から必要な RBAC アクション文字列を返す。
/// CreateTenant / UpdateTenant / DeleteTenant / SuspendTenant / ActivateTenant /
/// AddMember / RemoveMember は write、それ以外は read。
/// gRPC メソッド名から RBAC アクション文字列を返す。
/// 書き込み系メソッドは "write"、それ以外は "read" を返す。
fn tenant_grpc_action(method: &str) -> &'static str {
    match method {
        "CreateTenant" | "UpdateTenant" | "DeleteTenant" | "SuspendTenant" | "ActivateTenant"
        | "AddMember" | "RemoveMember" | "UpdateMemberRole" => "write",
        _ => "read",
    }
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

    async fn update_role(
        &self,
        tenant_id: &Uuid,
        user_id: &Uuid,
        role: &str,
    ) -> anyhow::Result<Option<TenantMember>> {
        let mut members = self.members.write().await;
        if let Some(member) = members
            .iter_mut()
            .find(|m| m.tenant_id == *tenant_id && m.user_id == *user_id)
        {
            member.role = role.to_string();
            Ok(Some(member.clone()))
        } else {
            Ok(None)
        }
    }

    async fn find_job(&self, job_id: &Uuid) -> anyhow::Result<Option<ProvisioningJob>> {
        let jobs = self.jobs.read().await;
        Ok(jobs.iter().find(|j| j.id == *job_id).cloned())
    }
}
