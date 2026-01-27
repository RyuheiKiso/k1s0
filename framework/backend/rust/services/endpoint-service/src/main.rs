//! endpoint-service - k1s0 framework endpoint discovery service
//!
//! This service provides:
//! - Endpoint information retrieval (Get/List)
//! - Service name to endpoint resolution
//!
//! # 起動方法
//!
//! ```bash
//! endpoint-service --env dev --port 50052
//! ```

use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use tokio::signal;
use tonic::transport::Server;
use tonic_health::server::health_reporter;
use tracing::{info, warn};

mod application;
mod domain;
mod infrastructure;
mod presentation;

use application::EndpointService;
use domain::{Endpoint, EndpointQuery, EndpointRepository};
use infrastructure::InMemoryRepository;
use presentation::grpc::endpoint_v1::endpoint_service_server::EndpointServiceServer;
use presentation::grpc::GrpcEndpointService;

/// endpoint-service CLI arguments
#[derive(Parser, Debug)]
#[command(name = "endpoint-service")]
#[command(about = "k1s0 framework endpoint discovery service")]
#[command(version)]
struct Args {
    /// Environment name (dev, stg, prod)
    #[arg(long, default_value = "dev")]
    env: String,

    /// Path to config file
    #[arg(long)]
    config: Option<String>,

    /// Path to secrets directory
    #[arg(long)]
    secrets_dir: Option<String>,

    /// gRPC server port
    #[arg(short, long, default_value = "50052")]
    port: u16,

    /// REST API port (optional)
    #[arg(long)]
    rest_port: Option<u16>,

    /// Database URL (if not set, uses in-memory storage)
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

    /// Kubernetes namespace
    #[arg(long, default_value = "default")]
    namespace: String,

    /// Kubernetes cluster domain
    #[arg(long, default_value = "cluster.local")]
    cluster_domain: String,
}

/// InMemoryリポジトリを使用するEndpointService型
type InMemoryEndpointService = EndpointService<InMemoryRepository>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // トレーシング初期化
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    info!(
        service = "endpoint-service",
        env = %args.env,
        port = %args.port,
        namespace = %args.namespace,
        "Starting endpoint-service"
    );

    // データベースURL指定時はPostgreSQLを使用
    if let Some(ref db_url) = args.database_url {
        info!(database_url = %db_url, "PostgreSQL mode (not yet implemented in main)");
        warn!("PostgreSQL mode not fully implemented, falling back to in-memory");
    }

    // InMemoryリポジトリの初期化
    let repository = Arc::new(InMemoryRepository::new());

    // 開発環境の場合、サンプルデータを登録
    if args.env == "dev" {
        setup_sample_data(&repository).await?;
    }

    // サービスの初期化
    let service = EndpointService::new(
        Arc::clone(&repository),
        &args.namespace,
        &args.cluster_domain,
    );

    // 開発環境での動作確認
    if args.env == "dev" {
        run_dev_tests(&service).await;
    }

    // gRPC サービスの作成
    let grpc_service = GrpcEndpointService::new(Arc::new(service));

    // Health reporter
    let (mut health_reporter, health_service) = health_reporter();
    health_reporter
        .set_serving::<EndpointServiceServer<GrpcEndpointService<InMemoryRepository>>>()
        .await;

    // gRPC サーバーの起動
    let addr: SocketAddr = format!("0.0.0.0:{}", args.port).parse()?;
    info!(address = %addr, "Starting gRPC server");

    Server::builder()
        .add_service(health_service)
        .add_service(EndpointServiceServer::new(grpc_service))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    info!("endpoint-service shutdown complete");
    Ok(())
}

/// サンプルデータの登録（開発環境用）
async fn setup_sample_data(
    repository: &Arc<InMemoryRepository>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting up sample data for development");

    let samples = vec![
        Endpoint::new(1, "auth-service", "/v1/login", "POST"),
        Endpoint::new(2, "auth-service", "/v1/logout", "POST"),
        Endpoint::new(3, "auth-service", "/v1/refresh", "POST"),
        Endpoint::new(4, "config-service", "/v1/settings", "GET"),
        Endpoint::new(5, "config-service", "/v1/settings/{key}", "GET"),
        Endpoint::new(6, "endpoint-service", "/v1/endpoints", "GET"),
    ];

    for endpoint in samples {
        repository.save(&endpoint).await?;
    }

    info!(count = 6, "Sample data setup complete");
    Ok(())
}

/// 開発環境での動作確認テスト
async fn run_dev_tests(service: &InMemoryEndpointService) {
    info!("Running development tests");

    // エンドポイント解決のテスト
    match service.resolve_endpoint("auth-service", "grpc").await {
        Ok(resolved) => {
            info!(
                service_name = "auth-service",
                protocol = "grpc",
                address = %resolved.address,
                "Resolve endpoint test: SUCCESS"
            );
        }
        Err(e) => {
            warn!(error = %e, "Resolve endpoint test: FAILED");
        }
    }

    // エンドポイント一覧のテスト
    let query = EndpointQuery::new().with_service_name("auth-service");
    match service.list_endpoints(&query).await {
        Ok(list) => {
            info!(
                service_name = "auth-service",
                count = %list.endpoints.len(),
                "List endpoints test: SUCCESS"
            );
        }
        Err(e) => {
            warn!(error = %e, "List endpoints test: FAILED");
        }
    }
}

/// グレースフルシャットダウンシグナルを待機
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

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
        _ = ctrl_c => {
            info!("Received Ctrl+C, initiating graceful shutdown");
        }
        _ = terminate => {
            info!("Received SIGTERM, initiating graceful shutdown");
        }
    }
}
