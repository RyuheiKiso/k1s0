use axum::{routing::get, Router};
use std::net::SocketAddr;

mod application;
mod domain;
mod infrastructure;
mod presentation;

#[derive(Debug, serde::Deserialize)]
struct Config {
    server: ServerConfig,
}

#[derive(Debug, serde::Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

fn load_config() -> Config {
    let config_path = "config/default.yaml";
    let content = std::fs::read_to_string(config_path).expect("Failed to read config");
    serde_yaml::from_str(&content).expect("Failed to parse config")
}

#[tokio::main]
async fn main() {
    let config = load_config();

    let app = Router::new()
        .route("/health", get(presentation::rest::health_check));

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .expect("Invalid address");

    println!("Listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
