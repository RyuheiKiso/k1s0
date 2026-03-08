use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

use k1s0_domain_master_server::adapter;
use k1s0_domain_master_server::domain;
use k1s0_domain_master_server::infrastructure;
use k1s0_domain_master_server::usecase;

use adapter::handler::{self, AppState};
use infrastructure::config::{Config, DatabaseConfig};
use k1s0_domain_master_server::MIGRATOR;
use k1s0_server_common::middleware::auth_middleware::AuthState;
use k1s0_server_common::middleware::grpc_auth::GrpcAuthLayer;
use k1s0_server_common::middleware::rbac::Tier;
use k1s0_server_common::middleware::shutdown::shutdown_signal;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Telemetry
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-domain-master-server".to_string(),
        version: "0.1.0".to_string(),
        tier: "business".to_string(),
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

    // 2. Config
    info!("starting {}", cfg.app.name);

    // 3. Database
    let db_cfg = cfg
        .database
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("database configuration is required"))?;
    let db_pool = connect_database(db_cfg).await?;
    MIGRATOR.run(&db_pool).await?;
    info!(
        schema = %db_cfg.schema,
        migrations_path = %DatabaseConfig::migrations_path().display(),
        "database connected and migrations applied"
    );

    // 4. Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("domain_master"));

    // 5. Repositories
    let category_repo: Arc<dyn domain::repository::category_repository::CategoryRepository> =
        Arc::new(
            infrastructure::persistence::category_repo_impl::CategoryPostgresRepository::new(
                db_pool.clone(),
            ),
        );

    let item_repo: Arc<dyn domain::repository::item_repository::ItemRepository> = Arc::new(
        infrastructure::persistence::item_repo_impl::ItemPostgresRepository::new(db_pool.clone()),
    );

    let version_repo: Arc<dyn domain::repository::version_repository::VersionRepository> =
        Arc::new(
            infrastructure::persistence::version_repo_impl::VersionPostgresRepository::new(
                db_pool.clone(),
            ),
        );

    let tenant_ext_repo: Arc<
        dyn domain::repository::tenant_extension_repository::TenantExtensionRepository,
    > = Arc::new(
        infrastructure::persistence::tenant_extension_repo_impl::TenantExtensionPostgresRepository::new(db_pool.clone()),
    );

    // 6. Kafka Producer (optional)
    let event_publisher: Arc<dyn usecase::event_publisher::DomainMasterEventPublisher> =
        if let Some(ref kafka_cfg) = cfg.kafka {
        match infrastructure::messaging::kafka_producer::DomainMasterKafkaProducer::new(kafka_cfg) {
            Ok(producer) => {
                info!("kafka producer initialized");
                Arc::new(producer)
            }
            Err(e) => {
                tracing::warn!("failed to initialize kafka producer: {}", e);
                Arc::new(usecase::event_publisher::NoopDomainMasterEventPublisher)
            }
        }
    } else {
        Arc::new(usecase::event_publisher::NoopDomainMasterEventPublisher)
    };

    // 7. Use Cases
    let manage_categories_uc = Arc::new(usecase::manage_categories::ManageCategoriesUseCase::new(
        category_repo.clone(),
        event_publisher.clone(),
    ));
    let manage_items_uc = Arc::new(usecase::manage_items::ManageItemsUseCase::new(
        category_repo.clone(),
        item_repo.clone(),
        version_repo.clone(),
        event_publisher.clone(),
    ));
    let get_item_versions_uc = Arc::new(
        usecase::get_item_versions::GetItemVersionsUseCase::new(
            category_repo.clone(),
            item_repo.clone(),
            version_repo.clone(),
        ),
    );
    let manage_tenant_extensions_uc = Arc::new(
        usecase::manage_tenant_extensions::ManageTenantExtensionsUseCase::new(
            category_repo.clone(),
            item_repo.clone(),
            tenant_ext_repo.clone(),
            event_publisher.clone(),
        ),
    );

    // 8. Auth
    let auth_cfg = cfg
        .auth
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("auth configuration is required"))?;
    let verifier = Arc::new(k1s0_auth::JwksVerifier::new(
        &auth_cfg.jwks_url,
        &auth_cfg.issuer,
        &auth_cfg.audience,
        std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
    ));
    let auth_state = Some(AuthState { verifier });

    // 9. AppState + Router
    let state = AppState {
        manage_categories_uc: manage_categories_uc.clone(),
        manage_items_uc: manage_items_uc.clone(),
        get_item_versions_uc: get_item_versions_uc.clone(),
        manage_tenant_extensions_uc: manage_tenant_extensions_uc.clone(),
        metrics: metrics.clone(),
        auth_state: auth_state.clone(),
    };
    let app = handler::router(state);

    // 10. gRPC Service
    let grpc_service =
        adapter::grpc::domain_master_grpc::DomainMasterGrpcService::new(
            manage_categories_uc,
            manage_items_uc,
            get_item_versions_uc,
            manage_tenant_extensions_uc,
        );
    let grpc_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.grpc_port).parse()?;
    info!("gRPC server listening on {}", grpc_addr);
    let grpc_metrics = metrics.clone();
    let grpc_auth_layer = GrpcAuthLayer::new(auth_state.clone(), Tier::Business, required_action);
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let mut rest_shutdown_rx = shutdown_rx.clone();
    let mut grpc_shutdown_rx = shutdown_rx.clone();
    let grpc_future = async move {
        use k1s0_domain_master_server::proto::k1s0::business::accounting::domainmaster::v1::domain_master_service_server::DomainMasterServiceServer;

        Server::builder()
            .layer(grpc_auth_layer)
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(DomainMasterServiceServer::new(grpc_service))
            .serve_with_shutdown(grpc_addr, async move {
                let _ = grpc_shutdown_rx.changed().await;
            })
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    // 11. Start REST server
    let rest_addr: SocketAddr = format!("{}:{}", cfg.server.host, cfg.server.port).parse()?;
    info!("REST server listening on {}", rest_addr);
    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    let rest_future = axum::serve(listener, app).with_graceful_shutdown(async move {
        let _ = rest_shutdown_rx.changed().await;
    });

    let shutdown_future = async move {
        shutdown_signal().await.map_err(|e| anyhow::anyhow!("{}", e))?;
        let _ = shutdown_tx.send(true);
        Ok::<(), anyhow::Error>(())
    };

    tokio::select! {
        result = shutdown_future => {
            result?;
        }
        result = rest_future => {
            if let Err(e) = result {
                return Err(anyhow::anyhow!("REST server error: {}", e));
            }
        }
        result = grpc_future => {
            result?;
        }
    }

    Ok(())
}

/// gRPC メソッド名 → 必要なアクションのマッピング（domain-master 固有）。
fn required_action(method: &str) -> &'static str {
    match method {
        "ListCategories" | "GetCategory" | "ListItems" | "GetItem" | "ListItemVersions"
        | "GetTenantExtension" | "ListTenantItems" => "read",
        "DeleteCategory" | "DeleteItem" | "DeleteTenantExtension" => "admin",
        _ => "write",
    }
}

async fn connect_database(db_cfg: &DatabaseConfig) -> anyhow::Result<sqlx::PgPool> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| db_cfg.connection_url());
    let lifetime = Duration::from_secs(db_cfg.conn_max_lifetime);
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(db_cfg.max_connections)
        .min_connections(db_cfg.max_idle_conns.min(db_cfg.max_connections))
        .idle_timeout(Some(lifetime))
        .max_lifetime(Some(lifetime))
        .connect(&url)
        .await?;
    Ok(pool)
}
