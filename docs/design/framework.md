# Framework 共通ライブラリ設計書

## 概要

k1s0 Framework は、マイクロサービス開発のための共通ライブラリ群を提供します。各 crate は独立して使用可能で、Clean Architecture の原則に従って設計されています。

## Crate 一覧

```
framework/backend/rust/crates/
├── k1s0-error/         # エラー表現の統一
├── k1s0-config/        # 設定読み込み
├── k1s0-validation/    # 入力バリデーション
├── k1s0-observability/ # ログ/トレース/メトリクス
├── k1s0-grpc-server/   # gRPC サーバ共通基盤
├── k1s0-grpc-client/   # gRPC クライアント共通（未実装）
├── k1s0-resilience/    # レジリエンスパターン
├── k1s0-health/        # ヘルスチェック（未実装）
├── k1s0-db/            # DB 接続・トランザクション（未実装）
└── k1s0-auth/          # 認証・認可（未実装）
```

---

## k1s0-error

### 目的

Clean Architecture に基づいたエラー表現の統一ライブラリ。transport 非依存で層別のエラー設計を提供する。

### 設計方針

- **domain 層**: transport 非依存のエラー型（HTTP/gRPC を意識しない）
- **application 層**: `error_code` を付与し、運用で識別可能にする
- **presentation 層**: REST（problem+json）/ gRPC（status + metadata）へ変換

### エラー分類

| 分類 | 説明 | HTTP | gRPC |
|------|------|------|------|
| InvalidInput | 入力不備 | 400 | INVALID_ARGUMENT |
| NotFound | リソースが見つからない | 404 | NOT_FOUND |
| Conflict | 競合（重複等） | 409 | ALREADY_EXISTS |
| Unauthorized | 認証エラー | 401 | UNAUTHENTICATED |
| Forbidden | 認可エラー | 403 | PERMISSION_DENIED |
| DependencyFailure | 依存障害 | 502 | UNAVAILABLE |
| Transient | 一時障害 | 503 | UNAVAILABLE |
| Internal | 内部エラー | 500 | INTERNAL |

### 主要な型

#### DomainError

```rust
pub struct DomainError {
    kind: ErrorKind,
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl DomainError {
    pub fn not_found(resource: &str, id: &str) -> Self;
    pub fn conflict(message: impl Into<String>) -> Self;
    pub fn invalid_input(message: impl Into<String>) -> Self;
    pub fn internal(message: impl Into<String>) -> Self;
    pub fn kind(&self) -> ErrorKind;
}
```

#### AppError

```rust
pub struct AppError {
    domain_error: DomainError,
    error_code: ErrorCode,
    trace_id: Option<String>,
    request_id: Option<String>,
}

impl AppError {
    pub fn from_domain(err: DomainError, code: ErrorCode) -> Self;
    pub fn with_trace_id(self, trace_id: impl Into<String>) -> Self;
    pub fn with_request_id(self, request_id: impl Into<String>) -> Self;
    pub fn to_http_error(&self) -> HttpError;
    pub fn to_grpc_error(&self) -> GrpcError;
}
```

#### ErrorCode

```rust
pub struct ErrorCode(String);

impl ErrorCode {
    pub fn new(code: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
}

// 例: ErrorCode::new("USER_NOT_FOUND")
```

### 使用例

```rust
use k1s0_error::{DomainError, AppError, ErrorCode, ErrorKind};

// domain 層: transport 非依存
let domain_err = DomainError::not_found("User", "user-123");

// application 層: error_code 付与
let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"))
    .with_trace_id("trace-abc123")
    .with_request_id("req-xyz789");

// presentation 層: REST/gRPC 変換
let http_err = app_err.to_http_error();  // -> 404 + problem+json
let grpc_err = app_err.to_grpc_error();  // -> NOT_FOUND + metadata
```

---

## k1s0-config

### 目的

環境変数を使用せず、YAML ファイルと secrets ファイルから設定を読み込むライブラリ。

### 設計方針

- 環境変数は使用しない（CLI 引数で参照先を指定）
- 機密情報は YAML に直接書かず、`*_file` キーでファイルパスを参照
- `--secrets-dir` で secrets ファイルの配置先を指定

### 起動引数

| 引数 | 短縮 | 説明 | デフォルト |
|------|-----|------|-----------|
| `--env` | `-e` | 環境名（必須: dev, stg, prod） | - |
| `--config` | `-c` | 設定ファイルのパス | `{config_dir}/{env}.yaml` |
| `--config-dir` | - | 設定ファイルのディレクトリ | `/etc/k1s0/config` |
| `--secrets-dir` | `-s` | secrets ディレクトリ | `/var/run/secrets/k1s0` |

### 優先順位

1. CLI 引数（参照先指定に限定）
2. YAML（`config/{env}.yaml`。非機密の静的設定）
3. DB（`fw_m_setting`。feature 固有の動的設定）※ 未対応

### 主要な型

#### ConfigOptions

```rust
pub struct ConfigOptions {
    pub env: String,
    pub config_path: Option<PathBuf>,
    pub config_dir: Option<PathBuf>,
    pub secrets_dir: Option<PathBuf>,
}

impl ConfigOptions {
    pub fn new(env: impl Into<String>) -> Self;
    pub fn with_config_path(self, path: impl Into<PathBuf>) -> Self;
    pub fn with_secrets_dir(self, dir: impl Into<PathBuf>) -> Self;
}
```

#### ConfigLoader

```rust
pub struct ConfigLoader {
    options: ConfigOptions,
}

impl ConfigLoader {
    pub fn new(options: ConfigOptions) -> Result<Self>;
    pub fn load<T: DeserializeOwned>(&self) -> Result<T>;
    pub fn resolve_secret_file(&self, path: &str) -> Result<String>;
}
```

#### ServiceInit

```rust
pub struct ServiceInit {
    env: String,
    config_dir: PathBuf,
    secrets_dir: PathBuf,
}

impl ServiceInit {
    pub fn from_args(args: &ServiceArgs) -> Result<Self>;
    pub fn load_config<T: DeserializeOwned>(&self) -> Result<T>;
    pub fn is_production(&self) -> bool;
    pub fn env(&self) -> &str;
}
```

### 使用例

```rust
use k1s0_config::{ConfigLoader, ConfigOptions};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AppConfig {
    db: DbConfig,
}

#[derive(Debug, Deserialize)]
struct DbConfig {
    host: String,
    port: u16,
    password_file: String,
}

let options = ConfigOptions::new("dev")
    .with_config_path("config/dev.yaml")
    .with_secrets_dir("/var/run/secrets/k1s0");

let loader = ConfigLoader::new(options)?;
let config: AppConfig = loader.load()?;

// *_file キーの値をファイルから読み込む
let password = loader.resolve_secret_file(&config.db.password_file)?;
```

---

## k1s0-validation

### 目的

API 境界での入力バリデーションを統一するライブラリ。REST（problem+json）と gRPC（INVALID_ARGUMENT）の両方に対応。

### 主要な型

#### FieldError

```rust
pub struct FieldError {
    field: String,
    kind: FieldErrorKind,
    message: String,
}

pub enum FieldErrorKind {
    Required,
    InvalidFormat,
    MinLength(usize),
    MaxLength(usize),
    MinValue(i64),
    MaxValue(i64),
    Pattern(String),
    Custom(String),
}

impl FieldError {
    pub fn required(field: impl Into<String>) -> Self;
    pub fn invalid_format(field: impl Into<String>, message: impl Into<String>) -> Self;
    pub fn min_length(field: impl Into<String>, min: usize) -> Self;
    pub fn max_length(field: impl Into<String>, max: usize) -> Self;
}
```

#### ValidationErrors

```rust
pub struct ValidationErrors {
    errors: HashMap<String, Vec<FieldError>>,
}

impl ValidationErrors {
    pub fn new() -> Self;
    pub fn add_field_error(&mut self, error: FieldError);
    pub fn is_empty(&self) -> bool;
    pub fn len(&self) -> usize;
    pub fn to_problem_details(&self, instance: &str, title: &str) -> ProblemDetails;
    pub fn to_grpc_details(&self) -> GrpcErrorDetails;
}
```

#### Validate トレイト

```rust
pub trait Validate {
    fn validate(&self) -> Result<(), ValidationErrors>;
}
```

### 使用例

```rust
use k1s0_validation::{ValidationErrors, FieldError, Validate};

#[derive(Debug)]
struct CreateUserRequest {
    name: String,
    email: String,
    password: String,
}

impl Validate for CreateUserRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if self.name.is_empty() {
            errors.add_field_error(FieldError::required("name"));
        }

        if !self.email.contains('@') {
            errors.add_field_error(
                FieldError::invalid_format("email", "有効なメールアドレスを入力してください")
            );
        }

        if self.password.len() < 8 {
            errors.add_field_error(FieldError::min_length("password", 8));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

---

## k1s0-observability

### 目的

観測性（ログ/トレース/メトリクス）の統一初期化ライブラリ。必須フィールドを強制し、OpenTelemetry と統合する。

### 設計方針

- **必須フィールドの強制**: `service.name`, `env` 等を初期化時に必須化
- **JSON ログの統一**: 構造化ログの必須フィールドを固定
- **OTel 統合**: OpenTelemetry によるトレース/メトリクス

### 必須フィールド（ログ）

| フィールド | 説明 |
|-----------|------|
| `timestamp` | ISO 8601 形式のタイムスタンプ |
| `level` | ログレベル（DEBUG/INFO/WARN/ERROR） |
| `message` | ログメッセージ |
| `service.name` | サービス名 |
| `service.env` | 環境名（dev/stg/prod） |
| `trace.id` | トレース ID（リクエスト相関用） |
| `request.id` | リクエスト ID |

### 主要な型

#### ObservabilityConfig

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

#### RequestContext

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

#### LogEntry

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

### 使用例

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

---

## k1s0-grpc-server

### 目的

gRPC サーバの共通基盤を提供する。OTel/ログ/メトリクスの共通エントリ、トレースコンテキスト伝播、error_code/status 統一を実現。

### 設計原則

1. **共通インターセプタ**: "最低限の礼儀"をテンプレで自動有効
2. **error_code 必須**: エラー時は必ず error_code を付与
3. **デッドライン検知**: クライアントがデッドラインを指定していない場合の検知
4. **構造化ログ**: JSON 形式で必須フィールドを統一

### デッドラインポリシー

```rust
pub enum DeadlinePolicy {
    /// 許可（ログ/メトリクスのみ）
    Allow,
    /// 警告（ログ/メトリクス + ヘッダ通知）
    Warn,
    /// 拒否（INVALID_ARGUMENT で返す）
    Reject,
}
```

### 主要な型

#### GrpcServerConfig

```rust
pub struct GrpcServerConfig {
    service_name: String,
    env: String,
    port: u16,
    interceptor: InterceptorConfig,
    tls: Option<TlsConfig>,
}

impl GrpcServerConfig {
    pub fn builder() -> GrpcServerConfigBuilder;
}
```

#### RequestContext

```rust
pub struct RequestContext {
    pub trace_id: String,
    pub request_id: String,
    pub tenant_id: Option<String>,
    pub deadline: Option<Instant>,
}
```

#### ResponseMetadata

```rust
pub struct ResponseMetadata {
    pub trace_id: Option<String>,
    pub request_id: Option<String>,
    pub error_code: Option<String>,
}

impl ResponseMetadata {
    pub fn from_context(ctx: &RequestContext) -> Self;
    pub fn with_error_code(self, code: impl Into<String>) -> Self;
}
```

#### RequestLog

```rust
pub struct RequestLog {
    level: LogLevel,
    message: String,
    service_name: String,
    env: String,
    trace_id: String,
    request_id: String,
    grpc_service: Option<String>,
    grpc_method: Option<String>,
    grpc_status: Option<GrpcStatusCode>,
    latency_ms: Option<f64>,
}

impl RequestLog {
    pub fn new(
        level: LogLevel,
        message: impl Into<String>,
        service_name: impl Into<String>,
        env: impl Into<String>,
        ctx: &RequestContext,
    ) -> Self;

    pub fn with_grpc(
        self,
        service: impl Into<String>,
        method: impl Into<String>,
        status: GrpcStatusCode,
    ) -> Self;

    pub fn with_latency(self, latency_ms: f64) -> Self;
    pub fn to_json(&self) -> Result<String>;
}
```

### 使用例

```rust
use k1s0_grpc_server::{
    GrpcServerConfig, RequestContext, ResponseMetadata, RequestLog,
};
use k1s0_grpc_server::error::{GrpcStatusCode, LogLevel};

// サーバ設定
let config = GrpcServerConfig::builder()
    .service_name("my-service")
    .env("dev")
    .port(50051)
    .build()
    .unwrap();

// リクエストコンテキスト
let ctx = RequestContext::new();

// レスポンスメタデータ
let resp = ResponseMetadata::from_context(&ctx)
    .with_error_code("USER_NOT_FOUND");

// リクエストログ
let log = RequestLog::new(
    LogLevel::Info,
    "request completed",
    "my-service",
    "dev",
    &ctx,
)
.with_grpc("UserService", "GetUser", GrpcStatusCode::Ok)
.with_latency(42.5);

println!("{}", log.to_json().unwrap());
```

---

## k1s0-resilience

### 目的

依存先呼び出しのガードレールを提供する。タイムアウト、同時実行制限、バルクヘッド、サーキットブレーカをサポート。

### 設計原則

1. **タイムアウト必須**: 無制限待機を防ぐ
2. **同時実行制限**: リソース枯渇を防ぐ
3. **障害隔離**: バルクヘッドで障害の波及を防ぐ
4. **サーキットブレーカ**: 必要時のみ有効化（既定OFF）

### タイムアウト

```rust
pub struct TimeoutConfig {
    timeout_ms: u64,
}

pub const MIN_TIMEOUT_MS: u64 = 100;      // 100ms
pub const MAX_TIMEOUT_MS: u64 = 300_000;  // 5分
pub const DEFAULT_TIMEOUT_MS: u64 = 30_000; // 30秒

impl TimeoutConfig {
    pub fn new(timeout_ms: u64) -> Self;
    pub fn validate(&self) -> Result<(), ResilienceError>;
}

pub struct TimeoutGuard {
    config: TimeoutConfig,
}

impl TimeoutGuard {
    pub fn new(config: TimeoutConfig) -> Result<Self, ResilienceError>;
    pub fn default_timeout() -> Self;

    pub async fn execute<F, T, E>(&self, future: F) -> Result<T, ResilienceError>
    where
        F: Future<Output = Result<T, E>>,
        E: Into<ResilienceError>;
}
```

### 同時実行制限

```rust
pub struct ConcurrencyConfig {
    max_concurrent: usize,
}

pub struct ConcurrencyLimiter {
    semaphore: Arc<Semaphore>,
}

impl ConcurrencyLimiter {
    pub fn new(config: ConcurrencyConfig) -> Self;
    pub fn default_config() -> Self;

    pub async fn execute<F, T, E>(&self, future: F) -> Result<T, ResilienceError>
    where
        F: Future<Output = Result<T, E>>,
        E: Into<ResilienceError>;
}
```

### バルクヘッド

```rust
pub struct BulkheadConfig {
    default_limit: usize,
    service_limits: HashMap<String, usize>,
}

impl BulkheadConfig {
    pub fn new(default_limit: usize) -> Self;
    pub fn with_service_limit(self, service: impl Into<String>, limit: usize) -> Self;
}

pub struct Bulkhead {
    config: BulkheadConfig,
    semaphores: HashMap<String, Arc<Semaphore>>,
}

impl Bulkhead {
    pub fn new(config: BulkheadConfig) -> Self;
    pub fn default_config() -> Self;

    pub async fn execute<F, T, E>(
        &self,
        service: &str,
        future: F,
    ) -> Result<T, ResilienceError>
    where
        F: Future<Output = Result<T, E>>,
        E: Into<ResilienceError>;
}
```

### サーキットブレーカ

```rust
pub enum CircuitState {
    /// 正常（呼び出し許可）
    Closed,
    /// 障害検知（呼び出し遮断）
    Open,
    /// 回復試行中
    HalfOpen,
}

pub struct CircuitBreakerConfig {
    enabled: bool,
    failure_threshold: u32,   // 障害と判定する連続失敗回数
    success_threshold: u32,   // 回復と判定する連続成功回数
    reset_timeout_secs: u64,  // Open -> HalfOpen への遷移時間
}

impl CircuitBreakerConfig {
    pub fn enabled() -> CircuitBreakerConfigBuilder;
    pub fn disabled() -> Self;
}

pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: AtomicU8,
    failure_count: AtomicU32,
    success_count: AtomicU32,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self;
    pub fn disabled() -> Self;

    pub fn allow_request(&self) -> bool;
    pub fn state(&self) -> CircuitState;

    pub async fn execute<F, T, E>(&self, future: F) -> Result<T, ResilienceError>
    where
        F: Future<Output = Result<T, E>>,
        E: Into<ResilienceError>;
}
```

### ResilienceError

```rust
pub struct ResilienceError {
    kind: ResilienceErrorKind,
    message: String,
}

pub enum ResilienceErrorKind {
    Timeout,
    ConcurrencyExceeded,
    CircuitOpen,
    BulkheadRejected,
}

impl ResilienceError {
    pub fn timeout(timeout_ms: u64) -> Self;
    pub fn concurrency_exceeded() -> Self;
    pub fn circuit_open() -> Self;
    pub fn error_code(&self) -> &'static str;
    pub fn is_retryable(&self) -> bool;
}
```

### 使用例

```rust
use k1s0_resilience::{
    TimeoutConfig, TimeoutGuard,
    ConcurrencyConfig, ConcurrencyLimiter,
    BulkheadConfig, Bulkhead,
    CircuitBreakerConfig, CircuitBreaker,
    ResilienceError,
};

// タイムアウト
let guard = TimeoutGuard::new(TimeoutConfig::new(5000))?;
let result = guard.execute(async {
    // 処理
    Ok::<_, ResilienceError>(42)
}).await?;

// 同時実行制限
let limiter = ConcurrencyLimiter::new(ConcurrencyConfig::new(10));
let result = limiter.execute(async {
    Ok::<_, ResilienceError>(42)
}).await?;

// バルクヘッド
let bulkhead = Bulkhead::new(
    BulkheadConfig::new(100)
        .with_service_limit("auth-service", 10)
        .with_service_limit("config-service", 20)
);
let result = bulkhead.execute("auth-service", async {
    Ok::<_, ResilienceError>(42)
}).await?;

// サーキットブレーカ
let cb = CircuitBreaker::new(
    CircuitBreakerConfig::enabled()
        .failure_threshold(5)
        .success_threshold(3)
        .reset_timeout_secs(30)
        .build()
);
let result = cb.execute(async {
    Ok::<_, ResilienceError>(42)
}).await?;
```

---

## 依存関係

```
k1s0-error
    └── (standalone)

k1s0-config
    └── (standalone)

k1s0-validation
    └── (standalone)

k1s0-observability
    └── (standalone)

k1s0-grpc-server
    ├── k1s0-error
    └── k1s0-observability

k1s0-resilience
    └── (standalone)
```

---

## 今後の実装予定

1. **k1s0-grpc-client**: gRPC クライアント共通基盤
2. **k1s0-health**: ヘルスチェックエンドポイント
3. **k1s0-db**: DB 接続プール、トランザクション管理
4. **k1s0-auth**: 認証・認可ミドルウェア
5. **k1s0-cache**: Redis キャッシュ統合
