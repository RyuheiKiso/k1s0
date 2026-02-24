// proto stubs・未接続の gRPC インフラは将来の proto codegen 後に使用される
#![allow(dead_code, unused_imports)]

use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod infrastructure;
mod proto;
mod usecase;

use adapter::grpc::ConfigGrpcService;
use adapter::handler;
use adapter::repository::config_postgres::ConfigPostgresRepository;
use adapter::repository::config_schema_postgres::ConfigSchemaPostgresRepository;
use infrastructure::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Telemetry
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-config-server".to_string(),
        version: "0.1.0".to_string(),
        tier: "system".to_string(),
        environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string()),
        trace_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
        sample_rate: 1.0,
        log_level: "info".to_string(),
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
        "starting config server"
    );

    // Metrics (shared across layers and repositories)
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-config-server"));

    // Config repository: PostgreSQL if DATABASE_URL or database config is set, otherwise in-memory
    let (config_repo, schema_repo): (
        Arc<dyn domain::repository::ConfigRepository>,
        Arc<dyn domain::repository::ConfigSchemaRepository>,
    ) = if let Ok(database_url) = std::env::var("DATABASE_URL") {
        info!("connecting to PostgreSQL...");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(cfg.database.as_ref().map_or(25, |db| db.max_open_conns))
            .connect(&database_url)
            .await?;
        info!("connected to PostgreSQL");
        let pg_repo = Arc::new(ConfigPostgresRepository::with_metrics(
            pool.clone(),
            metrics.clone(),
        ));
        let schema_pg_repo = Arc::new(ConfigSchemaPostgresRepository::with_metrics(
            pool,
            metrics.clone(),
        ));
        // キャッシュでラップ（TTL 300秒、最大10000エントリ）
        let cache = Arc::new(infrastructure::cache::ConfigCache::new(10_000, 300));
        info!("config cache initialized (max_capacity=10000, ttl=300s)");
        (
            Arc::new(
                adapter::repository::cached_config_repository::CachedConfigRepository::with_metrics(
                    pg_repo,
                    cache,
                    metrics.clone(),
                ),
            ),
            schema_pg_repo,
        )
    } else if let Some(ref db_cfg) = cfg.database {
        info!("connecting to PostgreSQL via config...");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(db_cfg.max_open_conns)
            .connect(&db_cfg.connection_url())
            .await?;
        info!("connected to PostgreSQL");
        let pg_repo = Arc::new(ConfigPostgresRepository::with_metrics(
            pool.clone(),
            metrics.clone(),
        ));
        let schema_pg_repo = Arc::new(ConfigSchemaPostgresRepository::with_metrics(
            pool,
            metrics.clone(),
        ));
        // キャッシュでラップ（TTL 300秒、最大10000エントリ）
        let cache = Arc::new(infrastructure::cache::ConfigCache::new(10_000, 300));
        info!("config cache initialized (max_capacity=10000, ttl=300s)");
        (
            Arc::new(
                adapter::repository::cached_config_repository::CachedConfigRepository::with_metrics(
                    pg_repo,
                    cache,
                    metrics.clone(),
                ),
            ),
            schema_pg_repo,
        )
    } else {
        info!("no database configured, using in-memory repository");
        (
            Arc::new(InMemoryConfigRepository::new()),
            Arc::new(InMemoryConfigSchemaRepository::new()),
        )
    };

    // Kafka producer (optional)
    let kafka_producer = cfg.kafka.as_ref().and_then(|kafka_cfg| {
        match infrastructure::kafka_producer::KafkaProducer::new(kafka_cfg) {
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

    // --- gRPC Service ---
    let get_config_uc = Arc::new(usecase::GetConfigUseCase::new(config_repo.clone()));
    let list_configs_uc = Arc::new(usecase::ListConfigsUseCase::new(config_repo.clone()));
    let get_service_config_uc =
        Arc::new(usecase::GetServiceConfigUseCase::new(config_repo.clone()));
    let update_config_uc_grpc = if let Some(ref producer) = kafka_producer {
        Arc::new(usecase::UpdateConfigUseCase::new_with_kafka(
            config_repo.clone(),
            producer.clone(),
        ))
    } else {
        Arc::new(usecase::UpdateConfigUseCase::new(config_repo.clone()))
    };
    let delete_config_uc = Arc::new(usecase::DeleteConfigUseCase::new(config_repo.clone()));

    let config_grpc_svc = Arc::new(ConfigGrpcService::new(
        get_config_uc,
        list_configs_uc,
        get_service_config_uc,
        update_config_uc_grpc,
        delete_config_uc,
    ));

    // tonic ラッパー
    let config_tonic = adapter::grpc::ConfigServiceTonic::new(config_grpc_svc);

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = if let Some(ref auth_cfg) = cfg.auth {
        info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier for config-server");
        let jwks_verifier = Arc::new(k1s0_auth::JwksVerifier::new(
            &auth_cfg.jwks_url,
            &auth_cfg.issuer,
            &auth_cfg.audience,
            std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
        ));
        Some(adapter::middleware::auth::ConfigAuthState {
            verifier: jwks_verifier,
        })
    } else {
        info!("no auth configured, config-server running without authentication");
        None
    };

    // AppState (REST handler 用) - Kafka通知付きで構築
    let mut state = adapter::handler::AppState {
        get_config_uc: std::sync::Arc::new(usecase::GetConfigUseCase::new(config_repo.clone())),
        list_configs_uc: std::sync::Arc::new(usecase::ListConfigsUseCase::new(config_repo.clone())),
        update_config_uc: if let Some(ref producer) = kafka_producer {
            std::sync::Arc::new(usecase::UpdateConfigUseCase::new_with_kafka(
                config_repo.clone(),
                producer.clone(),
            ))
        } else {
            std::sync::Arc::new(usecase::UpdateConfigUseCase::new(config_repo.clone()))
        },
        delete_config_uc: std::sync::Arc::new(usecase::DeleteConfigUseCase::new(
            config_repo.clone(),
        )),
        get_service_config_uc: std::sync::Arc::new(usecase::GetServiceConfigUseCase::new(
            config_repo.clone(),
        )),
        get_config_schema_uc: std::sync::Arc::new(usecase::GetConfigSchemaUseCase::new(
            schema_repo.clone(),
        )),
        list_config_schemas_uc: std::sync::Arc::new(usecase::ListConfigSchemasUseCase::new(
            schema_repo.clone(),
        )),
        upsert_config_schema_uc: std::sync::Arc::new(usecase::UpsertConfigSchemaUseCase::new(
            schema_repo,
        )),
        metrics: metrics.clone(),
        config_repo: config_repo.clone(),
        kafka_configured: false,
        auth_state: None,
    };
    if kafka_producer.is_some() {
        state = state.with_kafka();
    }
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    // Router
    let app = handler::router(state).layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    // gRPC server (port 50053)
    let grpc_addr: SocketAddr = ([0, 0, 0, 0], 50053).into();
    info!("gRPC server starting on {}", grpc_addr);

    use proto::k1s0::system::config::v1::config_service_server::ConfigServiceServer;

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(ConfigServiceServer::new(config_tonic))
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

    info!("config server stopped");

    Ok(())
}

// --- In-memory implementation for dev mode ---

use domain::entity::config_change_log::ConfigChangeLog;
use domain::entity::config_entry::{
    ConfigEntry, ConfigListResult, Pagination, ServiceConfigEntry, ServiceConfigResult,
};
use domain::entity::config_schema::ConfigSchema;
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
impl domain::repository::ConfigRepository for InMemoryConfigRepository {
    async fn find_by_namespace_and_key(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Option<ConfigEntry>> {
        let entries = self.entries.read().await;
        Ok(entries
            .iter()
            .find(|e| e.namespace == namespace && e.key == key)
            .cloned())
    }

    async fn list_by_namespace(
        &self,
        namespace: &str,
        page: i32,
        page_size: i32,
        search: Option<String>,
    ) -> anyhow::Result<ConfigListResult> {
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

    async fn create(&self, entry: &ConfigEntry) -> anyhow::Result<ConfigEntry> {
        let mut entries = self.entries.write().await;
        entries.push(entry.clone());
        Ok(entry.clone())
    }

    async fn update(
        &self,
        namespace: &str,
        key: &str,
        value_json: &serde_json::Value,
        expected_version: i32,
        description: Option<String>,
        updated_by: &str,
    ) -> anyhow::Result<ConfigEntry> {
        let mut entries = self.entries.write().await;
        let entry = entries
            .iter_mut()
            .find(|e| e.namespace == namespace && e.key == key);

        match entry {
            Some(e) => {
                if e.version != expected_version {
                    return Err(anyhow::anyhow!("version conflict: current={}", e.version));
                }
                e.value_json = value_json.clone();
                e.version += 1;
                if let Some(desc) = description {
                    e.description = Some(desc);
                }
                e.updated_by = updated_by.to_string();
                e.updated_at = chrono::Utc::now();
                Ok(e.clone())
            }
            None => Err(anyhow::anyhow!("config not found: {}/{}", namespace, key)),
        }
    }

    async fn delete(&self, namespace: &str, key: &str) -> anyhow::Result<bool> {
        let mut entries = self.entries.write().await;
        let len_before = entries.len();
        entries.retain(|e| !(e.namespace == namespace && e.key == key));
        Ok(entries.len() < len_before)
    }

    async fn find_by_service_name(
        &self,
        service_name: &str,
    ) -> anyhow::Result<ServiceConfigResult> {
        let entries = self.entries.read().await;
        // サービス名からキーワードを抽出してマッチング（開発用の簡易実装）
        // 本番では service_config_mappings テーブルによる明示的マッピングを使用
        // 例: "auth-server" -> ["auth", "server"] -> namespace に含まれるかチェック
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
            })
            .collect();

        if matched.is_empty() {
            return Err(anyhow::anyhow!("service not found: {}", service_name));
        }

        Ok(ServiceConfigResult {
            service_name: service_name.to_string(),
            entries: matched,
        })
    }

    async fn record_change_log(&self, _log: &ConfigChangeLog) -> anyhow::Result<()> {
        // In-memory: ログは捨てる（開発用）
        Ok(())
    }

    async fn list_change_logs(
        &self,
        _namespace: &str,
        _key: &str,
    ) -> anyhow::Result<Vec<ConfigChangeLog>> {
        // In-memory: 空リストを返す（開発用）
        Ok(vec![])
    }

    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<ConfigEntry>> {
        let entries = self.entries.read().await;
        Ok(entries.iter().find(|e| e.id == *id).cloned())
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
impl domain::repository::ConfigSchemaRepository for InMemoryConfigSchemaRepository {
    async fn find_by_service_name(
        &self,
        service_name: &str,
    ) -> anyhow::Result<Option<ConfigSchema>> {
        let schemas = self.schemas.read().await;
        Ok(schemas
            .iter()
            .find(|s| s.service_name == service_name)
            .cloned())
    }

    async fn list_all(&self) -> anyhow::Result<Vec<ConfigSchema>> {
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
