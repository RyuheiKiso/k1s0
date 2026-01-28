# k1s0-observability

## 目的

観測性（ログ/トレース/メトリクス）の統一初期化ライブラリ。必須フィールドを強制し、OpenTelemetry と統合する。

## 設計方針

- **必須フィールドの強制**: `service.name`, `env` 等を初期化時に必須化
- **JSON ログの統一**: 構造化ログの必須フィールドを固定
- **OTel 統合**: OpenTelemetry によるトレース/メトリクス

## 必須フィールド（ログ）

| フィールド | 説明 |
|-----------|------|
| `timestamp` | ISO 8601 形式のタイムスタンプ |
| `level` | ログレベル（DEBUG/INFO/WARN/ERROR） |
| `message` | ログメッセージ |
| `service.name` | サービス名 |
| `service.env` | 環境名（dev/stg/prod） |
| `trace.id` | トレース ID（リクエスト相関用） |
| `request.id` | リクエスト ID |

## 主要な型

### ObservabilityConfig

```rust
pub struct ObservabilityConfig {
    service_name: String,
    env: String,
    version: Option<String>,
}

impl ObservabilityConfig {
    pub fn builder() -> ObservabilityBuilder;
    pub fn service_name(&self) -> &str;
    pub fn env(&self) -> &str;
    pub fn new_request_context(&self) -> RequestContext;
}
```

### RequestContext

```rust
pub struct RequestContext {
    trace_id: String,
    request_id: String,
    tenant_id: Option<String>,
}

impl RequestContext {
    pub fn new() -> Self;
    pub fn trace_id(&self) -> &str;
    pub fn request_id(&self) -> &str;
}
```

### LogEntry

```rust
pub struct LogEntry {
    level: LogLevel,
    message: String,
    timestamp: String,
    service_name: Option<String>,
    env: Option<String>,
    trace_id: Option<String>,
    request_id: Option<String>,
    fields: HashMap<String, serde_json::Value>,
}

impl LogEntry {
    pub fn info(message: impl Into<String>) -> Self;
    pub fn warn(message: impl Into<String>) -> Self;
    pub fn error(message: impl Into<String>) -> Self;
    pub fn with_context(self, ctx: &RequestContext) -> Self;
    pub fn with_service(self, config: &ObservabilityConfig) -> Self;
    pub fn to_json(&self) -> Result<String>;
}
```

## 使用例

```rust
use k1s0_observability::{ObservabilityConfig, LogEntry};

let config = ObservabilityConfig::builder()
    .service_name("user-service")
    .env("dev")
    .build()
    .expect("必須フィールドが不足");

let ctx = config.new_request_context();

let entry = LogEntry::info("ユーザーを作成しました")
    .with_context(&ctx)
    .with_service(&config);

println!("{}", entry.to_json().unwrap());
// {"timestamp":"2026-01-27T10:00:00Z","level":"INFO","message":"ユーザーを作成しました","service.name":"user-service","service.env":"dev","trace.id":"...","request.id":"..."}
```
