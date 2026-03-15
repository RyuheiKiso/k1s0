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

# 社内ライブラリ
k1s0-telemetry = { path = "../../../library/rust/telemetry" }
k1s0-correlation = { path = "../../../library/rust/correlation", features = ["tower-layer"] }

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
    let protoc_path = protoc_bin_vendored::protoc_bin_path()
        .or_else(|_| std::env::var("PROTOC").map(std::path::PathBuf::from).map_err(Into::into));
    if let Ok(path) = protoc_path {
        std::env::set_var("PROTOC", path);
    } else {
        println!("cargo:warning=protoc not found; relying on system-installed protoc");
    }

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(
            &["{proto_path}"],
            &["api/proto/", "../../../../../../api/proto/"],
        )?;
    Ok(())
}
```

### build.rs 運用ノート

- `protoc` が未インストールの開発環境では `protoc-bin-vendored` を優先し、取得失敗時は `PROTOC` / システム `protoc` にフォールバックする。
- 生成先は `.out_dir("src/proto")` に固定し、CI/CD でも生成物パスを揃える。
- CI では `protobuf-compiler` を明示インストールするか、`PROTOC` 環境変数を設定してビルド再現性を担保する。

---

## 共通 main.rs 起動シーケンス {#共通mainrs}

全サーバーは以下の起動シーケンスに従う。サービス固有の DI は各implementation.md に記載する。

```
1. Config::load("config/config.yaml")
2. init_telemetry(&telemetry_cfg)   # ログ + トレーサー一括初期化（TelemetryConfigBuilder 推奨）
3. persistence::connect(&cfg.database) + sqlx::migrate!
4. KafkaProducer::new(&cfg.kafka)  ※Kafka 使用時
5. DI（サービス固有のユースケース・リポジトリ注入）
6. REST サーバー起動（axum::Router + axum::serve + CorrelationLayer）
7. gRPC サーバー起動（tonic::transport::Server）
8. graceful_shutdown（SIGTERM/SIGINT 待機）
9. k1s0_telemetry::shutdown()（OpenTelemetry フラッシュ）
```

---

## Graceful Shutdown パターン {#graceful-shutdown}

全 Rust サーバーは `k1s0-server-common` の共通シャットダウンシグナルを使用する。

### shutdown_signal() — 共通モジュール

`k1s0_server_common::shutdown::shutdown_signal()` として一元管理されている。
各サーバーの `startup.rs` にローカル定義を置かず、共通クレートからインポートする。

```rust
// k1s0-server-common の shutdown モジュール（全サーバー共通）
// Cargo.toml に features = ["shutdown"] が必要
use k1s0_server_common::shutdown::shutdown_signal;
```

動作仕様:
- Unix: SIGTERM + SIGINT（Ctrl-C）を待機（`tokio::select!`）
- Windows: SIGINT（Ctrl-C）のみ

### REST + gRPC の並行 Graceful Shutdown

```rust
let grpc_shutdown = k1s0_server_common::shutdown::shutdown_signal();
let grpc_future = async move {
    tonic::transport::Server::builder()
        // ...
        .serve_with_shutdown(grpc_addr, async move { let _ = grpc_shutdown.await; })
        .await
};

let rest_future = axum::serve(listener, app)
    .with_graceful_shutdown(async { let _ = k1s0_server_common::shutdown::shutdown_signal().await; });

tokio::select! {
    result = rest_future => { /* error handling */ }
    result = grpc_future => { /* error handling */ }
}

k1s0_telemetry::shutdown();  // テレメトリを最後にフラッシュ
```

### CorrelationLayer の適用

全サーバーの REST アプリケーションに `CorrelationLayer` を追加する（MetricsLayer の外側に配置）:

```rust
let app = handler::router(state)
    .layer(k1s0_telemetry::MetricsLayer::new(metrics.clone()))
    .layer(k1s0_correlation::layer::CorrelationLayer::new());  // 最外層
```

CorrelationLayer は受信リクエストに対して:
1. `X-Correlation-Id` ヘッダーを解析（なければ UUID v4 を自動生成）
2. `X-Trace-Id` ヘッダーを解析（あれば伝播）
3. `CorrelationContext` を `Extensions` に格納（ハンドラーから取得可能）
4. レスポンスヘッダーに `X-Correlation-Id` / `X-Trace-Id` を自動付与

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
