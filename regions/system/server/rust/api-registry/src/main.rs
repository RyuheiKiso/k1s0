use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use k1s0_api_registry_server::adapter;
use k1s0_api_registry_server::adapter::repository::schema_postgres::SchemaPostgresRepository;
use k1s0_api_registry_server::adapter::repository::version_postgres::VersionPostgresRepository;
use k1s0_api_registry_server::domain::repository::{ApiSchemaRepository, ApiSchemaVersionRepository};
use k1s0_api_registry_server::infrastructure::config::Config;
use k1s0_api_registry_server::infrastructure::database::create_pool;
use k1s0_api_registry_server::infrastructure::kafka::{
    KafkaSchemaEventPublisher, NoopSchemaEventPublisher, SchemaEventPublisher,
};
use k1s0_api_registry_server::proto::k1s0::system::apiregistry::v1::api_registry_service_server::ApiRegistryServiceServer;
use k1s0_api_registry_server::usecase;
use k1s0_api_registry_server::adapter::grpc::{ApiRegistryGrpcService, ApiRegistryServiceTonic};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Telemetry
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-api-registry-server".to_string(),
        version: "0.1.0".to_string(),
        tier: "system".to_string(),
        environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string()),
        trace_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
        sample_rate: 1.0,
        log_level: "info".to_string(),
    };
    k1s0_telemetry::init_telemetry(&telemetry_cfg).expect("failed to init telemetry");

    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

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
        let pool = create_pool(&url, db_cfg.max_open_conns).await?;
        info!("database connection established");
        Some(Arc::new(pool))
    } else if let Ok(url) = std::env::var("DATABASE_URL") {
        let pool = create_pool(&url, 10).await?;
        info!("database connection established from DATABASE_URL");
        Some(Arc::new(pool))
    } else {
        info!("no database configured");
        None
    };

    // Repositories
    let (schema_repo, version_repo): (
        Arc<dyn ApiSchemaRepository>,
        Arc<dyn ApiSchemaVersionRepository>,
    ) = if let Some(ref pool) = db_pool {
        (
            Arc::new(SchemaPostgresRepository::new(pool.clone())),
            Arc::new(VersionPostgresRepository::new(pool.clone())),
        )
    } else {
        anyhow::bail!("Database is required. Set DATABASE_URL or configure [database] in config.yaml");
    };

    // Kafka publisher
    let publisher: Arc<dyn SchemaEventPublisher> = if let Some(ref kafka_cfg) = cfg.kafka {
        match KafkaSchemaEventPublisher::new(&kafka_cfg.brokers, &kafka_cfg.schema_updated_topic) {
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

    // Use cases
    let list_schemas_uc = Arc::new(usecase::ListSchemasUseCase::new(schema_repo.clone()));
    let register_schema_uc = Arc::new(usecase::RegisterSchemaUseCase::with_publisher(
        schema_repo.clone(),
        version_repo.clone(),
        publisher.clone(),
    ));
    let get_schema_uc = Arc::new(usecase::GetSchemaUseCase::new(
        schema_repo.clone(),
        version_repo.clone(),
    ));
    let list_versions_uc = Arc::new(usecase::ListVersionsUseCase::new(
        schema_repo.clone(),
        version_repo.clone(),
    ));
    let register_version_uc = Arc::new(usecase::RegisterVersionUseCase::with_publisher(
        schema_repo.clone(),
        version_repo.clone(),
        publisher.clone(),
    ));
    let get_schema_version_uc = Arc::new(usecase::GetSchemaVersionUseCase::new(
        version_repo.clone(),
    ));
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

    // REST app state
    let state = adapter::handler::AppState {
        list_schemas_uc,
        register_schema_uc,
        get_schema_uc: get_schema_uc.clone(),
        list_versions_uc,
        register_version_uc,
        get_schema_version_uc: get_schema_version_uc.clone(),
        delete_version_uc,
        check_compatibility_uc: check_compatibility_uc.clone(),
        get_diff_uc,
        metrics: metrics.clone(),
    };

    let app = adapter::handler::router(state)
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    // gRPC service
    let grpc_svc = Arc::new(ApiRegistryGrpcService::new(
        get_schema_uc,
        get_schema_version_uc,
        check_compatibility_uc,
    ));
    let tonic_svc = ApiRegistryServiceTonic::new(grpc_svc);

    // gRPC server
    let grpc_port = cfg.server.grpc_port;
    let grpc_addr: SocketAddr = format!("0.0.0.0:{}", grpc_port).parse()?;
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_future = async move {
        tonic::transport::Server::builder()
            .add_service(ApiRegistryServiceServer::new(tonic_svc))
            .serve(grpc_addr)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    // REST server
    let rest_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port).parse()?;
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
