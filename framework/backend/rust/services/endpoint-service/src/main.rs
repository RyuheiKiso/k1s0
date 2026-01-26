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

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::signal;
use tonic::transport::Server;
use tracing::{info, warn};

mod application;
mod domain;
mod infrastructure;
mod presentation;

use application::EndpointService;
use domain::{Endpoint, EndpointRepository};
use infrastructure::InMemoryRepository;
use presentation::grpc::endpoint_v1::endpoint_service_server::EndpointServiceServer;
use presentation::grpc::GrpcEndpointService;

/// サービス設定
struct ServiceConfig {
    env_name: String,
    config_path: Option<PathBuf>,
    secrets_dir: Option<PathBuf>,
    namespace: String,
    cluster_domain: String,
    grpc_port: u16,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            env_name: "dev".to_string(),
            config_path: None,
            secrets_dir: None,
            namespace: "default".to_string(),
            cluster_domain: "cluster.local".to_string(),
            grpc_port: 50052,
        }
    }
}

fn parse_args() -> ServiceConfig {
    let args: Vec<String> = env::args().collect();
    let mut config = ServiceConfig::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--env" => {
                if i + 1 < args.len() {
                    config.env_name = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --env requires a value");
                    std::process::exit(1);
                }
            }
            "--config" => {
                if i + 1 < args.len() {
                    config.config_path = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("Error: --config requires a value");
                    std::process::exit(1);
                }
            }
            "--secrets-dir" => {
                if i + 1 < args.len() {
                    config.secrets_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("Error: --secrets-dir requires a value");
                    std::process::exit(1);
                }
            }
            "--namespace" => {
                if i + 1 < args.len() {
                    config.namespace = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --namespace requires a value");
                    std::process::exit(1);
                }
            }
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    config.grpc_port = args[i + 1].parse().unwrap_or(50052);
                    i += 2;
                } else {
                    eprintln!("Error: --port requires a value");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                println!("endpoint-service - k1s0 framework endpoint discovery service");
                println!();
                println!("USAGE:");
                println!("    endpoint-service [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("    --env <ENV>           Environment name (default: dev)");
                println!("    --config <PATH>       Path to config file");
                println!("    --secrets-dir <PATH>  Path to secrets directory");
                println!("    --namespace <NS>      Kubernetes namespace (default: default)");
                println!("    -p, --port <PORT>     gRPC port (default: 50052)");
                println!("    -h, --help            Print help information");
                std::process::exit(0);
            }
            _ => {
                eprintln!("Error: Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    config
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args();

    println!("endpoint-service starting...");
    println!("  Environment: {}", config.env_name);
    println!("  Namespace: {}", config.namespace);
    println!("  Port: {}", config.grpc_port);
    if let Some(ref path) = config.config_path {
        println!("  Config: {}", path.display());
    }
    if let Some(ref path) = config.secrets_dir {
        println!("  Secrets: {}", path.display());
    }

    // リポジトリの初期化
    let repository = Arc::new(InMemoryRepository::new());

    // サンプルエンドポイントの登録
    repository
        .save(&Endpoint::new(1, "auth-service", "/v1/login", "POST"))
        .await?;
    repository
        .save(&Endpoint::new(2, "auth-service", "/v1/logout", "POST"))
        .await?;
    repository
        .save(&Endpoint::new(3, "config-service", "/v1/settings", "GET"))
        .await?;

    // サービスの初期化
    let service = EndpointService::new(
        repository.clone(),
        &config.namespace,
        &config.cluster_domain,
    );

    // 動作確認
    println!("\nEndpoint resolution test:");
    let resolved = service.resolve_endpoint("auth-service", "grpc").await?;
    println!("  auth-service (grpc) -> {}", resolved.address);

    let resolved = service.resolve_endpoint("api-gateway", "http").await?;
    println!("  api-gateway (http) -> {}", resolved.address);

    // gRPC サービスの作成
    let grpc_service = GrpcEndpointService::new(Arc::new(service));

    // gRPC サーバーの起動
    let addr: SocketAddr = format!("0.0.0.0:{}", config.grpc_port).parse()?;
    println!("\nStarting gRPC server on {}", addr);

    Server::builder()
        .add_service(EndpointServiceServer::new(grpc_service))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    println!("endpoint-service shutdown complete.");
    Ok(())
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
            println!("\nReceived Ctrl+C, shutting down...");
        }
        _ = terminate => {
            println!("\nReceived SIGTERM, shutting down...");
        }
    }
}
