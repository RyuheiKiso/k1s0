# k1s0-grpc-server

## 目的

gRPC サーバの共通基盤を提供する。OTel/ログ/メトリクスの共通エントリ、トレースコンテキスト伝播、error_code/status 統一を実現。

## 設計原則

1. **共通インターセプタ**: "最低限の礼儀"をテンプレで自動有効
2. **error_code 必須**: エラー時は必ず error_code を付与
3. **デッドライン検知**: クライアントがデッドラインを指定していない場合の検知
4. **構造化ログ**: JSON 形式で必須フィールドを統一

## デッドラインポリシー

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

## 主要な型

### GrpcServerConfig

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

### RequestContext

```rust
pub struct RequestContext {
    pub trace_id: String,
    pub request_id: String,
    pub tenant_id: Option<String>,
    pub deadline: Option<Instant>,
}
```

### ResponseMetadata

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

### RequestLog

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

## 使用例

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
