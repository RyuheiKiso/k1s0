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

use clap::Parser;
use tokio::signal;
use tonic::transport::Server;
use tonic_health::server::health_reporter;
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

/// config-service CLI arguments
#[derive(Parser, Debug)]
#[command(name = "config-service")]
#[command(about = "k1s0 framework dynamic configuration service")]
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
    #[arg(short, long, default_value = "50051")]
    port: u16,

    /// REST API port (optional)
    #[arg(long)]
    rest_port: Option<u16>,

    /// Database URL (if not set, uses in-memory storage)
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

    /// Redis URL for caching (optional)
    #[arg(long, env = "REDIS_URL")]
    redis_url: Option<String>,
}

/// InMemoryリポジトリを使用するConfigService型
type InMemoryConfigService = ConfigService<InMemoryRepository, InMemoryCache>;

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
        service = "config-service",
        env = %args.env,
        port = %args.port,
        "Starting config-service"
    );

    // データベースURL指定時はPostgreSQLを使用
    if let Some(ref db_url) = args.database_url {
        info!(database_url = %db_url, "PostgreSQL mode (not yet implemented in main)");
        warn!("PostgreSQL mode not fully implemented, falling back to in-memory");
    }

    // Redis URL指定時はRedisキャッシュを使用
    if let Some(ref redis_url) = args.redis_url {
        info!(redis_url = %redis_url, "Redis cache mode (not yet implemented in main)");
        warn!("Redis cache mode not fully implemented, falling back to in-memory cache");
    }

    // InMemoryリポジトリとキャッシュの初期化
    let repository = Arc::new(InMemoryRepository::new());
    let cache = Arc::new(InMemoryCache::new());

    // 開発環境の場合、サンプルデータを登録
    if args.env == "dev" {
        setup_sample_data(&repository).await?;
    }

    // サービスの初期化
    let service = ConfigService::new(
        Arc::clone(&repository),
        Arc::clone(&cache),
        &args.env,
    );

    // 開発環境での動作確認
    if args.env == "dev" {
        run_dev_tests(&service).await;
    }

    // gRPC サービスの作成
    let grpc_service = GrpcConfigService::new(Arc::new(service));

    // Health reporter
    let (mut health_reporter, health_service) = health_reporter();
    health_reporter
        .set_serving::<ConfigServiceServer<GrpcConfigService<InMemoryRepository, InMemoryCache>>>()
        .await;

    // gRPC サーバーの起動
    let addr: SocketAddr = format!("0.0.0.0:{}", args.port).parse()?;
    info!(address = %addr, "Starting gRPC server");

    Server::builder()
        .add_service(health_service)
        .add_service(ConfigServiceServer::new(grpc_service))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    info!("config-service shutdown complete");
    Ok(())
}

/// サンプルデータの登録（開発環境用）
async fn setup_sample_data(
    repository: &Arc<InMemoryRepository>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting up sample data for development");

    let samples = vec![
        Setting::new(1, "sample-service", "dev", "feature.enabled", "true"),
        Setting::new(2, "sample-service", "dev", "http.timeout_ms", "5000"),
        Setting::new(3, "sample-service", "dev", "db.pool_size", "10"),
        Setting::new(4, "auth-service", "dev", "jwt.ttl_seconds", "3600"),
        Setting::new(5, "auth-service", "dev", "jwt.refresh_ttl_seconds", "604800"),
    ];

    for setting in samples {
        repository.save(&setting).await?;
    }

    info!(count = 5, "Sample data setup complete");
    Ok(())
}

/// 開発環境での動作確認テスト
async fn run_dev_tests(service: &InMemoryConfigService) {
    info!("Running development tests");

    // 設定取得のテスト
    match service.get_setting("sample-service", "feature.enabled", None).await {
        Ok(setting) => {
            info!(
                service_name = "sample-service",
                key = %setting.key,
                value = %setting.value,
                "Get setting test: SUCCESS"
            );
        }
        Err(e) => {
            warn!(error = %e, "Get setting test: FAILED");
        }
    }

    // 設定一覧のテスト
    let query = SettingQuery::new().with_service_name("sample-service");
    match service.list_settings(&query).await {
        Ok(list) => {
            info!(
                service_name = "sample-service",
                count = %list.settings.len(),
                "List settings test: SUCCESS"
            );
        }
        Err(e) => {
            warn!(error = %e, "List settings test: FAILED");
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
