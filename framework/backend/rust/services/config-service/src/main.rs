//! config-service - k1s0 framework dynamic configuration service
//!
//! This service provides:
//! - Setting retrieval (Get/List)
//! - Setting caching (in-memory)
//! - Setting change notification (WatchSettings)
//!
//! # 起動方法
//!
//! ```bash
//! config-service --env dev --port 50051
//! ```

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::signal;
use tonic::transport::Server;
use tracing::{info, warn};

mod application;
mod domain;
mod infrastructure;
mod presentation;

use application::ConfigService;
use domain::{Setting, SettingQuery, SettingRepository};
use infrastructure::{InMemoryCache, InMemoryRepository};
use presentation::grpc::config_v1::config_service_server::ConfigServiceServer;
use presentation::grpc::GrpcConfigService;

/// サービス設定
#[derive(Debug, Clone)]
struct ServiceConfig {
    env: String,
    config_path: Option<String>,
    secrets_dir: Option<String>,
    port: u16,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            env: "dev".to_string(),
            config_path: None,
            secrets_dir: None,
            port: 50051,
        }
    }
}

fn parse_args() -> ServiceConfig {
    let args: Vec<String> = std::env::args().collect();
    let mut config = ServiceConfig::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--env" | "-e" => {
                if i + 1 < args.len() {
                    config.env = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --env requires a value");
                    std::process::exit(1);
                }
            }
            "--config" | "-c" => {
                if i + 1 < args.len() {
                    config.config_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --config requires a value");
                    std::process::exit(1);
                }
            }
            "--secrets-dir" | "-s" => {
                if i + 1 < args.len() {
                    config.secrets_dir = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --secrets-dir requires a value");
                    std::process::exit(1);
                }
            }
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    config.port = args[i + 1].parse().unwrap_or(50051);
                    i += 2;
                } else {
                    eprintln!("Error: --port requires a value");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                println!("config-service - k1s0 framework configuration service");
                println!();
                println!("USAGE:");
                println!("    config-service [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("    -e, --env <ENV>           Environment name (default: dev)");
                println!("    -c, --config <PATH>       Path to config file");
                println!("    -s, --secrets-dir <PATH>  Path to secrets directory");
                println!("    -p, --port <PORT>         gRPC port (default: 50051)");
                println!("    -h, --help                Print help information");
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
async fn main() {
    let config = parse_args();

    println!("config-service starting...");
    println!("  Environment: {}", config.env);
    if let Some(ref path) = config.config_path {
        println!("  Config: {}", path);
    }
    if let Some(ref path) = config.secrets_dir {
        println!("  Secrets: {}", path);
    }
    println!("  Port: {}", config.port);

    // サービスの初期化
    let repository = Arc::new(InMemoryRepository::new());
    let cache = Arc::new(InMemoryCache::new());
    let service = ConfigService::new(
        Arc::clone(&repository),
        Arc::clone(&cache),
        &config.env,
    );

    // サンプルデータの追加（開発用）
    if config.env == "dev" {
        println!("  Loading sample data for development...");
        let samples = vec![
            Setting::new(1, "sample-service", "dev", "feature.enabled", "true"),
            Setting::new(2, "sample-service", "dev", "http.timeout_ms", "5000"),
            Setting::new(3, "sample-service", "dev", "db.pool_size", "10"),
        ];
        for setting in samples {
            repository.save(&setting).await.unwrap();
        }
    }

    // サービスの動作確認
    println!();
    println!("Testing service functionality...");

    // 設定取得のテスト
    match service.get_setting("sample-service", "feature.enabled", None).await {
        Ok(setting) => println!("  Get setting: {}={}", setting.key, setting.value),
        Err(e) => println!("  Get setting error: {}", e),
    }

    // 設定一覧のテスト
    let query = SettingQuery::new().with_service_name("sample-service");
    match service.list_settings(&query).await {
        Ok(list) => println!("  List settings: {} items", list.settings.len()),
        Err(e) => println!("  List settings error: {}", e),
    }

    // gRPC サービスの作成
    let grpc_service = GrpcConfigService::new(Arc::new(service));

    // gRPC サーバーの起動
    let addr: SocketAddr = format!("0.0.0.0:{}", config.port).parse().unwrap();
    println!();
    println!("Starting gRPC server on {}", addr);

    Server::builder()
        .add_service(ConfigServiceServer::new(grpc_service))
        .serve_with_shutdown(addr, shutdown_signal())
        .await
        .unwrap();

    println!("config-service shutdown complete.");
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
