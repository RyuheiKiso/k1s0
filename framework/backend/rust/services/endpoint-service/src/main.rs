//! endpoint-service - k1s0 framework endpoint discovery service
//!
//! This service provides:
//! - Endpoint information retrieval (Get/List)
//! - Service name to endpoint resolution

use std::env;
use std::path::PathBuf;
use std::sync::Arc;

mod application;
mod domain;
mod infrastructure;
mod presentation;

use application::EndpointService;
use domain::{Endpoint, EndpointRepository};
use infrastructure::InMemoryRepository;

/// サービス設定
struct ServiceConfig {
    env_name: String,
    config_path: Option<PathBuf>,
    secrets_dir: Option<PathBuf>,
    namespace: String,
    cluster_domain: String,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            env_name: "dev".to_string(),
            config_path: None,
            secrets_dir: None,
            namespace: "default".to_string(),
            cluster_domain: "cluster.local".to_string(),
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

    // TODO: gRPC サーバーの起動
    println!("\nendpoint-service initialized successfully.");
    println!("gRPC server implementation pending...");

    Ok(())
}
