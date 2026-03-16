# server-common 拡張機能

## 概要

server-common の Feature Flag で有効化できる拡張モジュール群。Database接続、認証、Kafka、設定ファイル読み込みを K1s0App ビルダーに統合し、サーバー初期化のボイラープレートを排除する。

## Feature Flags

```toml
[dependencies]
k1s0-server-common = { path = "...", features = ["database", "auth", "kafka-setup", "config-loader"] }
```

| Feature | 依存 | 説明 |
|---------|------|------|
| `database` | sqlx 等 | DB接続プール設定・マイグレーション自動実行 |
| `auth` | JWKS検証 | JWT認証設定 (JWKS URL, issuer, audience) |
| `kafka-setup` | rdkafka 等 | Kafkaブローカー接続設定 |
| `config-loader` | serde_yaml | YAMLファイルからの設定読み込み |

## API

### K1s0App (ビルダー)

サーバー初期化を一括管理する上位ビルダー。Config → Telemetry → Metrics → HealthCheck → K1s0Stack を順序保証付きで構築する。

```rust
pub struct K1s0App {
    // 基本フィールド
    telemetry_config: TelemetryConfig,
    profile: Option<Profile>,
    health_checks: Vec<Box<dyn HealthCheck>>,
    skip_correlation: bool,
    skip_request_id: bool,
    // Feature gated
    #[cfg(feature = "database")]    database_setup: Option<DatabaseSetup>,
    #[cfg(feature = "auth")]        auth_config: Option<AuthConfig>,
    #[cfg(feature = "kafka-setup")] kafka_setup: Option<KafkaSetup>,
}
```

#### 基本メソッド

| メソッド | 説明 |
|---------|------|
| `new(telemetry_config)` | ビルダー作成 |
| `profile(profile)` | Profile を明示指定（省略時は environment から自動判定） |
| `add_health_check(check)` | HealthCheck を追加 |
| `without_correlation()` | Correlation ID ミドルウェアを無効化 |
| `without_request_id()` | Request ID ミドルウェアを無効化 |
| `build()` | Telemetry初期化 + K1s0AppReady を返却 |

#### Feature 別メソッド

| Feature | メソッド | 説明 |
|---------|---------|------|
| `database` | `with_database(config)` | マイグレーション自動実行付きDB設定 |
| `database` | `with_database_no_migrate(config)` | マイグレーションなしDB設定 |
| `auth` | `with_auth(auth_config)` | JWT認証設定 |
| `kafka-setup` | `with_kafka(config)` | Kafka接続設定 |
| `config-loader` | `load_config::<T>(path)` | YAML設定ファイル読み込み（static メソッド） |

### K1s0AppReady

ビルド完了後の不変状態。`wrap()` で Router にミドルウェアスタックを適用する。

```rust
impl K1s0AppReady {
    pub fn service_name(&self) -> &str;
    pub fn profile(&self) -> &Profile;
    pub fn metrics(&self) -> Arc<Metrics>;
    pub fn health_checker(&self) -> Arc<CompositeHealthChecker>;
    pub fn wrap(&self, router: Router) -> Router;  // /healthz, /metrics 自動付与

    #[cfg(feature = "database")]    pub fn database_setup(&self) -> Option<&DatabaseSetup>;
    #[cfg(feature = "auth")]        pub fn auth_config(&self) -> Option<&AuthConfig>;
    #[cfg(feature = "kafka-setup")] pub fn kafka_setup(&self) -> Option<&KafkaSetup>;
}
```

### DatabaseConfig / DatabaseSetup

```rust
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,     // default: 10
    pub min_connections: u32,     // default: 1
    pub connect_timeout_secs: u64, // default: 30
}

pub struct DatabaseSetup { /* config + run_migrations flag */ }
impl DatabaseSetup {
    pub fn new(config: DatabaseConfig) -> Self;
    pub fn without_migrations(self) -> Self;
    pub fn config(&self) -> &DatabaseConfig;
    pub fn should_run_migrations(&self) -> bool;
}
```

### AuthConfig

```rust
pub struct AuthConfig {
    pub jwks_url: String,
    pub issuer: String,
    pub audience: String,
    pub cache_ttl_secs: u64, // default: 600
}
```

### InfraGuard（インフラ設定ガード）

stable サービスが必要なインフラ設定（Database, Kafka, Redis, Storage）を起動時に検証する機構。
設定不足の場合は起動を拒否する（fail-safe 設計）。

#### API

| 関数 | 説明 |
|---|---|
| `allow_in_memory_infra(environment)` | dev/test 環境かつ `ALLOW_IN_MEMORY_INFRA=true` の場合のみバイパスを許可 |
| `require_infra(name, kind, environment, value)` | インフラ設定の存在を検証。None + バイパス無効 → エラー |
| `InfraKind` | Database, Kafka, Redis, Storage の列挙型 |

#### 動作フロー

1. `value` が `Some` → そのまま返す
2. `value` が `None` + バイパス有効（dev/test + 環境変数） → `warn` ログ + `Ok(None)`
3. `value` が `None` + バイパス無効（production 等） → `bail!`（起動失敗）

#### cfg ゲート

- `#[cfg(any(debug_assertions, feature = "dev-infra-bypass"))]` — dev ビルドではバイパス可能
- `#[cfg(not(...))]` — release ビルドでは常にバイパス不可

#### 対象サーバー

workflow, auth, file, tenant, service-catalog, event-store, search, notification, scheduler, ratelimit の 10 サーバーに適用済み。

### KafkaConfig / KafkaSetup

```rust
pub struct KafkaConfig {
    pub brokers: Vec<String>,     // default: ["localhost:9092"]
    pub group_id: Option<String>,
    pub client_id: Option<String>,
}

pub struct KafkaSetup { /* config wrapper */ }
impl KafkaSetup {
    pub fn new(config: KafkaConfig) -> Self;
    pub fn config(&self) -> &KafkaConfig;
}
```

### ConfigError / load_config

```rust
pub enum ConfigError {
    NotFound(String),
    ParseError(String),
    Io(std::io::Error),
}

pub fn load_config<T: DeserializeOwned>(path: &str) -> Result<T, ConfigError>;
```

## 使用例

```rust
use k1s0_server_common::middleware::app::{K1s0App, AuthConfig};
use k1s0_server_common::middleware::database::DatabaseConfig;
use k1s0_server_common::middleware::kafka_setup::KafkaConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // YAML から設定読み込み
    let config: AppConfig = K1s0App::load_config("config/config.yaml")?;

    let app = K1s0App::new(config.telemetry)
        .with_database(config.database)
        .with_auth(config.auth)
        .with_kafka(config.kafka)
        .add_health_check(Box::new(DbHealthCheck))
        .build()
        .await?;

    let router = app.wrap(api_routes());
    // router には /healthz, /metrics が自動付与済み

    axum::serve(listener, router).await?;
    Ok(())
}
```

## 設計判断

| 判断 | 理由 |
|------|------|
| Feature Flag による条件コンパイル | 不要な依存を排除し、コンパイル時間を最小化 |
| K1s0App → K1s0AppReady の2段階 | Telemetry初期化の非同期処理を分離し、型レベルで状態を保証 |
| ビルダーパターン | 初期化順序ミスや設定漏れを構造的に排除 |
| load_config を static メソッドに | K1s0App 構築前に設定を読む必要があるため |
| DatabaseSetup に without_migrations | テスト環境やマイグレーション別管理のケースに対応 |
