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

## Go 版（k1s0-grpc-server）

### 主要な型

```go
// GrpcServerConfig は gRPC サーバ設定。
type GrpcServerConfig struct {
    ServiceName string
    Env         string
    Port        int
    TLS         *TLSConfig
}

func NewGrpcServerConfigBuilder() *GrpcServerConfigBuilder

// RequestContext はリクエストスコープのコンテキスト。
type RequestContext struct {
    TraceID   string
    RequestID string
    TenantID  string
    Deadline  *time.Time
}

// ResponseMetadata はレスポンスメタデータ。
type ResponseMetadata struct {
    TraceID   string
    RequestID string
    ErrorCode string
}

func NewResponseMetadata(ctx *RequestContext) *ResponseMetadata
func (m *ResponseMetadata) WithErrorCode(code string) *ResponseMetadata

// DeadlinePolicy はデッドラインポリシー。
type DeadlinePolicy int

const (
    DeadlinePolicyAllow  DeadlinePolicy = iota
    DeadlinePolicyWarn
    DeadlinePolicyReject
)
```

### 使用例

```go
import k1s0grpc "github.com/k1s0/framework/backend/go/k1s0-grpc-server"

config := k1s0grpc.NewGrpcServerConfigBuilder().
    ServiceName("my-service").
    Env("dev").
    Port(50051).
    Build()

ctx := &k1s0grpc.RequestContext{TraceID: "abc", RequestID: "req-001"}
resp := k1s0grpc.NewResponseMetadata(ctx).WithErrorCode("USER_NOT_FOUND")
```

## C# 版（K1s0.Grpc.Server）

### 主要な型

```csharp
public class GrpcServerConfig
{
    public string ServiceName { get; }
    public string Env { get; }
    public int Port { get; }
    public static GrpcServerConfigBuilder Builder();
}

public record RequestContext(
    string TraceId, string RequestId,
    string? TenantId = null, DateTime? Deadline = null);

public record ResponseMetadata(
    string? TraceId = null, string? RequestId = null, string? ErrorCode = null)
{
    public static ResponseMetadata FromContext(RequestContext ctx);
    public ResponseMetadata WithErrorCode(string code);
}

public enum DeadlinePolicy { Allow, Warn, Reject }
```

### 使用例

```csharp
using K1s0.Grpc.Server;

var config = GrpcServerConfig.Builder()
    .ServiceName("my-service")
    .Env("dev")
    .Port(50051)
    .Build();

var ctx = new RequestContext("trace-abc", "req-001");
var resp = ResponseMetadata.FromContext(ctx).WithErrorCode("USER_NOT_FOUND");
```

## Python 版（k1s0-grpc-server）

grpcio ベースの gRPC サーバ基盤。

### 主要な型

```python
@dataclass
class GrpcServerConfig:
    service_name: str
    env: str
    port: int = 50051

@dataclass
class RequestContext:
    trace_id: str
    request_id: str
    tenant_id: str | None = None
    deadline: float | None = None

@dataclass
class ResponseMetadata:
    trace_id: str | None = None
    request_id: str | None = None
    error_code: str | None = None

    @classmethod
    def from_context(cls, ctx: RequestContext) -> "ResponseMetadata": ...
    def with_error_code(self, code: str) -> "ResponseMetadata": ...
```

### 使用例

```python
from k1s0_grpc_server import GrpcServerConfig, RequestContext, ResponseMetadata

config = GrpcServerConfig(service_name="my-service", env="dev", port=50051)
ctx = RequestContext(trace_id="abc", request_id="req-001")
resp = ResponseMetadata.from_context(ctx).with_error_code("USER_NOT_FOUND")
```

## Kotlin 版（k1s0-grpc-server）

grpc-kotlin ベースの gRPC サーバ基盤。

### 主要な型

```kotlin
data class GrpcServerConfig(
    val serviceName: String,
    val env: String,
    val port: Int = 50051
) {
    class Builder {
        fun serviceName(name: String): Builder
        fun env(env: String): Builder
        fun port(port: Int): Builder
        fun build(): GrpcServerConfig
    }
}

data class RequestContext(
    val traceId: String,
    val requestId: String,
    val tenantId: String? = null,
    val deadline: Instant? = null
)

data class ResponseMetadata(
    val traceId: String? = null,
    val requestId: String? = null,
    val errorCode: String? = null
) {
    companion object {
        fun fromContext(ctx: RequestContext): ResponseMetadata
    }
    fun withErrorCode(code: String): ResponseMetadata
}

enum class DeadlinePolicy { Allow, Warn, Reject }
```

### 使用例

```kotlin
import com.k1s0.grpc.server.*

val config = GrpcServerConfig.Builder()
    .serviceName("my-service")
    .env("dev")
    .port(50051)
    .build()

val ctx = RequestContext(traceId = "abc", requestId = "req-001")
val resp = ResponseMetadata.fromContext(ctx).withErrorCode("USER_NOT_FOUND")
```
