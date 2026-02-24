use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

mod adapter;
mod domain;
mod infra;
mod usecase;

use adapter::graphql_handler;
use infra::auth::JwksVerifier;
use infra::config::Config;
use infra::grpc::{ConfigGrpcClient, FeatureFlagGrpcClient, TenantGrpcClient};
use usecase::{
    ConfigQueryResolver, FeatureFlagQueryResolver, SubscriptionResolver, TenantMutationResolver,
    TenantQueryResolver,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // --- Telemetry ---
    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-graphql-gateway-server".to_string(),
        version: "0.1.0".to_string(),
        tier: "system".to_string(),
        environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string()),
        trace_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
        sample_rate: 1.0,
        log_level: "info".to_string(),
    };
    k1s0_telemetry::init_telemetry(&telemetry_cfg).expect("failed to init telemetry");

    // --- Config ---
    let cfg = Config::load("config/config.yaml")?;
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

    // --- JWT 検証 ---
    let jwks_verifier = Arc::new(JwksVerifier::new(cfg.auth.jwks_url.clone()));

    // --- Resolver DI ---
    let tenant_query = Arc::new(TenantQueryResolver::new(tenant_client.clone()));
    let feature_flag_query = Arc::new(FeatureFlagQueryResolver::new(feature_flag_client.clone()));
    let config_query = Arc::new(ConfigQueryResolver::new(config_client.clone()));
    let tenant_mutation = Arc::new(TenantMutationResolver::new(tenant_client.clone()));
    let subscription = Arc::new(SubscriptionResolver::new(config_client.clone()));

    // --- Router ---
    let app = graphql_handler::router(
        jwks_verifier,
        tenant_query,
        feature_flag_query,
        config_query,
        tenant_mutation,
        subscription,
        feature_flag_client,
        cfg.graphql.clone(),
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
