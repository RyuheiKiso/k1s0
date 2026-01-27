//! auth-service - k1s0 framework authentication and authorization service
//!
//! This service provides:
//! - User authentication (login, token issuance)
//! - Permission checking (CheckPermission)
//! - Role management
//!
//! # 起動方法
//!
//! ```bash
//! auth-service --env dev --port 50051
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

use application::AuthService;
use domain::{Role, RoleRepository, User, UserRepository};
use infrastructure::{
    InMemoryPermissionRepository, InMemoryRoleRepository, InMemoryTokenRepository,
    InMemoryUserRepository,
};
use presentation::grpc::auth_v1::auth_service_server::AuthServiceServer;
use presentation::grpc::GrpcAuthService;

/// auth-service CLI arguments
#[derive(Parser, Debug)]
#[command(name = "auth-service")]
#[command(about = "k1s0 framework authentication and authorization service")]
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

    /// JWT issuer
    #[arg(long, default_value = "k1s0-auth")]
    issuer: String,

    /// JWT secret (for development only, use secrets file in production)
    #[arg(long, default_value = "dev-secret-change-in-production")]
    jwt_secret: String,
}

/// InMemoryリポジトリを使用するAuthService型
type InMemoryAuthService = AuthService<
    InMemoryUserRepository,
    InMemoryRoleRepository,
    InMemoryPermissionRepository,
    InMemoryTokenRepository,
>;

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
        service = "auth-service",
        env = %args.env,
        port = %args.port,
        "Starting auth-service"
    );

    // データベースURL指定時はPostgreSQLを使用
    if let Some(ref db_url) = args.database_url {
        info!(database_url = %db_url, "PostgreSQL mode (not yet implemented in main)");
        // TODO: PostgreSQL リポジトリを使用する実装
        // 現時点ではInMemoryにフォールバック
        warn!("PostgreSQL mode not fully implemented, falling back to in-memory");
    }

    // InMemoryリポジトリの初期化
    let user_repo = Arc::new(InMemoryUserRepository::new());
    let role_repo = Arc::new(InMemoryRoleRepository::new());
    let permission_repo = Arc::new(InMemoryPermissionRepository::new());
    let token_repo = Arc::new(InMemoryTokenRepository::new());

    // 開発環境の場合、サンプルデータを登録
    if args.env == "dev" {
        setup_sample_data(&user_repo, &role_repo, &permission_repo).await?;
    }

    // サービスの初期化
    let service = AuthService::new(
        user_repo,
        role_repo,
        permission_repo,
        token_repo,
        &args.issuer,
        &args.jwt_secret,
    );

    // 開発環境での動作確認
    if args.env == "dev" {
        run_dev_tests(&service).await;
    }

    // gRPC サービスの作成
    let grpc_service = GrpcAuthService::new(Arc::new(service));

    // Health reporter
    let (mut health_reporter, health_service) = health_reporter();
    health_reporter
        .set_serving::<AuthServiceServer<GrpcAuthService<
            InMemoryUserRepository,
            InMemoryRoleRepository,
            InMemoryPermissionRepository,
            InMemoryTokenRepository,
        >>>()
        .await;

    // gRPC サーバーの起動
    let addr: SocketAddr = format!("0.0.0.0:{}", args.port).parse()?;
    info!(address = %addr, "Starting gRPC server");

    Server::builder()
        .add_service(health_service)
        .add_service(AuthServiceServer::new(grpc_service))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    info!("auth-service shutdown complete");
    Ok(())
}

/// サンプルデータの登録（開発環境用）
async fn setup_sample_data(
    user_repo: &Arc<InMemoryUserRepository>,
    role_repo: &Arc<InMemoryRoleRepository>,
    permission_repo: &Arc<InMemoryPermissionRepository>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Setting up sample data for development");

    // ロール
    role_repo.add_role(Role::new(1, "admin", "Administrator"));
    role_repo.add_role(Role::new(2, "user", "Normal user"));
    role_repo.add_role(Role::new(3, "viewer", "Read-only user"));

    // ユーザー（パスワード: password123）
    let admin_user = User::new(
        1,
        "admin",
        "admin@example.com",
        "System Administrator",
        "hash:password123",
    );
    user_repo.save(&admin_user).await?;

    let test_user = User::new(
        2,
        "testuser",
        "test@example.com",
        "Test User",
        "hash:password123",
    );
    user_repo.save(&test_user).await?;

    // ロール割り当て
    role_repo.assign_role(1, 1).await?; // admin -> admin role
    role_repo.assign_role(2, 2).await?; // testuser -> user role

    // パーミッション
    permission_repo.add_permission(1, "user:read", None);
    permission_repo.add_permission(1, "user:write", None);
    permission_repo.add_permission(1, "admin:all", None);
    permission_repo.add_permission(2, "user:read", None);

    info!("Sample data setup complete");
    Ok(())
}

/// 開発環境での動作確認テスト
async fn run_dev_tests(service: &InMemoryAuthService) {
    info!("Running development tests");

    // 認証テスト
    match service.authenticate("admin", "password123").await {
        Ok(token) => {
            info!(
                token_type = %token.token_type,
                expires_in = %token.expires_in,
                "Authentication test: SUCCESS"
            );
        }
        Err(e) => {
            warn!(error = %e, "Authentication test: FAILED");
        }
    }

    // パーミッションチェックテスト
    match service.check_permission(1, "admin:all", None).await {
        Ok(has_perm) => {
            info!(
                user_id = 1,
                permission = "admin:all",
                allowed = %has_perm,
                "Permission check test: SUCCESS"
            );
        }
        Err(e) => {
            warn!(error = %e, "Permission check test: FAILED");
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
