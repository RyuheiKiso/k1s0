# system-server Rust 共通実装リファレンス

system tier の全 Rust サーバーで共通する実装パターンを定義する。各サーバーのimplementation.md ではサービス固有部分のみを記載し、共通部分は本ドキュメントを参照する。

---

## 共通 Cargo.toml 依存 {#共通cargo依存}

全サーバーで共通して使用する依存クレート。サービス固有の依存は各implementation.md に記載する。

```toml
[dependencies]
# Web フレームワーク
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors"] }
hyper = { version = "1", features = ["full"] }

# gRPC
tonic = "0.12"
prost = "0.13"
prost-types = "0.13"

# シリアライゼーション
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# DB
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }

# Kafka
rdkafka = { version = "0.36", features = ["cmake-build"] }

# OpenTelemetry
opentelemetry = "0.27"
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.27", features = ["tonic"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-opentelemetry = "0.28"

# ユーティリティ
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
validator = { version = "0.18", features = ["derive"] }

# メトリクス
prometheus = "0.13"

[dev-dependencies]
mockall = "0.13"
tokio-test = "0.4"
testcontainers = "0.23"

[build-dependencies]
tonic-build = "0.12"
```

---

## 共通 build.rs パターン {#共通buildrs}

tonic-build による proto コンパイル。`{proto_path}` と `{include_paths}` はサービスごとに異なる。

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["{proto_path}"],
            &["api/proto/", "../../../../../../api/proto/"],
        )?;
    Ok(())
}
```

---

## 共通 main.rs 起動シーケンス {#共通mainrs}

全サーバーは以下の起動シーケンスに従う。サービス固有の DI は各implementation.md に記載する。

```
1. Config::load("config/config.yaml")
2. init_telemetry(&telemetry_cfg)   # ログ + トレーサー一括初期化
3. persistence::connect(&cfg.database) + sqlx::migrate!
4. KafkaProducer::new(&cfg.kafka)  ※Kafka 使用時
5. DI（サービス固有のユースケース・リポジトリ注入）
6. REST サーバー起動（axum::Router + axum::serve）
7. gRPC サーバー起動（tonic::transport::Server）
8. graceful_shutdown（SIGTERM/SIGINT 待機）
```

---

## 共通 config.yaml セクション {#共通configyaml}

全サーバーの config.yaml に含まれる共通セクション。サービス固有セクションは各設計書を参照。

```yaml
app:
  name: "{service-name}"
  version: "0.1.0"
  environment: "production"    # dev | staging | production

server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 50051             # サービスにより異なる

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_system"
  user: "app"
  password: ""                 # Vault 経由で注入
  ssl_mode: "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  security_protocol: "PLAINTEXT"

observability:
  otlp_endpoint: "http://otel-collector.observability:4317"
  log_level: "info"
  log_format: "json"
  metrics_enabled: true
```

---

## 関連ドキュメント

- [system-server.md](../auth/server.md) -- auth-server 設計（参考実装）
- [system-server-deploy.md](deploy.md) -- Dockerfile・Helm・テスト方針
- [テンプレート仕様-サーバー-Rust](../../templates/server/サーバー-Rust.md) -- Rust テンプレート詳細
