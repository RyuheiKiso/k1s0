//! auth-service - k1s0 framework authentication and authorization service
//!
//! This service provides:
//! - User authentication (login, token issuance)
//! - Permission checking (CheckPermission)
//! - Role management

use std::env;
use std::path::PathBuf;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    let mut env_name = String::from("dev");
    let mut config_path: Option<PathBuf> = None;
    let mut secrets_dir: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--env" => {
                if i + 1 < args.len() {
                    env_name = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --env requires a value");
                    std::process::exit(1);
                }
            }
            "--config" => {
                if i + 1 < args.len() {
                    config_path = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("Error: --config requires a value");
                    std::process::exit(1);
                }
            }
            "--secrets-dir" => {
                if i + 1 < args.len() {
                    secrets_dir = Some(PathBuf::from(&args[i + 1]));
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

    println!("auth-service starting...");
    println!("  Environment: {}", env_name);
    if let Some(ref path) = config_path {
        println!("  Config: {}", path.display());
    }
    if let Some(ref path) = secrets_dir {
        println!("  Secrets: {}", path.display());
    }

    // TODO: Initialize service
    // - Load configuration
    // - Initialize database connection
    // - Initialize observability (tracing, metrics)
    // - Start gRPC server

    println!("auth-service is not yet implemented. Placeholder only.");
}
