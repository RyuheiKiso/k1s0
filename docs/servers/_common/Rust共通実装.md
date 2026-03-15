# system-server Rust 共通実装リファレンス

system tier の全 Rust サーバーで共通する実装パターンを定義する。各サーバーのimplementation.md ではサービス固有部分のみを記載し、共通部分は本ドキュメントを参照する。

---

## 共通 Cargo.toml 依存 {#共通cargo依存}

全サーバーで共通して使用する依存クレート。サービス固有の依存は各implementation.md に記載する。

### workspace.dependencies（一元管理）

`regions/system/Cargo.toml` の `[workspace.dependencies]` で共通依存のバージョンと features を一元定義している。各 crate は `workspace = true` で参照し、バージョンの分散を防ぐ。

```toml
# regions/system/Cargo.toml（抜粋）
[workspace.dependencies]
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower = { version = "0.5", features = ["util"] }
tower-http = { version = "0.6", features = ["trace", "cors"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }
tonic = "0.12"
tonic-build = "0.12"
prost = "0.13"
prost-types = "0.13"
reqwest = { version = "0.12", features = ["json"] }
thiserror = "2"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
async-trait = "0.1"
utoipa = { version = "5", features = ["axum_extras", "chrono", "uuid"] }
utoipa-swagger-ui = { version = "8", features = ["axum"] }
mockall = "0.13"
tokio-test = "0.4"
```

### 各 crate での参照パターン

```toml
[dependencies]
# workspace.dependencies で定義済みのものは workspace = true で参照する
axum = {workspace = true}
tokio = {workspace = true}
serde = {workspace = true}
sqlx = {workspace = true}

# workspace 定義に含まれない追加 features がある場合は features を指定する
sqlx = {workspace = true, features = ["migrate"]}
reqwest = {workspace = true, features = ["rustls-tls"]}

# optional 依存の場合
axum = {workspace = true, optional = true}

# workspace 未定義の依存はバージョンを直接指定する
rdkafka = { version = "0.36", features = ["cmake-build"] }

# 社内ライブラリ（path 依存はワークスペース管理外）
k1s0-telemetry = { path = "../../../library/rust/telemetry", features = ["full"] }
k1s0-correlation = { path = "../../../library/rust/correlation", features = ["tower-layer"] }

[dev-dependencies]
mockall = {workspace = true}
tokio-test = {workspace = true}

[build-dependencies]
tonic-build = {workspace = true}
```

### axum 0.7 / 0.8 の混在

workspace.dependencies には axum **0.7** を定義している。以下の 3 crate は axum **0.8** を使用しており、個別にバージョンを指定する（`workspace = true` を使わない）:

- `library/rust/auth` — axum 0.8
- `library/rust/idempotency` — axum 0.8
- `server/rust/graphql-gateway` — axum 0.8

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
