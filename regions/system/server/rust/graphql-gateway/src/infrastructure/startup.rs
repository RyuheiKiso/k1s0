use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use crate::adapter::graphql_handler;
use super::auth::JwksVerifier;
use super::config::Config;
use super::grpc::{
    ConfigGrpcClient, FeatureFlagGrpcClient, NavigationGrpcClient, ServiceCatalogGrpcClient,
    TenantGrpcClient,
};
use crate::usecase::{
    ConfigQueryResolver, FeatureFlagQueryResolver, NavigationQueryResolver,
    ServiceCatalogMutationResolver, ServiceCatalogQueryResolver, SubscriptionResolver,
    TenantMutationResolver, TenantQueryResolver,
};

pub async fn run() -> anyhow::Result<()> {
    // --- Telemetry ---
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-graphql-gateway-server".to_string(),
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

    // --- Config ---
    cfg.validate()?;

    info!(
        app_name = %cfg.app.name,
        version = %cfg.app.version,
        environment = %cfg.app.environment,
        "starting graphql-gateway server"
    );

    // --- gRPC クライアント ---
    let tenant_client = Arc::new(TenantGrpcClient::connect(&cfg.backends.tenant).await?);
    let feature_flag_client =
        Arc::new(FeatureFlagGrpcClient::connect(&cfg.backends.featureflag).await?);
    let config_client = Arc::new(ConfigGrpcClient::connect(&cfg.backends.config).await?);
    let navigation_client =
        Arc::new(NavigationGrpcClient::connect(&cfg.backends.navigation).await?);
    let service_catalog_client =
        Arc::new(ServiceCatalogGrpcClient::connect(&cfg.backends.service_catalog).await?);

    // --- JWT 検証 ---
    let jwks_verifier = Arc::new(JwksVerifier::new(cfg.auth.jwks_url.clone()));

    // --- Metrics ---
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-graphql-gateway-server",
    ));

    // --- Resolver DI ---
    let tenant_query = Arc::new(TenantQueryResolver::new(tenant_client.clone()));
    let feature_flag_query = Arc::new(FeatureFlagQueryResolver::new(feature_flag_client.clone()));
    let config_query = Arc::new(ConfigQueryResolver::new(config_client.clone()));
    let navigation_query = Arc::new(NavigationQueryResolver::new(navigation_client.clone()));
    let service_catalog_query =
        Arc::new(ServiceCatalogQueryResolver::new(service_catalog_client.clone()));
    let tenant_mutation = Arc::new(TenantMutationResolver::new(tenant_client.clone()));
    let service_catalog_mutation =
        Arc::new(ServiceCatalogMutationResolver::new(service_catalog_client.clone()));
    let subscription = Arc::new(SubscriptionResolver::new(
        config_client.clone(),
        tenant_client.clone(),
        feature_flag_client.clone(),
    ));

    // --- Router ---
    let app = graphql_handler::router(
        jwks_verifier,
        tenant_query,
        feature_flag_query,
        config_query,
        navigation_query,
        service_catalog_query,
        tenant_mutation,
        service_catalog_mutation,
        subscription,
        feature_flag_client,
        tenant_client,
        config_client,
        navigation_client,
        service_catalog_client,
        cfg.graphql.clone(),
        metrics,
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("graphql-gateway starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("graphql-gateway exited");
    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = signal::ctrl_c() => {},
        _ = terminate => {},
    }
    info!("shutdown signal received");
}
