//! auth-service - k1s0 framework authentication and authorization service
//!
//! This service provides:
//! - User authentication (login, token issuance)
//! - Permission checking (CheckPermission)
//! - Role management

use std::env;
use std::path::PathBuf;
use std::sync::Arc;

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

/// サービス設定
struct ServiceConfig {
    env_name: String,
    config_path: Option<PathBuf>,
    secrets_dir: Option<PathBuf>,
    jwt_secret: String,
    issuer: String,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            env_name: "dev".to_string(),
            config_path: None,
            secrets_dir: None,
            jwt_secret: "dev-secret-change-in-production".to_string(),
            issuer: "k1s0-auth".to_string(),
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
            "--help" | "-h" => {
                println!("auth-service - k1s0 framework authentication service");
                println!();
                println!("USAGE:");
                println!("    auth-service [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("    --env <ENV>           Environment name (default: dev)");
                println!("    --config <PATH>       Path to config file");
                println!("    --secrets-dir <PATH>  Path to secrets directory");
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

    println!("auth-service starting...");
    println!("  Environment: {}", config.env_name);
    println!("  Issuer: {}", config.issuer);
    if let Some(ref path) = config.config_path {
        println!("  Config: {}", path.display());
    }
    if let Some(ref path) = config.secrets_dir {
        println!("  Secrets: {}", path.display());
    }

    // リポジトリの初期化
    let user_repo = Arc::new(InMemoryUserRepository::new());
    let role_repo = Arc::new(InMemoryRoleRepository::new());
    let permission_repo = Arc::new(InMemoryPermissionRepository::new());
    let token_repo = Arc::new(InMemoryTokenRepository::new());

    // サンプルデータの登録
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

    // サービスの初期化
    let service = AuthService::new(
        user_repo,
        role_repo,
        permission_repo,
        token_repo,
        &config.issuer,
        &config.jwt_secret,
    );

    // 動作確認
    println!("\nAuthentication test:");
    match service.authenticate("admin", "password123").await {
        Ok(token) => {
            println!("  Login successful!");
            println!("  Token type: {}", token.token_type);
            println!("  Expires in: {} seconds", token.expires_in);
        }
        Err(e) => {
            println!("  Login failed: {}", e);
        }
    }

    println!("\nPermission check test:");
    let has_admin = service.check_permission(1, "admin:all", None).await?;
    println!("  admin has 'admin:all': {}", has_admin);

    let has_admin = service.check_permission(2, "admin:all", None).await?;
    println!("  testuser has 'admin:all': {}", has_admin);

    // TODO: gRPC サーバーの起動
    println!("\nauth-service initialized successfully.");
    println!("gRPC server implementation pending...");

    Ok(())
}
