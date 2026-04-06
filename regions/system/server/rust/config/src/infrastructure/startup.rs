use anyhow::Context;
// base64 エンコードされた暗号化鍵のデコードに使用
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
// proto stubs・未接続の gRPC インフラは将来の proto codegen 後に使用される
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use super::config::Config;
use crate::adapter::grpc::ConfigGrpcService;
use crate::adapter::handler;
use crate::adapter::repository::config_postgres::ConfigPostgresRepository;
use crate::adapter::repository::config_schema_postgres::ConfigSchemaPostgresRepository;
// gRPC 認証レイヤー
use k1s0_server_common::middleware::grpc_auth::GrpcAuthLayer;
use k1s0_server_common::middleware::rbac::Tier;

pub async fn run() -> anyhow::Result<()> {
    // Telemetry
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-config-server".to_string(),
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

    // Config

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        host = %cfg.server.host,
        metrics_enabled = cfg.observability.metrics.enabled,
        metrics_path = %cfg.observability.metrics.path,
        read_timeout = %cfg.server.read_timeout,
        write_timeout = %cfg.server.write_timeout,
        shutdown_timeout = %cfg.server.shutdown_timeout,
        cache_refresh_on_miss = cfg.config_server.cache.refresh_on_miss,
        namespace_default_prefix = %cfg.config_server.namespace.default_prefix,
        namespace_allowed_tiers = ?cfg.config_server.namespace.allowed_tiers,
        namespace_max_depth = cfg.config_server.namespace.max_depth,
        audit_enabled = cfg.config_server.audit.enabled,
        audit_retention_days = cfg.config_server.audit.retention_days,
        audit_kafka_enabled = cfg.config_server.audit.kafka_enabled,
        audit_kafka_topic = %cfg.config_server.audit.kafka_topic,
        "starting config server"
    );

    // Metrics (shared across layers and repositories)
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-config-server"));

    // STATIC-HIGH-002: AES-256-GCM 暗号化鍵の初期化
    // CONFIG_ENCRYPTION_KEY 環境変数（優先）または config_server.encryption.key_base64 から取得する
    let encryption_key: Option<[u8; 32]> = if cfg.config_server.encryption.enabled {
        let key_b64 = std::env::var("CONFIG_ENCRYPTION_KEY")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                if cfg.config_server.encryption.key_base64.is_empty() {
                    None
                } else {
                    Some(cfg.config_server.encryption.key_base64.clone())
                }
            })
            .context("config_server.encryption.enabled = true の場合、CONFIG_ENCRYPTION_KEY 環境変数または config_server.encryption.key_base64 が必要です")?;

        let key_bytes = BASE64_STANDARD
            .decode(&key_b64)
            .context("CONFIG_ENCRYPTION_KEY の base64 デコードに失敗しました")?;

        anyhow::ensure!(
            key_bytes.len() == 32,
            "CONFIG_ENCRYPTION_KEY は 32 バイト（base64 エンコード後 44 文字）である必要があります。実際: {} バイト",
            key_bytes.len()
        );

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        info!("設定値暗号化を有効化しました（対象 namespace: {:?}）", cfg.config_server.encryption.sensitive_namespaces);
        Some(key)
    } else {
        // 本番環境（dev/development/local/test 以外）では暗号化を必須とする（MED-001 監査対応）
        // 設定値にはデータベースパスワード・APIキー等の機密情報が含まれる可能性があるため、
        // 本番での平文保存はセキュリティリスクとなる
        let env = cfg.app.environment.as_str();
        if !matches!(env, "dev" | "development" | "local" | "test") {
            anyhow::bail!(
                "本番環境（environment={}）では config_server.encryption.enabled = true が必須です。\
                config/config.prod.yaml の encryption.enabled を true に設定し、\
                CONFIG_ENCRYPTION_KEY 環境変数に 32 バイト（base64 エンコード）の鍵を設定してください。",
                env
            );
        }
        info!("設定値暗号化は無効です（environment={}: 開発環境のみ許可）", env);
        None
    };

    // Config repository: PostgreSQL if DATABASE_URL or database config is set, otherwise in-memory
    let (config_repo, schema_repo): (
        Arc<dyn crate::domain::repository::ConfigRepository>,
        Arc<dyn crate::domain::repository::ConfigSchemaRepository>,
    ) = if let Ok(database_url) = std::env::var("DATABASE_URL") {
        info!("connecting to PostgreSQL...");
        if let Some(db_cfg) = cfg.database.as_ref() {
            info!(
                max_open_conns = db_cfg.max_open_conns,
                max_idle_conns = db_cfg.max_idle_conns,
                conn_max_lifetime = %db_cfg.conn_max_lifetime,
                "database pool options loaded from config"
            );
        }
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(cfg.database.as_ref().map_or(25, |db| db.max_open_conns))
            .max_lifetime(
                cfg.database
                    .as_ref()
                    .and_then(|db| parse_duration(&db.conn_max_lifetime)),
            )
            .connect(&database_url)
            .await?;
        info!("connected to PostgreSQL");
        // STATIC-HIGH-002: 暗号化が有効な場合は暗号化鍵を設定する
        let base_pg_repo = ConfigPostgresRepository::with_metrics(pool.clone(), metrics.clone());
        let pg_repo = Arc::new(if let Some(key) = encryption_key {
            base_pg_repo.set_encryption(key, cfg.config_server.encryption.sensitive_namespaces.clone())
        } else {
            base_pg_repo
        });
        let schema_pg_repo = Arc::new(ConfigSchemaPostgresRepository::with_metrics(
            pool,
            metrics.clone(),
        ));
        let cache_ttl_seconds = cfg.config_server.cache.ttl_seconds().unwrap_or_else(|e| {
            tracing::warn!(error = %e, ttl = %cfg.config_server.cache.ttl, "invalid cache ttl, fallback to 60s");
            60
        });

        let cache = Arc::new(super::cache::ConfigCache::new(
            cfg.config_server.cache.max_entries as u64,
            cache_ttl_seconds,
        ));
        info!(
            max_capacity = cfg.config_server.cache.max_entries,
            ttl_seconds = cache_ttl_seconds,
            "config cache initialized"
        );
        (
            Arc::new(
                crate::adapter::repository::cached_config_repository::CachedConfigRepository::with_metrics(
                    pg_repo,
                    cache,
                    metrics.clone(),
                ),
            ),
            schema_pg_repo,
        )
    } else if let Some(ref db_cfg) = cfg.database {
        info!("connecting to PostgreSQL via config...");
        info!(
            max_open_conns = db_cfg.max_open_conns,
            max_idle_conns = db_cfg.max_idle_conns,
            conn_max_lifetime = %db_cfg.conn_max_lifetime,
            "database pool options loaded from config"
        );
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(db_cfg.max_open_conns)
            .max_lifetime(parse_duration(&db_cfg.conn_max_lifetime))
            .connect(&db_cfg.connection_url())
            .await?;
        info!("connected to PostgreSQL");
        // STATIC-HIGH-002: 暗号化が有効な場合は暗号化鍵を設定する
        let base_pg_repo = ConfigPostgresRepository::with_metrics(pool.clone(), metrics.clone());
        let pg_repo = Arc::new(if let Some(key) = encryption_key {
            base_pg_repo.set_encryption(key, cfg.config_server.encryption.sensitive_namespaces.clone())
        } else {
            base_pg_repo
        });
        let schema_pg_repo = Arc::new(ConfigSchemaPostgresRepository::with_metrics(
            pool,
            metrics.clone(),
        ));
        let cache_ttl_seconds = cfg.config_server.cache.ttl_seconds().unwrap_or_else(|e| {
            tracing::warn!(error = %e, ttl = %cfg.config_server.cache.ttl, "invalid cache ttl, fallback to 60s");
            60
        });

        let cache = Arc::new(super::cache::ConfigCache::new(
            cfg.config_server.cache.max_entries as u64,
            cache_ttl_seconds,
        ));
        info!(
            max_capacity = cfg.config_server.cache.max_entries,
            ttl_seconds = cache_ttl_seconds,
            "config cache initialized"
        );
        (
            Arc::new(
                crate::adapter::repository::cached_config_repository::CachedConfigRepository::with_metrics(
                    pg_repo,
                    cache,
                    metrics.clone(),
                ),
            ),
            schema_pg_repo,
        )
    } else {
        // infra_guard: stable サービスでは DB 設定を必須化（dev/test 以外はエラー）
        k1s0_server_common::require_infra(
            "config",
            k1s0_server_common::InfraKind::Database,
            &cfg.app.environment,
            None::<String>,
        )?;
        info!("no database configured, using in-memory repository (dev/test bypass)");
        (
            Arc::new(InMemoryConfigRepository::new()),
            Arc::new(InMemoryConfigSchemaRepository::new()),
        )
    };

    // Kafka producer (optional)
    let kafka_producer = cfg.kafka.as_ref().and_then(|kafka_cfg| {
        info!(
            brokers = ?kafka_cfg.brokers,
            consumer_group = %kafka_cfg.consumer_group,
            subscribe_topics = ?kafka_cfg.topics.subscribe,
            "configuring kafka producer"
        );
        match super::kafka_producer::KafkaProducer::new(kafka_cfg) {
            Ok(p) => {
                info!("kafka producer initialized for config change notifications");
                Some(std::sync::Arc::new(p.with_metrics(metrics.clone())))
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "failed to create kafka producer, config change events will not be published"
                );
                None
            }
        }
    });

    // --- WatchConfig (broadcast channel) ---
    let watch_uc = Arc::new(crate::usecase::watch_config::WatchConfigUseCase::new());
    let watch_tx = watch_uc.sender();
    info!("watch config broadcast channel initialized");

    // --- gRPC Service ---
    let get_config_uc = Arc::new(crate::usecase::GetConfigUseCase::new(config_repo.clone()));
    let list_configs_uc = Arc::new(crate::usecase::ListConfigsUseCase::new(config_repo.clone()));
    let get_service_config_uc = Arc::new(crate::usecase::GetServiceConfigUseCase::new(
        config_repo.clone(),
    ));
    let update_config_uc_grpc = if let Some(ref producer) = kafka_producer {
        Arc::new(
            crate::usecase::UpdateConfigUseCase::new_with_kafka_and_watch(
                config_repo.clone(),
                producer.clone(),
                watch_tx.clone(),
            )
            .with_schema_repo(schema_repo.clone()),
        )
    } else {
        Arc::new(
            crate::usecase::UpdateConfigUseCase::new_with_watch(
                config_repo.clone(),
                watch_tx.clone(),
            )
            .with_schema_repo(schema_repo.clone()),
        )
    };
    let delete_config_uc = Arc::new(crate::usecase::DeleteConfigUseCase::new(
        config_repo.clone(),
    ));
    let get_config_schema_uc = Arc::new(crate::usecase::GetConfigSchemaUseCase::new(
        schema_repo.clone(),
    ));
    let upsert_config_schema_uc = Arc::new(crate::usecase::UpsertConfigSchemaUseCase::new(
        schema_repo.clone(),
    ));
    let list_config_schemas_uc = Arc::new(crate::usecase::ListConfigSchemasUseCase::new(
        schema_repo.clone(),
    ));

    let config_grpc_svc = Arc::new(
        ConfigGrpcService::new_with_watch(
            get_config_uc,
            list_configs_uc,
            get_service_config_uc,
            update_config_uc_grpc,
            delete_config_uc,
            watch_uc.clone(),
        )
        .with_schema_usecases(
            get_config_schema_uc,
            upsert_config_schema_uc,
            list_config_schemas_uc,
        ),
    );

    // tonic ラッパー
    let config_tonic = crate::adapter::grpc::ConfigServiceTonic::new(config_grpc_svc);

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = k1s0_server_common::require_auth_state(
        "config-server",
        &cfg.app.environment,
        cfg.auth
            .as_ref()
            .map(|auth_cfg| -> anyhow::Result<_> {
                // nested 形式の AuthConfig から JWKS 検証器を初期化する
                let jwks = auth_cfg
                    .jwks
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("auth.jwks configuration is required"))?;
                info!(jwks_url = %jwks.url, "initializing JWKS verifier for config-server");
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

    // AppState (REST handler 用) - Kafka通知付きで構築
    let mut state = handler::AppState::new(config_repo.clone(), schema_repo.clone());
    state.update_config_uc = if let Some(ref producer) = kafka_producer {
        std::sync::Arc::new(
            crate::usecase::UpdateConfigUseCase::new_with_kafka_and_watch(
                config_repo.clone(),
                producer.clone(),
                watch_tx.clone(),
            )
            .with_schema_repo(schema_repo.clone()),
        )
    } else {
        std::sync::Arc::new(
            crate::usecase::UpdateConfigUseCase::new_with_watch(
                config_repo.clone(),
                watch_tx.clone(),
            )
            .with_schema_repo(schema_repo.clone()),
        )
    };
    state.metrics = metrics.clone();
    if kafka_producer.is_some() {
        state = state.with_kafka();
    }
    // gRPC 認証レイヤー用に auth_state を REST への移動前にクローンしておく。
    let grpc_auth_layer = GrpcAuthLayer::new(auth_state.clone(), Tier::System, config_grpc_action);
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    // Router
    let app = handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

    // gRPC server
    let grpc_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.grpc_port).parse()?;
    info!("gRPC server starting on {}", grpc_addr);

    use crate::proto::k1s0::system::config::v1::config_service_server::ConfigServiceServer;

    // gRPC Health Check Protocol サービスを登録する。
    // readyz エンドポイントや Kubernetes の livenessProbe/readinessProbe が
    // Bearer token なしでヘルスチェックできるようにするため。
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<ConfigServiceServer<crate::adapter::grpc::ConfigServiceTonic>>()
        .await;

    let grpc_metrics = metrics;
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(health_service)
            .add_service(ConfigServiceServer::new(config_tonic))
            .serve_with_shutdown(grpc_addr, async move {
                let _ = grpc_shutdown.await;
            })
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    // REST server
    let rest_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port).parse()?;
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

    info!("config server stopped");

    // テレメトリのエクスポーターをフラッシュしてシャットダウンする
    k1s0_telemetry::shutdown();

    Ok(())
}

/// gRPC メソッド名から必要な RBAC アクション文字列を返す。
/// UpdateConfig / DeleteConfig / UpsertConfigSchema は write、それ以外は read。
fn config_grpc_action(method: &str) -> &'static str {
    match method {
        "UpdateConfig" | "DeleteConfig" | "UpsertConfigSchema" => "write",
        _ => "read",
    }
}

// --- In-memory implementation for dev mode ---

use crate::domain::entity::config_change_log::ConfigChangeLog;
use crate::domain::entity::config_entry::{
    ConfigEntry, ConfigListResult, Pagination, ServiceConfigEntry, ServiceConfigResult,
};
use crate::domain::entity::config_schema::ConfigSchema;
use crate::domain::error::ConfigRepositoryError;
use tokio::sync::RwLock;
use uuid::Uuid;

/// InMemoryConfigRepository は開発用のインメモリ設定リポジトリ。
struct InMemoryConfigRepository {
    entries: RwLock<Vec<ConfigEntry>>,
}

impl InMemoryConfigRepository {
    fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl crate::domain::repository::ConfigRepository for InMemoryConfigRepository {
    /// namespace と key で設定値を取得する（インメモリ実装）。
    /// STATIC-CRITICAL-001: tenant_id は受け取るが、インメモリ実装はテナント分離を行わない（開発用）。
    async fn find_by_namespace_and_key(
        &self,
        _tenant_id: Uuid,
        namespace: &str,
        key: &str,
    ) -> Result<Option<ConfigEntry>, ConfigRepositoryError> {
        let entries = self.entries.read().await;
        Ok(entries
            .iter()
            .find(|e| e.namespace == namespace && e.key == key)
            .cloned())
    }

    /// namespace 内の設定値一覧を取得する（インメモリ実装）。
    /// STATIC-CRITICAL-001: tenant_id は受け取るが、インメモリ実装はテナント分離を行わない（開発用）。
    async fn list_by_namespace(
        &self,
        _tenant_id: Uuid,
        namespace: &str,
        page: i32,
        page_size: i32,
        search: Option<String>,
    ) -> Result<ConfigListResult, ConfigRepositoryError> {
        let entries = self.entries.read().await;
        let mut filtered: Vec<_> = entries
            .iter()
            .filter(|e| {
                if e.namespace != namespace {
                    return false;
                }
                if let Some(ref s) = search {
                    if !e.key.contains(s) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        let total_count = filtered.len() as i64;
        let offset = ((page - 1) * page_size) as usize;
        let limit = page_size as usize;

        filtered = filtered.into_iter().skip(offset).take(limit).collect();
        let has_next = (offset + limit) < total_count as usize;

        Ok(ConfigListResult {
            entries: filtered,
            pagination: Pagination {
                total_count,
                page,
                page_size,
                has_next,
            },
        })
    }

    /// 設定値を更新する（インメモリ実装、楽観的排他制御付き）。
    /// STATIC-CRITICAL-001: tenant_id は受け取るが、インメモリ実装はテナント分離を行わない（開発用）。
    async fn update(
        &self,
        _tenant_id: Uuid,
        namespace: &str,
        key: &str,
        value_json: &serde_json::Value,
        expected_version: i32,
        description: Option<String>,
        updated_by: &str,
    ) -> Result<ConfigEntry, ConfigRepositoryError> {
        let mut entries = self.entries.write().await;
        let entry = entries
            .iter_mut()
            .find(|e| e.namespace == namespace && e.key == key);

        match entry {
            Some(e) => {
                // バージョン不一致: 楽観的排他制御エラー
                if e.version != expected_version {
                    return Err(ConfigRepositoryError::VersionConflict {
                        expected: expected_version,
                        current: e.version,
                    });
                }
                e.value_json = value_json.clone();
                e.version += 1;
                if let Some(desc) = description {
                    e.description = desc;
                }
                e.updated_by = updated_by.to_string();
                e.updated_at = chrono::Utc::now();
                Ok(e.clone())
            }
            // キーが存在しない: NotFound エラー
            None => Err(ConfigRepositoryError::NotFound {
                namespace: namespace.to_string(),
                key: key.to_string(),
            }),
        }
    }

    /// 設定値を削除する（インメモリ実装）。
    /// STATIC-CRITICAL-001: tenant_id は受け取るが、インメモリ実装はテナント分離を行わない（開発用）。
    async fn delete(
        &self,
        _tenant_id: Uuid,
        namespace: &str,
        key: &str,
    ) -> Result<bool, ConfigRepositoryError> {
        let mut entries = self.entries.write().await;
        let len_before = entries.len();
        entries.retain(|e| !(e.namespace == namespace && e.key == key));
        Ok(entries.len() < len_before)
    }

    /// サービス名に紐づく設定値を一括取得する（インメモリ実装）。
    /// STATIC-CRITICAL-001: tenant_id は受け取るが、インメモリ実装はテナント分離を行わない（開発用）。
    async fn find_by_service_name(
        &self,
        _tenant_id: Uuid,
        service_name: &str,
    ) -> Result<ServiceConfigResult, ConfigRepositoryError> {
        let entries = self.entries.read().await;
        let primary_keyword = service_name.split('-').next().unwrap_or(service_name);
        let matched: Vec<ServiceConfigEntry> = entries
            .iter()
            .filter(|e| {
                e.namespace
                    .split('.')
                    .any(|ns_part| ns_part == primary_keyword)
            })
            .map(|e| ServiceConfigEntry {
                namespace: e.namespace.clone(),
                key: e.key.clone(),
                value: e.value_json.clone(),
                version: e.version,
            })
            .collect();

        if matched.is_empty() {
            return Err(ConfigRepositoryError::ServiceNotFound(
                service_name.to_string(),
            ));
        }

        Ok(ServiceConfigResult {
            service_name: service_name.to_string(),
            entries: matched,
        })
    }

    /// 設定変更ログを記録する（インメモリ実装、開発用のため捨てる）。
    async fn record_change_log(&self, _log: &ConfigChangeLog) -> Result<(), ConfigRepositoryError> {
        // In-memory: ログは捨てる（開発用）
        Ok(())
    }
}

/// InMemoryConfigSchemaRepository は開発用のインメモリ設定スキーマリポジトリ。
struct InMemoryConfigSchemaRepository {
    schemas: RwLock<Vec<ConfigSchema>>,
}

impl InMemoryConfigSchemaRepository {
    fn new() -> Self {
        Self {
            schemas: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl crate::domain::repository::ConfigSchemaRepository for InMemoryConfigSchemaRepository {
    // CRITICAL-RUST-001 監査対応: InMemory 実装はシグネチャを合わせるため _tenant_id を受け取る（RLS 不要）。
    async fn find_by_service_name(
        &self,
        service_name: &str,
        _tenant_id: &str,
    ) -> anyhow::Result<Option<ConfigSchema>> {
        let schemas = self.schemas.read().await;
        Ok(schemas
            .iter()
            .find(|s| s.service_name == service_name)
            .cloned())
    }

    // CRITICAL-RUST-001 監査対応: InMemory 実装はシグネチャを合わせるため _tenant_id を受け取る（RLS 不要）。
    async fn find_by_namespace(
        &self,
        namespace: &str,
        _tenant_id: &str,
    ) -> anyhow::Result<Option<ConfigSchema>> {
        let schemas = self.schemas.read().await;
        Ok(schemas
            .iter()
            .find(|s| namespace.starts_with(&s.namespace_prefix))
            .cloned())
    }

    // CRITICAL-RUST-001 監査対応: InMemory 実装はシグネチャを合わせるため _tenant_id を受け取る（RLS 不要）。
    async fn list_all(&self, _tenant_id: &str) -> anyhow::Result<Vec<ConfigSchema>> {
        let schemas = self.schemas.read().await;
        Ok(schemas.clone())
    }

    async fn upsert(&self, schema: &ConfigSchema) -> anyhow::Result<ConfigSchema> {
        let mut schemas = self.schemas.write().await;
        if let Some(existing) = schemas
            .iter_mut()
            .find(|s| s.service_name == schema.service_name)
        {
            existing.namespace_prefix = schema.namespace_prefix.clone();
            existing.schema_json = schema.schema_json.clone();
            existing.updated_by = schema.updated_by.clone();
            existing.updated_at = chrono::Utc::now();
            Ok(existing.clone())
        } else {
            schemas.push(schema.clone());
            Ok(schema.clone())
        }
    }
}

fn parse_duration(raw: &str) -> Option<std::time::Duration> {
    let trimmed = raw.trim();

    if let Some(value) = trimmed.strip_suffix("ms") {
        return value
            .trim()
            .parse::<u64>()
            .ok()
            .map(std::time::Duration::from_millis);
    }
    if let Some(value) = trimmed.strip_suffix('s') {
        return value
            .trim()
            .parse::<u64>()
            .ok()
            .map(std::time::Duration::from_secs);
    }
    if let Some(value) = trimmed.strip_suffix('m') {
        return value
            .trim()
            .parse::<u64>()
            .ok()
            .map(|mins| std::time::Duration::from_secs(mins * 60));
    }
    if let Some(value) = trimmed.strip_suffix('h') {
        return value
            .trim()
            .parse::<u64>()
            .ok()
            .map(|hours| std::time::Duration::from_secs(hours * 60 * 60));
    }

    trimmed
        .parse::<u64>()
        .ok()
        .map(std::time::Duration::from_secs)
}
