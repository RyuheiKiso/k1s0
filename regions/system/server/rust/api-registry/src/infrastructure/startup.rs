use anyhow::Context;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use k1s0_server_common::middleware::grpc_auth::GrpcAuthLayer;
use k1s0_server_common::middleware::rbac::Tier;

use super::cache::SchemaCache;
use super::config::Config;
use super::database::create_pool;
use super::kafka::{KafkaSchemaEventPublisher, NoopSchemaEventPublisher, SchemaEventPublisher};
use crate::adapter;
use crate::adapter::grpc::{ApiRegistryGrpcService, ApiRegistryServiceTonic};
use crate::adapter::repository::cached_schema_repository::CachedSchemaRepository;
use crate::adapter::repository::schema_postgres::SchemaPostgresRepository;
use crate::adapter::repository::version_postgres::VersionPostgresRepository;
use crate::domain::repository::{ApiSchemaRepository, ApiSchemaVersionRepository};
use crate::proto::k1s0::system::apiregistry::v1::api_registry_service_server::ApiRegistryServiceServer;
use crate::usecase;

pub async fn run() -> anyhow::Result<()> {
    // Telemetry
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-api-registry-server".to_string(),
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
        environment = %cfg.app.environment,
        "starting api-registry server"
    );

    // DB pool
    let db_pool = if let Some(ref db_cfg) = cfg.database {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_cfg.connection_url());
        info!("connecting to database");
        let pool = create_pool(
            &url,
            db_cfg.max_open_conns,
            db_cfg.max_idle_conns,
            db_cfg.conn_max_lifetime,
        )
        .await?;
        info!("database connection established");
        Some(Arc::new(pool))
    } else if let Ok(url) = std::env::var("DATABASE_URL") {
        let pool = create_pool(&url, 10, 2, 300).await?;
        info!("database connection established from DATABASE_URL");
        Some(Arc::new(pool))
    } else {
        info!("no database configured");
        None
    };

    // Schema cache (max 5000 entries, TTL 600 seconds)
    let schema_cache = Arc::new(SchemaCache::new(5000, 600));

    // Repositories
    let (schema_repo, version_repo): (
        Arc<dyn ApiSchemaRepository>,
        Arc<dyn ApiSchemaVersionRepository>,
    ) = if let Some(ref pool) = db_pool {
        let inner_schema: Arc<dyn ApiSchemaRepository> =
            Arc::new(SchemaPostgresRepository::new(pool.clone()));
        (
            Arc::new(CachedSchemaRepository::new(inner_schema, schema_cache)),
            Arc::new(VersionPostgresRepository::new(pool.clone())),
        )
    } else {
        anyhow::bail!(
            "Database is required. Set DATABASE_URL or configure [database] in config.yaml"
        );
    };

    // Kafka publisher
    let publisher: Arc<dyn SchemaEventPublisher> = if let Some(ref kafka_cfg) = cfg.kafka {
        match KafkaSchemaEventPublisher::new(
            &kafka_cfg.brokers,
            &kafka_cfg.topic,
            Some(&kafka_cfg.security_protocol),
        ) {
            Ok(p) => {
                info!("Kafka schema event publisher enabled");
                Arc::new(p)
            }
            Err(e) => {
                tracing::warn!("Failed to create Kafka publisher, using noop: {}", e);
                Arc::new(NoopSchemaEventPublisher)
            }
        }
    } else {
        info!("no kafka configured, using noop publisher");
        Arc::new(NoopSchemaEventPublisher)
    };

    // Schema validator factory
    let validator_factory: Arc<dyn super::validator::SchemaValidatorFactory> =
        if let Some(ref validator_cfg) = cfg.validator {
            Arc::new(super::validator::ConfigurableSchemaValidatorFactory::new(
                validator_cfg.openapi_spec_validator_path.clone(),
                validator_cfg.buf_path.clone(),
                validator_cfg.timeout_seconds,
            ))
        } else {
            Arc::new(super::validator::DefaultSchemaValidatorFactory)
        };

    // Use cases
    let list_schemas_uc = Arc::new(usecase::ListSchemasUseCase::new(schema_repo.clone()));
    let register_schema_uc = Arc::new(
        usecase::RegisterSchemaUseCase::with_publisher(
            schema_repo.clone(),
            version_repo.clone(),
            publisher.clone(),
        )
        .with_validator(validator_factory.clone()),
    );
    let get_schema_uc = Arc::new(usecase::GetSchemaUseCase::new(
        schema_repo.clone(),
        version_repo.clone(),
    ));
    let list_versions_uc = Arc::new(usecase::ListVersionsUseCase::new(
        schema_repo.clone(),
        version_repo.clone(),
    ));
    let register_version_uc = Arc::new(
        usecase::RegisterVersionUseCase::with_publisher(
            schema_repo.clone(),
            version_repo.clone(),
            publisher.clone(),
        )
        .with_validator(validator_factory.clone()),
    );
    let get_schema_version_uc =
        Arc::new(usecase::GetSchemaVersionUseCase::new(version_repo.clone()));
    let delete_version_uc = Arc::new(usecase::DeleteVersionUseCase::with_publisher(
        schema_repo.clone(),
        version_repo.clone(),
        publisher.clone(),
    ));
    let check_compatibility_uc = Arc::new(usecase::CheckCompatibilityUseCase::new(
        schema_repo.clone(),
        version_repo.clone(),
    ));
    let get_diff_uc = Arc::new(usecase::GetDiffUseCase::new(
        schema_repo.clone(),
        version_repo.clone(),
    ));

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-api-registry-server",
    ));

    // Token verifier (JWKS verifier if auth configured)
    let auth_state = k1s0_server_common::require_auth_state(
        "api-registry",
        &cfg.app.environment,
        cfg.auth
            .as_ref()
            .map(|auth_cfg| -> anyhow::Result<_> {
                // JWKS URL を取得（nested 形式: auth.jwks.url）
                let jwks_url = auth_cfg
                    .jwks
                    .as_ref()
                    .map(|j| j.url.as_str())
                    .unwrap_or_default();
                let cache_ttl = auth_cfg
                    .jwks
                    .as_ref()
                    .map(|j| j.cache_ttl_secs)
                    .unwrap_or(300);
                info!(jwks_url = %jwks_url, "initializing JWKS verifier for api-registry");
                let jwks_verifier = Arc::new(
                    k1s0_auth::JwksVerifier::new(
                        jwks_url,
                        &auth_cfg.jwt.issuer,
                        &auth_cfg.jwt.audience,
                        std::time::Duration::from_secs(cache_ttl),
                    )
                    .context("JWKS 検証器の作成に失敗")?,
                );
                Ok(crate::adapter::middleware::auth::AuthState {
                    verifier: jwks_verifier,
                })
            })
            .transpose()?,
    )?;

    // gRPC 認証レイヤー: メソッド名をアクション（read/write）にマッピングして RBAC チェックを行う
    let grpc_auth_layer =
        GrpcAuthLayer::new(auth_state.clone(), Tier::System, api_registry_grpc_action);

    // REST app state
    let mut state = adapter::handler::AppState {
        list_schemas_uc: list_schemas_uc.clone(),
        register_schema_uc: register_schema_uc.clone(),
        get_schema_uc: get_schema_uc.clone(),
        list_versions_uc: list_versions_uc.clone(),
        register_version_uc: register_version_uc.clone(),
        get_schema_version_uc: get_schema_version_uc.clone(),
        delete_version_uc: delete_version_uc.clone(),
        check_compatibility_uc: check_compatibility_uc.clone(),
        get_diff_uc: get_diff_uc.clone(),
        metrics: metrics.clone(),
        auth_state: None,
        // CRITICAL-003 対応: /readyz で DB 疎通確認に使用する（Arc<PgPool> から PgPool を取り出す）
        db_pool: db_pool.as_ref().map(|p| (**p).clone()),
    };
    if let Some(auth_st) = auth_state {
        state = state.with_auth(auth_st);
    }

    let app = adapter::handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
        .layer(k1s0_correlation::layer::CorrelationLayer::new());

    // gRPC service
    let grpc_svc = Arc::new(ApiRegistryGrpcService::new(
        list_schemas_uc,
        register_schema_uc,
        get_schema_uc,
        list_versions_uc,
        register_version_uc,
        get_schema_version_uc,
        delete_version_uc,
        check_compatibility_uc,
        get_diff_uc,
    ));
    let tonic_svc = ApiRegistryServiceTonic::new(grpc_svc);

    // gRPC server
    let grpc_port = cfg.server.grpc_port;
    let grpc_addr: SocketAddr = format!("0.0.0.0:{}", grpc_port).parse()?;
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(ApiRegistryServiceServer::new(tonic_svc))
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

    k1s0_telemetry::shutdown();

    Ok(())
}

/// gRPC メソッド名を RBAC アクション（read/write）にマッピングする。
/// スキーマ・バージョンの登録・削除は write、それ以外は read とする。
fn api_registry_grpc_action(method: &str) -> &'static str {
    match method {
        "RegisterSchema" | "RegisterVersion" | "DeleteVersion" => "write",
        _ => "read",
    }
}
