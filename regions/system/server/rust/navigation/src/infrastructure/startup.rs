use anyhow::Context;
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;

use super::config::Config;
use super::navigation_loader::YamlNavigationConfigLoader;
use crate::adapter;
use crate::proto::k1s0::system::navigation::v1::navigation_service_server::NavigationServiceServer;
use crate::usecase;
use crate::usecase::get_navigation::JwksNavigationTokenVerifier;

// HIGH-001 監査対応: startup 関数は実装上 too_many_lines を超えるが分割は可読性を損なうため許容する
#[allow(clippy::too_many_lines)]
pub async fn run() -> anyhow::Result<()> {
    // Telemetry
    let config_path =
        std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config/config.yaml".to_string());
    let cfg = Config::load(&config_path)?;

    let telemetry_cfg = k1s0_telemetry::TelemetryConfig {
        service_name: "k1s0-navigation-server".to_string(),
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
        .map_err(|e| anyhow::anyhow!("テレメトリの初期化に失敗: {e}"))?;

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
            cfg.auth
                .as_ref()
                .map(|auth_cfg| -> anyhow::Result<_> {
                    // nested 形式の AuthConfig から JWKS 検証器を初期化する
                    let jwks = auth_cfg
                        .jwks
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("auth.jwks configuration is required"))?;
                    info!(jwks_url = %jwks.url, "initializing JWKS verifier");
                    Ok(Arc::new(JwksNavigationTokenVerifier::new(Arc::new(
                        k1s0_auth::JwksVerifier::new(
                            &jwks.url,
                            &auth_cfg.jwt.issuer,
                            &auth_cfg.jwt.audience,
                            std::time::Duration::from_secs(jwks.cache_ttl_secs),
                        )
                        .context("JWKS 検証器の作成に失敗")?,
                    )))
                        as Arc<
                            dyn usecase::get_navigation::NavigationTokenVerifier,
                        >)
                })
                .transpose()?,
        )?;

    // Use case
    let get_navigation_uc = Arc::new(usecase::GetNavigationUseCase::new(loader, verifier));

    // gRPC service
    let grpc_svc = Arc::new(adapter::grpc::NavigationGrpcService::new(
        get_navigation_uc.clone(),
    ));
    let navigation_tonic = adapter::grpc::NavigationServiceTonic::new(grpc_svc);


    // REST (health/readyz/metrics only)
    let rest_state = adapter::handler::AppState {
        metrics: metrics.clone(),
        get_navigation_uc: get_navigation_uc.clone(),
    };
    // REST router（メトリクスレイヤーとCorrelation IDレイヤーを追加）
    let app = adapter::handler::router(
        rest_state,
        cfg.observability.metrics.enabled,
        &cfg.observability.metrics.path,
    )
    .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
    .layer(k1s0_correlation::layer::CorrelationLayer::new());

    // gRPC server
    let grpc_addr: SocketAddr = ([0, 0, 0, 0], cfg.server.grpc_port).into();
    info!("gRPC server starting on {}", grpc_addr);

    // gRPC Health Check Protocol サービスを登録する。
    // readyz エンドポイントや Kubernetes の livenessProbe/readinessProbe が
    // Bearer token なしでヘルスチェックできるようにするため。
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<NavigationServiceServer<adapter::grpc::NavigationServiceTonic>>()
        .await;

    // gRPC グレースフルシャットダウン用シグナル
    let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();
    let grpc_metrics = metrics;
    let grpc_future = async move {
        tonic::transport::Server::builder()
            .layer(k1s0_telemetry::GrpcMetricsLayer::new(grpc_metrics))
            .add_service(health_service)
            .add_service(NavigationServiceServer::new(navigation_tonic))
            .serve_with_shutdown(grpc_addr, async move {
                let _ = grpc_shutdown.await;
            })
            .await
            .map_err(|e| anyhow::anyhow!("gRPC server error: {e}"))
    };

    // REST server
    let rest_addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("REST server starting on {}", rest_addr);

    let listener = tokio::net::TcpListener::bind(rest_addr).await?;
    // REST グレースフルシャットダウンを設定
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

    // テレメトリのフラッシュとシャットダウン
    k1s0_telemetry::shutdown();

    Ok(())
}
