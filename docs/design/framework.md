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
├── k1s0-grpc-client/   # gRPC クライアント共通
├── k1s0-resilience/    # レジリエンスパターン
├── k1s0-health/        # ヘルスチェック
├── k1s0-cache/         # Redis キャッシュ
├── k1s0-db/            # DB 接続・トランザクション
└── k1s0-auth/          # 認証・認可
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

## k1s0-grpc-client

### 目的

gRPC クライアント呼び出しの共通基盤を提供する。deadline 必須、retry 原則 0、サービスディスカバリをサポート。

### 設計原則

1. **deadline 必須**: 無制限呼び出しを防ぐ（100ms〜5分）
2. **retry 原則 0**: リトライは明示的な opt-in（ADR 参照必須）
3. **トレース伝播**: W3C Trace Context の自動付与
4. **サービスディスカバリ**: K8s DNS 形式での論理名解決

### 主要な型

#### GrpcClientConfig

```rust
pub struct GrpcClientConfig {
    timeout_ms: u64,              // デフォルト: 30秒
    connect_timeout_ms: u64,      // デフォルト: 5秒
    retry: RetryConfig,           // デフォルト: 無効
    tls: TlsConfig,
}

pub const MIN_TIMEOUT_MS: u64 = 100;
pub const MAX_TIMEOUT_MS: u64 = 300_000;  // 5分
```

#### GrpcClientBuilder

```rust
pub struct GrpcClientBuilder {
    service_name: String,
    target_address: Option<String>,
    target_service: Option<String>,
    config: GrpcClientConfig,
    discovery: Option<ServiceDiscoveryConfig>,
}

impl GrpcClientBuilder {
    pub fn new(service_name: impl Into<String>) -> Self;
    pub fn target_address(self, address: impl Into<String>) -> Self;
    pub fn target_service(self, service: impl Into<String>) -> Self;
    pub fn config(self, config: GrpcClientConfig) -> Self;
    pub fn discovery(self, config: ServiceDiscoveryConfig) -> Self;
    pub fn build(self) -> Result<GrpcClientConnection, GrpcClientError>;
}
```

#### ServiceDiscoveryConfig

```rust
pub struct ServiceDiscoveryConfig {
    default_namespace: String,
    cluster_domain: String,        // デフォルト: "svc.cluster.local"
    default_port: u16,             // デフォルト: 50051
    services: HashMap<String, ServiceEndpoint>,
}

impl ServiceDiscoveryConfig {
    pub fn builder() -> ServiceDiscoveryConfigBuilder;
}
```

#### RetryConfig

```rust
pub struct RetryConfig {
    enabled: bool,
    adr_reference: Option<String>,  // ADR 参照（有効化時必須）
    max_attempts: u32,
}

impl RetryConfig {
    pub fn disabled() -> Self;
    pub fn enabled(adr_reference: impl Into<String>) -> RetryConfigBuilder;
}
```

#### CallOptions

```rust
pub struct CallOptions {
    timeout_ms: Option<u64>,
    metadata: RequestMetadata,
}

impl CallOptions {
    pub fn new() -> Self;
    pub fn with_timeout_ms(self, timeout_ms: u64) -> Self;
    pub fn with_trace_id(self, trace_id: impl Into<String>) -> Self;
    pub fn with_request_id(self, request_id: impl Into<String>) -> Self;
    pub fn with_tenant_id(self, tenant_id: impl Into<String>) -> Self;
}
```

### 使用例

```rust
use k1s0_grpc_client::{
    GrpcClientBuilder, GrpcClientConfig, ServiceDiscoveryConfig,
    ServiceEndpoint, CallOptions, RetryConfig,
};

// 直接アドレス指定
let conn = GrpcClientBuilder::new("my-service")
    .target_address("localhost:50051")
    .build()?;

// サービスディスカバリ経由
let discovery = ServiceDiscoveryConfig::builder()
    .default_namespace("production")
    .service("auth-service", ServiceEndpoint::new("auth.example.com", 50051))
    .build();

let conn = GrpcClientBuilder::new("my-service")
    .target_service("auth-service")
    .discovery(discovery)
    .build()?;

// 呼び出しオプション
let options = CallOptions::new()
    .with_timeout_ms(5000)
    .with_trace_id("abc123")
    .with_request_id("req-001");
```

---

## k1s0-health

### 目的

Kubernetes 対応のヘルスチェック機能を提供する。readiness/liveness/startup プローブをサポート。

### 設計原則

1. **3段階ステータス**: Healthy / Degraded / Unhealthy
2. **コンポーネント単位**: 各コンポーネント（DB、キャッシュ等）の個別ステータス
3. **K8s プローブ対応**: readiness/liveness/startup
4. **Graceful shutdown**: readiness 状態の動的切り替え

### 主要な型

#### HealthStatus

```rust
pub enum HealthStatus {
    Healthy,      // すべて正常
    Degraded,     // 一部機能低下
    Unhealthy,    // サービス不可
}

impl HealthStatus {
    pub fn to_http_status_code(&self) -> u16;
    pub fn is_healthy(&self) -> bool;
    pub fn is_serving(&self) -> bool;  // Healthy | Degraded
    pub fn merge(self, other: Self) -> Self;
}
```

#### ComponentHealth

```rust
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl ComponentHealth {
    pub fn healthy(name: impl Into<String>) -> Self;
    pub fn degraded(name: impl Into<String>, error: impl Into<String>) -> Self;
    pub fn unhealthy(name: impl Into<String>, error: impl Into<String>) -> Self;
    pub fn with_latency_ms(self, latency: u64) -> Self;
}
```

#### HealthResponse

```rust
pub struct HealthResponse {
    pub status: HealthStatus,
    pub service: String,
    pub version: Option<String>,
    pub components: Vec<ComponentHealth>,
}

impl HealthResponse {
    pub fn new(service: impl Into<String>) -> Self;
    pub fn with_version(self, version: impl Into<String>) -> Self;
    pub fn with_component(self, component: ComponentHealth) -> Self;
}
```

#### ProbeHandler

```rust
pub struct ProbeHandler {
    service_name: String,
    version: Option<String>,
    readiness: Option<Arc<ReadinessState>>,
}

impl ProbeHandler {
    pub fn new(service_name: impl Into<String>) -> Self;
    pub fn with_version(self, version: impl Into<String>) -> Self;
    pub fn with_readiness(self, state: Arc<ReadinessState>) -> Self;
    pub fn liveness(&self) -> HealthResponse;
    pub fn readiness(&self) -> HealthResponse;
    pub fn startup(&self) -> HealthResponse;
}
```

#### ReadinessState

```rust
pub struct ReadinessState {
    ready: AtomicBool,
}

impl ReadinessState {
    pub fn ready() -> Self;
    pub fn not_ready() -> Self;
    pub fn is_ready(&self) -> bool;
    pub fn set_ready(&self);
    pub fn set_not_ready(&self);
}
```

### 使用例

```rust
use k1s0_health::{HealthResponse, HealthStatus, ComponentHealth};
use k1s0_health::probe::{ProbeHandler, ReadinessState};
use std::sync::Arc;

// ヘルスレスポンス
let response = HealthResponse::new("my-service")
    .with_version("1.0.0")
    .with_component(ComponentHealth::healthy("database"))
    .with_component(ComponentHealth::healthy("cache"));

assert_eq!(response.status, HealthStatus::Healthy);

// プローブハンドラー
let readiness = Arc::new(ReadinessState::ready());
let handler = ProbeHandler::new("my-service")
    .with_version("1.0.0")
    .with_readiness(readiness.clone());

// Graceful shutdown
readiness.set_not_ready();
```

---

## k1s0-db

### 目的

データベース接続、プール管理、トランザクション、リポジトリパターンの標準化を提供する。

### 設計原則

1. **Clean Architecture 対応**: domain/application 層用インターフェース
2. **トランザクション境界**: Unit of Work パターン
3. **リポジトリパターン**: CRUD 抽象化、ページング対応
4. **PostgreSQL 重視**: SQLx による実装

### 主要な型

#### DbConfig

```rust
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password_file: Option<String>,
    pub ssl_mode: SslMode,
    pub pool: PoolConfig,
    pub timeout: TimeoutConfig,
}

impl DbConfig {
    pub fn builder() -> DbConfigBuilder;
}
```

#### PoolConfig

```rust
pub struct PoolConfig {
    pub max_connections: u32,      // デフォルト: 10
    pub min_connections: u32,      // デフォルト: 1
    pub idle_timeout_secs: u64,    // デフォルト: 600
    pub max_lifetime_secs: u64,    // デフォルト: 1800
}
```

#### TransactionOptions

```rust
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,      // デフォルト
    RepeatableRead,
    Serializable,
}

pub enum TransactionMode {
    ReadWrite,          // デフォルト
    ReadOnly,
}

pub struct TransactionOptions {
    pub isolation_level: IsolationLevel,
    pub mode: TransactionMode,
}

impl TransactionOptions {
    pub fn new() -> Self;
    pub fn read_only() -> Self;
    pub fn serializable() -> Self;
    pub fn with_isolation_level(self, level: IsolationLevel) -> Self;
}
```

#### Repository トレイト

```rust
#[async_trait]
pub trait Repository<T, ID: ?Sized>: Send + Sync {
    async fn find_by_id(&self, id: &ID) -> DbResult<Option<T>>;
    async fn find_all(&self) -> DbResult<Vec<T>>;
    async fn save(&self, entity: &T) -> DbResult<T>;
    async fn delete(&self, id: &ID) -> DbResult<bool>;
}

#[async_trait]
pub trait PagedRepository<T, ID>: Repository<T, ID> {
    async fn find_paginated(&self, pagination: &Pagination) -> DbResult<PagedResult<T>>;
}
```

#### Pagination

```rust
pub struct Pagination {
    pub page: u64,          // 1から開始
    pub page_size: u64,     // 1-1000
}

pub struct PagedResult<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

impl<T> PagedResult<T> {
    pub fn has_next_page(&self) -> bool;
    pub fn has_prev_page(&self) -> bool;
}
```

#### Unit of Work

```rust
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    async fn begin(&self) -> DbResult<()>;
    async fn commit(&self) -> DbResult<()>;
    async fn rollback(&self) -> DbResult<()>;
}

pub async fn execute_in_transaction<F, T, E>(
    uow: &impl UnitOfWork,
    f: F,
) -> DbResult<T>
where
    F: FnOnce() -> Future<Output = Result<T, E>>,
    E: Into<DbError>;
```

### Features

```toml
[features]
default = []
postgres = ["sqlx"]
full = ["postgres"]
```

### 使用例

```rust
use k1s0_db::{DbConfig, DbPoolBuilder, Repository, Pagination};

// 接続設定
let config = DbConfig::builder()
    .host("localhost")
    .database("myapp")
    .username("app_user")
    .password_file("/run/secrets/db_password")
    .build()?;

// プール作成
let pool = DbPoolBuilder::new()
    .host(&config.host)
    .database(&config.database)
    .build()
    .await?;

// ページネーション
let pagination = Pagination { page: 1, page_size: 20 };
let result = repository.find_paginated(&pagination).await?;
```

---

## k1s0-auth

### 目的

JWT/OIDC 検証、ポリシー評価、監査ログの統一化を提供する。

### 設計原則

1. **JWT/OIDC 統一**: JWKS 自動更新、複数キーのローテーション対応
2. **ポリシー柔軟性**: RBAC/ABAC 両対応
3. **監査ログ**: 全認証・認可操作の記録
4. **ミドルウェア統合**: Axum/Tonic 両対応

### 主要な型

#### Claims

```rust
pub struct Claims {
    pub sub: String,                // ユーザーID
    pub iss: String,                // 発行者
    pub aud: Option<AudienceClaim>, // 対象者
    pub exp: i64,                   // 有効期限
    pub iat: i64,                   // 発行日時
    pub roles: Vec<String>,         // ロール
    pub permissions: Vec<String>,   // パーミッション
    pub tenant_id: Option<String>,  // テナントID
}
```

#### JwtVerifier

```rust
pub struct JwtVerifierConfig {
    issuer: String,
    jwks_uri: String,
    audience: Option<String>,
    jwks_cache_ttl_secs: u64,
}

impl JwtVerifierConfig {
    pub fn new(issuer: impl Into<String>) -> Self;
    pub fn with_jwks_uri(self, uri: impl Into<String>) -> Self;
    pub fn with_audience(self, audience: impl Into<String>) -> Self;
}

pub struct JwtVerifier {
    config: JwtVerifierConfig,
}

impl JwtVerifier {
    pub fn new(config: JwtVerifierConfig) -> Self;
    pub async fn verify(&self, token: &str) -> Result<Claims, AuthError>;
}
```

#### PolicyEvaluator

```rust
pub enum PolicyDecision {
    Allow,
    Deny,
    NotApplicable,
}

pub struct PolicyRequest {
    pub subject: PolicySubject,
    pub action: Action,
    pub resource: ResourceContext,
}

pub struct PolicyResult {
    pub decision: PolicyDecision,
    pub reason: Option<String>,
    pub matched_rules: Vec<String>,
}

pub struct PolicyEvaluator {
    rules: Arc<RwLock<Vec<PolicyRule>>>,
}

impl PolicyEvaluator {
    pub fn new() -> Self;
    pub async fn add_rules(&self, rules: Vec<PolicyRule>);
    pub async fn evaluate(&self, request: &PolicyRequest) -> PolicyResult;
}
```

#### AuditLogger

```rust
pub enum AuditEventType {
    AuthenticationSuccess,
    AuthenticationFailure,
    AuthorizationSuccess,
    AuthorizationFailure,
    DataAccess,
    DataModification,
}

pub struct AuditEvent {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub actor: AuditActor,
    pub resource: Option<AuditResource>,
    pub action: String,
    pub result: AuditResult,
}

pub struct AuditLogger {
    service_name: String,
}

impl AuditLogger {
    pub fn with_default_sink(service_name: &str) -> Self;
    pub fn log_authentication_success(&self, actor: AuditActor);
    pub fn log_authentication_failure(&self, actor: AuditActor, reason: &str);
    pub fn log_authorization(&self, request: &PolicyRequest, decision: PolicyDecision);
}
```

### Features

```toml
[features]
default = []
axum-layer = ["axum", "tower"]
tonic-interceptor = ["tonic"]
redis-cache = ["k1s0-cache/redis"]
postgres-policy = ["k1s0-db/postgres"]
full = ["axum-layer", "tonic-interceptor", "redis-cache", "postgres-policy"]
```

### 使用例

```rust
use k1s0_auth::{JwtVerifier, JwtVerifierConfig, PolicyEvaluator, AuditLogger};
use k1s0_auth::policy::{PolicyBuilder, PolicySubject, Action, PolicyRequest};

// JWT検証
let config = JwtVerifierConfig::new("https://auth.example.com")
    .with_jwks_uri("https://auth.example.com/.well-known/jwks.json")
    .with_audience("my-api");

let verifier = JwtVerifier::new(config);
let claims = verifier.verify("eyJ...").await?;

// ポリシー評価
let evaluator = PolicyEvaluator::new();
let rules = PolicyBuilder::new()
    .admin_rule("admin")
    .read_rule("user_read", "user", vec!["user"], 10)
    .build();
evaluator.add_rules(rules).await;

let subject = PolicySubject::new("user123").with_role("admin");
let action = Action::new("user", "delete");
let request = PolicyRequest {
    subject,
    action,
    resource: ResourceContext::default(),
};
let result = evaluator.evaluate(&request).await;

// 監査ログ
let logger = AuditLogger::with_default_sink("my-service");
logger.log_authentication_success(AuditActor::new("user123"));
```

---

## k1s0-cache

### 目的

Redis キャッシュクライアントの標準化を提供する。Cache-Aside パターン、TTL 管理をサポート。

### 主要な型

#### CacheConfig

```rust
pub struct CacheConfig {
    pub host: String,
    pub port: u16,
    pub key_prefix: String,
    pub default_ttl_secs: Option<u64>,
}

impl CacheConfig {
    pub fn builder() -> CacheConfigBuilder;
}
```

#### CacheOperations トレイト

```rust
#[async_trait]
pub trait CacheOperations: Send + Sync {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> CacheResult<Option<T>>;
    async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Option<Duration>) -> CacheResult<()>;
    async fn delete(&self, key: &str) -> CacheResult<bool>;
    async fn exists(&self, key: &str) -> CacheResult<bool>;
    async fn get_or_set<T, F, Fut>(&self, key: &str, f: F, ttl: Option<Duration>) -> CacheResult<T>;
}
```

### Features

```toml
[features]
default = []
redis = ["dep:redis", "dep:bb8", "dep:bb8-redis"]
health = ["dep:k1s0-health"]
full = ["redis", "health"]
```

### 使用例

```rust
use k1s0_cache::{CacheConfig, CacheClient, CacheOperations};
use std::time::Duration;

let config = CacheConfig::builder()
    .host("localhost")
    .port(6379)
    .key_prefix("myapp")
    .build()?;

let client = CacheClient::new(config).await?;

// 値の設定
client.set("user:123", &user, Some(Duration::from_secs(3600))).await?;

// 値の取得
let user: Option<User> = client.get("user:123").await?;

// Cache-Aside パターン
let user = client.get_or_set(
    "user:123",
    || async { db.find_user("123").await },
    Some(Duration::from_secs(3600)),
).await?;
```

---

## 依存関係

```
k1s0-error          # 基盤（依存なし）
k1s0-config         # 基盤（依存なし）
k1s0-validation     # 基盤（依存なし）
k1s0-observability  # 基盤（依存なし）
k1s0-resilience     # 基盤（依存なし）

k1s0-grpc-server    # インフラ
  ├── k1s0-error
  └── k1s0-observability

k1s0-grpc-client    # インフラ（依存なし）

k1s0-health         # インフラ（依存なし）

k1s0-cache          # 業務
  └── k1s0-health (feature="health")

k1s0-db             # 業務
  └── sqlx (feature="postgres")

k1s0-auth           # 業務
  ├── k1s0-cache (feature="redis-cache")
  ├── k1s0-db (feature="postgres-policy")
  ├── axum, tower (feature="axum-layer")
  └── tonic (feature="tonic-interceptor")
```

---

# Frontend Framework

## 概要

k1s0 Frontend Framework は、フロントエンド開発のための共通パッケージ群を提供します。個別機能チームが「画面の中身」以外（ナビゲーション/レイアウト/デザイン/権限制御/設定読込/観測）を再実装せずに済む状態を実現します。

## React パッケージ一覧

```
framework/frontend/react/packages/
├── @k1s0/navigation/     # 設定駆動ナビゲーション（実装済み）
├── @k1s0/config/         # YAML設定管理（実装済み）
├── @k1s0/api-client/     # API通信クライアント（実装済み）
├── @k1s0/ui/             # Design System（実装済み）
├── @k1s0/shell/          # AppShell（実装済み）
├── @k1s0/auth-client/    # 認証クライアント（実装済み）
├── @k1s0/observability/  # OTel/ログ（実装済み）
├── eslint-config-k1s0/   # ESLint設定（未実装）
└── tsconfig-k1s0/        # TypeScript設定（未実装）
```

### 実装状況

| パッケージ | 状態 | 説明 |
|-----------|:----:|------|
| @k1s0/navigation | ✅ | 設定駆動ナビゲーション、React Router統合、権限/feature flag制御 |
| @k1s0/config | ✅ | YAML設定読み込み、Zodスキーマバリデーション、環境マージ |
| @k1s0/api-client | ✅ | fetchベースAPI通信、OTel計測、ProblemDetailsエラー |
| @k1s0/ui | ✅ | Material-UI v5/v6 Design System、テーマ、フォーム、フィードバック |
| @k1s0/shell | ✅ | AppShell（Header/Sidebar/Footer）、レスポンシブ対応 |
| @k1s0/auth-client | ✅ | JWT/OIDCトークン管理、認証ガード、セッション管理 |
| @k1s0/observability | ✅ | OpenTelemetry統合、構造化ログ、Web Vitals計測 |

---

## @k1s0/navigation

### 目的

`config/{env}.yaml` の設定からルート/メニュー/フローを自動構築し、権限・feature flag による表示制御を行う。

### 主要コンポーネント

| コンポーネント | 説明 |
|---------------|------|
| `ConfigRouter` | YAML設定からReact Routerルートを自動生成 |
| `NavigationProvider` | ナビゲーション状態のContext提供 |
| `MenuBuilder` | 設定からメニュー構造を構築 |
| `FlowController` | マルチステップフロー制御 |
| `PermissionGuard` | 権限ベースのルートガード |
| `FlagGuard` | feature flagベースのガード |

### 使用例

```tsx
import { ConfigRouter, NavigationProvider } from '@k1s0/navigation';
import { useConfig } from '@k1s0/config';

function App() {
  const config = useConfig();

  return (
    <NavigationProvider>
      <ConfigRouter config={config.ui.navigation} />
    </NavigationProvider>
  );
}
```

---

## @k1s0/config

### 目的

YAML設定ファイルの読み込み、型付け、バリデーションを提供する。

### 主要機能

| モジュール | 説明 |
|-----------|------|
| `schema` | Zodスキーマ定義（apiConfigSchema, authConfigSchema, appConfigSchema） |
| `loader` | ConfigLoader, loadConfigFromUrl, parseConfig |
| `merge` | deepMerge, mergeConfigs, mergeEnvironmentConfig |

### 使用例

```tsx
import { ConfigLoader, validateConfig } from '@k1s0/config';

const loader = new ConfigLoader({
  baseUrl: '/config',
  env: 'dev',
});

const config = await loader.load();
const validated = validateConfig(config);
```

---

## @k1s0/api-client

### 目的

API通信の標準化、OpenTelemetry計測、エラーハンドリングを提供する。

### 主要コンポーネント

| コンポーネント | 説明 |
|---------------|------|
| `ApiClient` | fetchベースのHTTPクライアント |
| `ApiClientProvider` | Context提供 |
| `TokenManager` | 認証トークン管理 |
| `OTelTracer` | OpenTelemetry計測 |
| `ErrorBoundary` | エラー境界コンポーネント |
| `useApiRequest` | API呼び出しフック |

### 使用例

```tsx
import { useApiRequest } from '@k1s0/api-client';

function UserList() {
  const { data, loading, error } = useApiRequest('/api/users');

  if (loading) return <Loading />;
  if (error) return <ErrorDisplay error={error} />;

  return <ul>{data.map(user => <li key={user.id}>{user.name}</li>)}</ul>;
}
```

---

## @k1s0/ui

### 目的

k1s0 Design/UX 標準コンポーネントライブラリを提供する。Material-UI v5/v6 をベースに統一されたデザインシステムを実現。

### モジュール構成

| モジュール | 内容 |
|-----------|------|
| `theme/` | K1s0ThemeProvider, createK1s0Theme, palette, typography, spacing |
| `form/` | FormContainer, FormField, validation, types |
| `feedback/` | Toast, ConfirmDialog, FeedbackProvider |
| `state/` | Loading, EmptyState |

### 使用例

```tsx
import { K1s0ThemeProvider, FormContainer, FormField, Toast } from '@k1s0/ui';

function App() {
  return (
    <K1s0ThemeProvider>
      <FormContainer onSubmit={handleSubmit}>
        <FormField name="email" label="メールアドレス" required />
        <FormField name="password" label="パスワード" type="password" />
      </FormContainer>
      <Toast />
    </K1s0ThemeProvider>
  );
}
```

---

## @k1s0/shell

### 目的

AppShell（Header/Sidebar/Footer）の標準レイアウトを提供する。

### 主要コンポーネント

| コンポーネント | 説明 |
|---------------|------|
| `AppShell` | メインレイアウトコンテナ |
| `Header` | ヘッダーコンポーネント |
| `Sidebar` | サイドバー（メニュー）コンポーネント |
| `Footer` | フッターコンポーネント |
| `useResponsiveLayout` | レスポンシブ対応フック |

### 使用例

```tsx
import { AppShell, Header, Sidebar, Footer } from '@k1s0/shell';
import { useConfig } from '@k1s0/config';

function Layout({ children }) {
  const config = useConfig();

  return (
    <AppShell>
      <Header title={config.app.name} />
      <Sidebar menuItems={config.ui.navigation.menus} />
      <main>{children}</main>
      <Footer />
    </AppShell>
  );
}
```

---

## @k1s0/auth-client

### 目的

JWT/OIDC 認証クライアントを提供する。トークン管理、認証状態管理、認証ガード、セッション管理を実現。

### モジュール構成

| モジュール | 内容 |
|-----------|------|
| `token/` | TokenManager, SessionTokenStorage, LocalTokenStorage, MemoryTokenStorage, decoder |
| `provider/` | AuthProvider, useAuth, useAuthState, useIsAuthenticated, useUser, usePermissions |
| `guard/` | AuthGuard, RequireAuth, RequireRole, RequirePermission |
| `session/` | SessionManager, useSession |

### 主要な型

```typescript
interface Claims {
  sub: string;           // ユーザーID
  iss: string;           // 発行者
  aud?: string | string[]; // 対象者
  exp: number;           // 有効期限
  iat: number;           // 発行日時
  roles?: string[];      // ロール
  permissions?: string[]; // パーミッション
  tenant_id?: string;    // テナントID
}

interface AuthState {
  status: 'idle' | 'loading' | 'authenticated' | 'unauthenticated' | 'error';
  user: AuthUser | null;
  error: AuthError | null;
}

interface AuthClientConfig {
  tokenStorage?: TokenStorage;
  refreshToken?: TokenRefresher;
  refreshThreshold?: number;  // デフォルト: 300秒（5分）
  autoRefresh?: boolean;
}
```

### 使用例

```tsx
import { AuthProvider, useAuth, RequireAuth } from '@k1s0/auth-client';

// アプリのルートで AuthProvider をラップ
function App() {
  const refreshToken = async (token: string) => {
    const response = await fetch('/api/auth/refresh', {
      method: 'POST',
      headers: { Authorization: `Bearer ${token}` },
    });
    return response.json();
  };

  return (
    <AuthProvider config={{ refreshToken, autoRefresh: true }}>
      <Router />
    </AuthProvider>
  );
}

// 認証が必要なページで RequireAuth を使用
function ProtectedPage() {
  return (
    <RequireAuth redirectTo="/login" navigate={navigate}>
      <Dashboard />
    </RequireAuth>
  );
}

// useAuth フックで認証状態を取得
function UserProfile() {
  const { user, logout, isAuthenticated } = useAuth();

  if (!isAuthenticated) return null;

  return (
    <div>
      <p>ようこそ、{user.name} さん</p>
      <button onClick={logout}>ログアウト</button>
    </div>
  );
}

// ロールベースの認可
function AdminPanel() {
  return (
    <RequireRole roles={['admin']} fallback={<AccessDenied />}>
      <AdminDashboard />
    </RequireRole>
  );
}
```

---

## @k1s0/observability

### 目的

フロントエンド向け観測性ライブラリを提供する。OpenTelemetry 統合、構造化ログ、エラートラッキング、パフォーマンス計測を実現。

### モジュール構成

| モジュール | 内容 |
|-----------|------|
| `tracing/` | TracingService, SpanBuilder |
| `logging/` | Logger, ConsoleLogSink, BufferedLogSink |
| `metrics/` | MetricsCollector, Web Vitals |
| `errors/` | ErrorTracker, グローバルエラーハンドリング |
| `provider/` | ObservabilityProvider, useTracing, useLogger, useMetrics, useErrorTracker |
| `utils/` | generateTraceId, generateSpanId, parseTraceparent |

### 必須フィールド（ログ）

バックエンド（k1s0-observability）と同じ必須フィールドをフロントエンドでも強制。

| フィールド | 説明 |
|-----------|------|
| `timestamp` | ISO 8601 形式のタイムスタンプ |
| `level` | ログレベル（debug/info/warn/error） |
| `message` | ログメッセージ |
| `service_name` | サービス名 |
| `env` | 環境名（dev/stg/prod） |
| `trace_id` | トレース ID（リクエスト相関用） |
| `span_id` | スパン ID |

### 主要な型

```typescript
interface ObservabilityConfig {
  serviceName: string;
  env: string;
  version?: string;
  logLevel?: LogLevel;
  enableTracing?: boolean;
  enableMetrics?: boolean;
  enableErrorTracking?: boolean;
  traceExporter?: TraceExporter;
  logSink?: LogSink;
}

interface LogEntry {
  timestamp: string;
  level: LogLevel;
  message: string;
  service_name: string;
  env: string;
  trace_id?: string;
  span_id?: string;
  request_id?: string;
  [key: string]: unknown;
}

interface SpanInfo {
  traceId: string;
  spanId: string;
  parentSpanId?: string;
  name: string;
  startTime: number;
  endTime?: number;
  status: SpanStatus;
  attributes: Record<string, unknown>;
}
```

### 使用例

```tsx
import {
  ObservabilityProvider,
  useLogger,
  useTracing,
  useSpan,
} from '@k1s0/observability';

// アプリのルートで ObservabilityProvider をラップ
function App() {
  return (
    <ObservabilityProvider
      config={{
        serviceName: 'my-frontend',
        env: 'dev',
        enableTracing: true,
        enableErrorTracking: true,
      }}
    >
      <Router />
    </ObservabilityProvider>
  );
}

// useLogger フックでログ出力
function UserActions() {
  const logger = useLogger();

  const handleClick = () => {
    logger.info('ボタンがクリックされました', { buttonId: 'submit' });
  };

  return <button onClick={handleClick}>送信</button>;
}

// useSpan フックでスパン計測
function DataFetcher() {
  const { startSpan, endSpan } = useTracing();

  const fetchData = async () => {
    const span = startSpan('fetch-user-data');
    try {
      const data = await fetch('/api/users');
      endSpan(span.spanId, 'ok');
      return data;
    } catch (error) {
      endSpan(span.spanId, 'error', { error: error.message });
      throw error;
    }
  };

  return <button onClick={fetchData}>データ取得</button>;
}

// Web Vitals の自動収集
function PerformanceMonitor() {
  const metrics = useMetrics();

  useEffect(() => {
    const listener = (metric) => {
      console.log('Web Vital:', metric);
    };
    metrics.addListener(listener);
    return () => metrics.removeListener(listener);
  }, [metrics]);

  return null;
}
```

---

## Flutter パッケージ一覧

```
framework/frontend/flutter/packages/
├── k1s0_config/         # YAML設定管理
├── k1s0_http/           # API通信クライアント
├── k1s0_auth/           # 認証クライアント
├── k1s0_observability/  # OTel/ログ
├── k1s0_ui/             # Design System
└── k1s0_state/          # 状態管理
```

### 実装状況

| パッケージ | 状態 | 説明 |
|-----------|:----:|------|
| k1s0_config | ✅ | YAML設定管理、Zodスキーマバリデーション、環境マージ |
| k1s0_http | ✅ | Dioベース通信クライアント、OTel計測、ProblemDetails対応 |
| k1s0_auth | ✅ | JWT/OIDC認証、SecureStorage、トークン自動更新 |
| k1s0_observability | ✅ | 構造化ログ、分散トレース、メトリクス収集 |
| k1s0_ui | ✅ | Material 3 Design System、共通ウィジェット、テーマ |
| k1s0_state | ✅ | Riverpod状態管理、AsyncValueヘルパー、永続化 |

---

## k1s0_config (Flutter)

### 目的

YAML 設定ファイルの読み込み、型付け、バリデーション、環境マージを提供する。

### 主要な型

```dart
@freezed
class AppConfig with _$AppConfig {
  const factory AppConfig({
    required ApiConfig api,
    required AuthConfig auth,
    required LoggingConfig logging,
    @Default({}) Map<String, bool> featureFlags,
  }) = _AppConfig;
}

class ConfigLoader {
  ConfigLoader({required String defaultPath, String? environment});
  Future<AppConfig> load();
}
```

### 使用例

```dart
final loader = ConfigLoader(
  defaultPath: 'assets/config/default.yaml',
  environment: 'production',
);
final config = await loader.load();

// Riverpod Provider経由でアクセス
ConfigScope(
  config: config,
  child: MyApp(),
)

// 子ウィジェットで使用
final config = ref.watch(configProvider);
```

---

## k1s0_http (Flutter)

### 目的

Dio ベースの HTTP クライアント。トレース伝播、エラーハンドリング、ProblemDetails 対応を提供。

### 主要な型

```dart
class K1s0HttpClient {
  K1s0HttpClient({required HttpClientConfig config});

  Future<K1s0Response<T>> get<T>(String path, {RequestOptions? options});
  Future<K1s0Response<T>> post<T>(String path, {dynamic data, RequestOptions? options});
  Future<K1s0Response<T>> put<T>(String path, {dynamic data, RequestOptions? options});
  Future<K1s0Response<T>> delete<T>(String path, {RequestOptions? options});
}

@freezed
class ProblemDetails with _$ProblemDetails {
  const factory ProblemDetails({
    required String type,
    required String title,
    required int status,
    String? detail,
    String? instance,
    String? errorCode,
    String? traceId,
  }) = _ProblemDetails;
}

class ApiError {
  final ApiErrorKind kind;
  final String message;
  final ProblemDetails? problemDetails;
}
```

### 使用例

```dart
final client = K1s0HttpClient(
  config: HttpClientConfig(
    baseUrl: 'https://api.example.com',
    timeout: Duration(seconds: 30),
  ),
);

try {
  final response = await client.get<User>('/users/123');
  print(response.data);
} on ApiError catch (e) {
  print('Error: ${e.message}');
  if (e.problemDetails != null) {
    print('Error Code: ${e.problemDetails!.errorCode}');
  }
}
```

---

## k1s0_auth (Flutter)

### 目的

JWT/OIDC 認証クライアント。トークン管理、認証状態管理、認証ガード、GoRouter 統合を提供。

### 主要な型

```dart
@freezed
class Claims with _$Claims {
  const factory Claims({
    required String sub,
    required String iss,
    String? aud,
    required int exp,
    required int iat,
    @Default([]) List<String> roles,
    @Default([]) List<String> permissions,
    String? tenantId,
  }) = _Claims;
}

@freezed
class AuthState with _$AuthState {
  const factory AuthState.initial() = AuthInitial;
  const factory AuthState.loading() = AuthLoading;
  const factory AuthState.authenticated(AuthUser user) = AuthAuthenticated;
  const factory AuthState.unauthenticated() = AuthUnauthenticated;
  const factory AuthState.error(AuthError error) = AuthError;
}

class AuthNotifier extends StateNotifier<AuthState> {
  Future<void> login(String accessToken, {String? refreshToken});
  Future<void> logout();
  Future<void> refreshTokens();
}
```

### 使用例

```dart
// AuthProvider で認証状態を管理
final authState = ref.watch(authProvider);

authState.when(
  initial: () => SplashScreen(),
  loading: () => LoadingScreen(),
  authenticated: (user) => HomePage(),
  unauthenticated: () => LoginPage(),
  error: (error) => ErrorPage(error),
);

// 認証ガード
AuthGuard(
  child: DashboardPage(),
  unauthenticatedBuilder: (context) => LoginPage(),
)

// ロールベースの認可
RequireRole(
  roles: ['admin'],
  child: AdminPanel(),
  fallback: AccessDenied(),
)

// GoRouter 統合
final router = GoRouter(
  redirect: authGuard(
    ref,
    redirectTo: '/login',
    allowedPaths: ['/login', '/register'],
  ),
  routes: [...],
);
```

---

## k1s0_observability (Flutter)

### 目的

フロントエンド向け観測性ライブラリ。構造化ログ、分散トレース、エラートラッキング、パフォーマンスメトリクスを提供。

### 必須フィールド（ログ）

バックエンド（k1s0-observability）と同じ必須フィールドをフロントエンドでも強制。

| フィールド | 説明 |
|-----------|------|
| `timestamp` | ISO 8601 形式のタイムスタンプ |
| `level` | ログレベル（debug/info/warn/error） |
| `message` | ログメッセージ |
| `service_name` | サービス名 |
| `env` | 環境名（dev/stg/prod） |
| `trace_id` | トレース ID（リクエスト相関用） |
| `span_id` | スパン ID |

### 主要な型

```dart
@freezed
class LogEntry with _$LogEntry {
  const factory LogEntry({
    required DateTime timestamp,
    required LogLevel level,
    required String message,
    required String serviceName,
    required String env,
    String? traceId,
    String? spanId,
    @Default({}) Map<String, dynamic> fields,
  }) = _LogEntry;
}

class Logger {
  void debug(String message, [Map<String, dynamic>? fields]);
  void info(String message, [Map<String, dynamic>? fields]);
  void warn(String message, [Map<String, dynamic>? fields]);
  void error(String message, [Object? error, StackTrace? stackTrace]);
}

class Tracer {
  Future<T> trace<T>(String name, Future<T> Function() fn);
  T traceSync<T>(String name, T Function() fn);
}
```

### 使用例

```dart
// Logger の使用
final logger = ref.read(loggerProvider);
logger.info('ユーザーがログインしました', {
  'userId': user.id,
  'loginMethod': 'oauth',
});

// Tracer の使用
final tracer = ref.read(tracerProvider);
final user = await tracer.trace('fetch-user-data', () async {
  return await api.getUser(userId);
});

// エラートラッキング
final errorTracker = ref.read(errorTrackerProvider);
try {
  await riskyOperation();
} catch (e, stackTrace) {
  errorTracker.capture(e, stackTrace);
}
```

---

## k1s0_ui (Flutter)

### 目的

k1s0 Design System を提供する。Material 3 ベースの統一されたテーマ、共通ウィジェット、フォームバリデーション、フィードバックコンポーネントを実現。

### モジュール構成

| モジュール | 内容 |
|-----------|------|
| `theme/` | K1s0Theme, K1s0Colors, K1s0Typography, K1s0Spacing, ThemeProvider |
| `widgets/` | K1s0PrimaryButton, K1s0SecondaryButton, K1s0Card, K1s0TextField |
| `form/` | K1s0Validators, K1s0FormContainer, K1s0FormSection |
| `feedback/` | K1s0Snackbar, K1s0Dialog |
| `state/` | K1s0Loading, K1s0ErrorState, K1s0EmptyState |

### 使用例

```dart
// テーマ設定
MaterialApp(
  theme: ref.watch(themeProvider).lightTheme,
  darkTheme: ref.watch(themeProvider).darkTheme,
  themeMode: ref.watch(themeProvider).themeMode,
)

// ボタン
K1s0PrimaryButton(
  onPressed: () {},
  loading: isSubmitting,
  child: Text('Submit'),
)

// テキストフィールド
K1s0TextField(
  controller: controller,
  label: 'Email',
  validator: K1s0Validators.combine([
    K1s0Validators.required,
    K1s0Validators.email,
  ]),
)

// フィードバック
K1s0Snackbar.success(context, 'Operation completed!');

final confirmed = await K1s0Dialog.confirm(
  context,
  title: 'Delete Item',
  message: 'Are you sure?',
  isDanger: true,
);

// 状態ウィジェット
K1s0Loading(message: 'Loading...')
K1s0ErrorState(message: 'Error occurred', onRetry: _retry)
K1s0EmptyState(title: 'No items', message: 'Add your first item')
```

---

## k1s0_state (Flutter)

### 目的

Riverpod 状態管理ユーティリティを提供する。AsyncValue ヘルパー、状態永続化、グローバル状態管理を実現。

### モジュール構成

| モジュール | 内容 |
|-----------|------|
| `async/` | AsyncValue拡張、AsyncState、K1s0AsyncNotifier |
| `persistence/` | StateStorage、PreferencesStorage、HiveStorage、PersistedState |
| `global/` | AppState、UserPreferences、NavigationState、ConnectivityState |
| `utils/` | StateLogger、Debouncer、Throttler、StateSelector |
| `widgets/` | AsyncValueWidget、StateConsumer、StateScope |

### 使用例

```dart
// AsyncValue 拡張
final items = ref.watch(itemsProvider);
items.when2(
  data: (data) => ListView(...),
  loading: () => LoadingWidget(),
  error: (e, s) => ErrorWidget(e),
  refreshing: (data) => RefreshingWidget(data),
);

// グローバル状態
ref.read(appStateProvider.notifier).setDarkMode(true);
final isDark = ref.watch(isDarkModeProvider);

// 状態永続化
final storage = await PreferencesStorage.create();
ref.read(userPreferencesProvider.notifier).initialize(storage);

// デバウンス
final debouncer = Debouncer(duration: Duration(milliseconds: 300));
debouncer.run(() => search(query));

// 状態ログ
K1s0StateProvider(
  enableLogging: true,
  child: MyApp(),
)
```

---

## Frontend 依存関係

```
React:
@k1s0/shell
  └── @k1s0/ui

@k1s0/navigation
  └── @k1s0/config

@k1s0/api-client
  └── (standalone)

@k1s0/config
  └── (standalone)

@k1s0/ui
  └── (standalone, Material-UI依存)

@k1s0/auth-client
  └── (standalone, jose依存)

@k1s0/observability
  └── @opentelemetry/api (optional)

Flutter:
k1s0_config
  └── (standalone, yaml依存)

k1s0_http
  └── dio, k1s0_observability(optional)

k1s0_auth
  ├── flutter_secure_storage
  ├── jwt_decoder
  └── go_router(optional)

k1s0_observability
  └── (standalone)

k1s0_ui
  └── flutter_riverpod

k1s0_state
  ├── flutter_riverpod
  ├── shared_preferences
  └── hive_flutter
```
