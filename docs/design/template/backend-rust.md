# backend-rust テンプレート

← [テンプレート設計書](./)

---

## ディレクトリ構造

```
feature/backend/rust/{service_name}/
├── .k1s0/
│   └── manifest.json.tera
├── Cargo.toml.tera
├── README.md.tera
├── config/
│   ├── default.yaml
│   ├── dev.yaml
│   ├── stg.yaml
│   └── prod.yaml
├── deploy/
│   ├── base/
│   │   ├── configmap.yaml.tera
│   │   ├── deployment.yaml.tera
│   │   ├── service.yaml.tera
│   │   └── kustomization.yaml.tera
│   └── overlays/
│       ├── dev/
│       │   └── kustomization.yaml.tera
│       ├── stg/
│       │   └── kustomization.yaml.tera
│       └── prod/
│           └── kustomization.yaml.tera
├── proto/
│   └── service.proto.tera
├── openapi/
│   └── openapi.yaml.tera
├── migrations/
│   ├── 0001_initial.up.sql.tera
│   └── 0001_initial.down.sql.tera
└── src/
    ├── main.rs.tera
    ├── application/
    │   ├── mod.rs.tera
    │   ├── services/
    │   │   └── mod.rs.tera
    │   └── usecases/
    │       └── mod.rs.tera
    ├── domain/
    │   ├── mod.rs.tera
    │   ├── entities/
    │   │   └── mod.rs.tera
    │   └── errors/
    │       └── mod.rs.tera
    ├── infrastructure/
    │   └── mod.rs
    └── presentation/
        └── mod.rs
```

## Cargo.toml.tera

```toml
[package]
name = "{{ feature_name }}"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[dependencies]
# Framework crates
k1s0-error = { path = "../../../../framework/backend/rust/crates/k1s0-error" }
k1s0-config = { path = "../../../../framework/backend/rust/crates/k1s0-config" }
k1s0-observability = { path = "../../../../framework/backend/rust/crates/k1s0-observability" }
k1s0-validation = { path = "../../../../framework/backend/rust/crates/k1s0-validation" }
{% if with_grpc %}
k1s0-grpc-server = { path = "../../../../framework/backend/rust/crates/k1s0-grpc-server" }
{% endif %}
k1s0-resilience = { path = "../../../../framework/backend/rust/crates/k1s0-resilience" }

# Runtime
tokio = { version = "1", features = ["full"] }
{% if with_grpc %}
tonic = "0.12"
prost = "0.13"
{% endif %}
{% if with_rest %}
axum = "0.7"
{% endif %}

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json"] }
opentelemetry = "0.24"
{% if with_db %}

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"] }
{% endif %}

[dev-dependencies]
tokio-test = "0.4"
```

## main.rs.tera

```rust
//! {{ feature_name }} サービス
//!
//! k1s0 framework を使用した {{ feature_name }} のエントリポイント。

mod application;
mod domain;
mod infrastructure;
mod presentation;

use std::sync::Arc;
use clap::Parser;
use tracing::info;
{% if with_rest %}
use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;
{% endif %}
{% if with_grpc %}
use tonic::transport::Server as TonicServer;
{% endif %}

/// CLI 引数
#[derive(Parser, Debug)]
#[command(name = "{{ feature_name }}")]
struct Args {
    #[arg(long, default_value = "dev")]
    env: String,

    #[arg(long, default_value = "./config")]
    config: String,
{% if with_grpc %}
    #[arg(long, default_value = "50051")]
    grpc_port: u16,
{% endif %}
{% if with_rest %}
    #[arg(long, default_value = "8080")]
    http_port: u16,
{% endif %}
    #[arg(long, default_value = "9090")]
    health_port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Observability 初期化
    let _guard = k1s0_observability::init_with_config(
        k1s0_observability::ObservabilityConfig::builder()
            .service_name("{{ feature_name }}")
            .service_version(env!("CARGO_PKG_VERSION"))
            .environment(&args.env)
            .build(),
    )?;

    info!(service = "{{ feature_name }}", "Starting service");

    // 設定読み込み
    let config = k1s0_config::ConfigBuilder::new()
        .add_source(k1s0_config::File::with_name(&format!("{}/{}.yaml", args.config, args.env)))
        .build()?;

    // ヘルスチェックサーバー起動
    let health_registry = Arc::new(k1s0_health::HealthRegistry::new());
    let health_addr = format!("0.0.0.0:{}", args.health_port).parse()?;
    let health_router = k1s0_health::health_router(health_registry.clone());
    let health_server = tokio::spawn(async move {
        info!(addr = %health_addr, "Starting health check server");
        axum::serve(
            tokio::net::TcpListener::bind(health_addr).await.unwrap(),
            health_router,
        ).await.unwrap();
    });

{% if with_rest %}
    // HTTP サーバー起動
    let http_addr = format!("0.0.0.0:{}", args.http_port).parse()?;
    let http_router = Router::new()
        .route("/", get(|| async { "{{ feature_name }} service is running" }))
        .layer(TraceLayer::new_for_http());
    let http_server = tokio::spawn(async move {
        info!(addr = %http_addr, "Starting HTTP server");
        axum::serve(
            tokio::net::TcpListener::bind(http_addr).await.unwrap(),
            http_router,
        ).await.unwrap();
    });
{% endif %}

{% if with_grpc %}
    // gRPC サーバー起動
    let grpc_addr = format!("0.0.0.0:{}", args.grpc_port).parse()?;
    let grpc_server = tokio::spawn(async move {
        info!(addr = %grpc_addr, "Starting gRPC server");
        TonicServer::builder()
            // .add_service(your_service)
            .serve(grpc_addr)
            .await
            .unwrap();
    });
{% endif %}

    // Graceful shutdown
    info!("Service started, waiting for shutdown signal...");
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    health_server.abort();
{% if with_rest %}
    http_server.abort();
{% endif %}
{% if with_grpc %}
    grpc_server.abort();
{% endif %}

    k1s0_observability::shutdown();
    info!("Service shutdown complete");
    Ok(())
}
```
