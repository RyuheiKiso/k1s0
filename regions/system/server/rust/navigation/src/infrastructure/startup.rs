use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use super::config::Config;
use super::navigation_loader::YamlNavigationConfigLoader;
use crate::usecase;
use crate::usecase::get_navigation::JwksNavigationTokenVerifier;
use crate::adapter;
use crate::proto;

pub async fn run() -> anyhow::Result<()> {
    // Telemetry
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-navigation-server".to_string(),
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
        "starting navigation server"
    );

    // Metrics
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new(
        "k1s0-navigation-server",
    ));

    // Navigation config loader
    let loader = Arc::new(YamlNavigationConfigLoader::new(
        &cfg.navigation.navigation_path,
    ));

    // Token verifier (optional)
    let verifier: Option<Arc<dyn usecase::get_navigation::NavigationTokenVerifier>> =
        k1s0_server_common::require_auth_state(
        "navigation-server",
        &cfg.app.environment,
        cfg.auth.as_ref().map(|auth_cfg| {
            info!(jwks_url = %auth_cfg.jwks_url, "initializing JWKS verifier");
            Arc::new(JwksNavigationTokenVerifier::new(Arc::new(k1s0_auth::JwksVerifier::new(
                &auth_cfg.jwks_url,
                &auth_cfg.issuer,
                &auth_cfg.audience,
                std::time::Duration::from_secs(auth_cfg.jwks_cache_ttl_secs),
            )))) as Arc<dyn usecase::get_navigation::NavigationTokenVerifier>
        }),
    )?;

    // Use case
    let get_navigation_uc = Arc::new(usecase::GetNavigationUseCase::new(loader, verifier));

    // gRPC service
    let grpc_svc = Arc::new(adapter::grpc::NavigationGrpcService::new(
        get_navigation_uc.clone(),
    ));
    let navigation_tonic = adapter::grpc::NavigationServiceTonic::new(grpc_svc);

    use proto::k1s0::system::navigation::v1::navigation_service_server::NavigationServiceServer;

    // REST (health/readyz/metrics only)
    let rest_state = adapter::handler::AppState {
        metrics: metrics.clone(),
        get_navigation_uc: get_navigation_uc.clone(),
    };
    let app = adapter::handler::router(
        rest_state,
        cfg.observability.metrics.enabled,
        &cfg.observability.metrics.path,
    )
        .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()));

    // gRPC server
    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(NavigationServiceServer::new(navigation_tonic))
            .serve(grpc_addr)
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
    };

    // REST server
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
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
