# テレメトリマクロ / 計装ユーティリティ

> **対応言語**: Rust 専用
> proc-macro による属性マクロであり、Rust コンパイラの機能に依存するため他言語への移植対象外です。

## 概要

`k1s0-telemetry-macros` と `k1s0-telemetry` の計装モジュール群。proc-macro による自動トレーシング、SQLx プール計装、Kafka トレースコンテキスト伝播、エラー分類を提供する。

## Feature Flags

```toml
# telemetry-macros (proc-macro crate、Feature Flag なし)
k1s0-telemetry-macros = { path = "..." }

# telemetry の計装機能
k1s0-telemetry = { path = "...", features = ["sqlx-instrument", "kafka-instrument"] }
```

| Feature | クレート | 説明 |
|---------|---------|------|
| (なし) | telemetry-macros | `#[k1s0_trace]` proc-macro |
| `sqlx-instrument` | telemetry | TracedPool (SQLx メトリクス自動記録) |
| `kafka-instrument` | telemetry | KafkaTracing (OpenTelemetry コンテキスト伝播) |

## API

### #[k1s0_trace] マクロ

関数を `tracing::instrument` でラップし、自動的にスパンを作成する proc-macro アトリビュート。

#### オプション

| オプション | 説明 | 例 |
|-----------|------|-----|
| `skip(arg1, arg2)` | 指定した引数をスパンから除外 | `#[k1s0_trace(skip(password))]` |
| `name = "..."` | カスタムスパン名を設定 | `#[k1s0_trace(name = "auth.login")]` |

#### 展開結果

```rust
// 入力
#[k1s0_trace(skip(password), name = "auth.login")]
async fn login(user: &str, password: &str) -> Result<Token, AuthError> { ... }

// 展開後
#[tracing::instrument(name = "auth.login", skip(password), level = "info")]
async fn login(user: &str, password: &str) -> Result<Token, AuthError> { ... }
```

### TracedPool

`sqlx::PgPool` をラップし、メトリクスを自動記録する。`sqlx-instrument` Feature で有効化。

```rust
pub struct TracedPool {
    inner: sqlx::PgPool,
    metrics: Arc<Metrics>,
}

impl TracedPool {
    pub fn new(pool: sqlx::PgPool, metrics: Arc<Metrics>) -> Self;
    pub fn inner(&self) -> &sqlx::PgPool;
    pub fn metrics(&self) -> &Metrics;
}
```

### KafkaTracing

Kafka メッセージヘッダーに OpenTelemetry トレースコンテキストを注入/抽出する。`kafka-instrument` Feature で有効化。

```rust
pub struct KafkaTracing;

impl KafkaTracing {
    /// 現在のスパンのコンテキストを Kafka ヘッダーに注入
    pub fn inject_context(headers: &mut HashMap<String, Vec<u8>>);

    /// Kafka ヘッダーからコンテキストを抽出し、現在のスパンの親に設定
    pub fn extract_context(headers: &HashMap<String, Vec<u8>>);
}
```

### ErrorSeverity / classify_error

エラーメッセージの内容に基づいて重大度を分類する。リトライ判定やアラート制御に使用。

```rust
pub enum ErrorSeverity {
    Transient,  // timeout, connection, unavailable → リトライ可能
    Permanent,  // not found, invalid, unauthorized → リトライ不要
    Unknown,    // 分類不能
}

pub fn classify_error(err: &dyn std::error::Error) -> ErrorSeverity;
```

## 使用例

```rust
use k1s0_telemetry_macros::k1s0_trace;
use k1s0_telemetry::instrument::sqlx::TracedPool;
use k1s0_telemetry::instrument::kafka::KafkaTracing;
use k1s0_telemetry::error_classifier::{classify_error, ErrorSeverity};

// 自動トレーシング
#[k1s0_trace]
async fn get_user(pool: &TracedPool, id: UserId) -> Result<User, AppError> {
    let row = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(pool.inner())
        .await?;
    Ok(row)
}

// Kafka コンテキスト伝播 (Producer側)
let mut headers = HashMap::new();
KafkaTracing::inject_context(&mut headers);
producer.send(record.with_headers(headers)).await?;

// Kafka コンテキスト伝播 (Consumer側)
KafkaTracing::extract_context(&message.headers);
process_message(message).await?;

// エラー分類
match classify_error(&err) {
    ErrorSeverity::Transient => retry(operation).await,
    ErrorSeverity::Permanent => return Err(err),
    ErrorSeverity::Unknown => log_and_alert(err),
}
```

## 設計判断

| 判断 | 理由 |
|------|------|
| proc-macro を別クレートに分離 | Rust の proc-macro クレート制約（通常コードと混在不可）に準拠 |
| `tracing::instrument` へのラッパー | k1s0 固有のデフォルト (level = "info") を統一適用 |
| TracedPool のラッパーパターン | 既存の sqlx API をそのまま使いつつメトリクスを透過的に追加 |
| KafkaTracing を static メソッドに | ステートレスな操作で、インスタンス不要 |
| エラー分類をメッセージベースに | 型に依存しない汎用的な分類を実現（トレードオフ: 精度は限定的） |
