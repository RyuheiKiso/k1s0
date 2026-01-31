# k1s0-grpc-client

## 目的

gRPC クライアント呼び出しの共通基盤を提供する。deadline 必須、retry 原則 0、サービスディスカバリをサポート。

## 設計原則

1. **deadline 必須**: 無制限呼び出しを防ぐ（100ms〜5分）
2. **retry 原則 0**: リトライは明示的な opt-in（ADR 参照必須）
3. **トレース伝播**: W3C Trace Context の自動付与
4. **サービスディスカバリ**: K8s DNS 形式での論理名解決

## 主要な型

### GrpcClientConfig

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

### GrpcClientBuilder

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

### ServiceDiscoveryConfig

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

### RetryConfig

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

### CallOptions

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

## 使用例

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

## Go 版（k1s0-grpc-client）

### 主要な型

```go
// GrpcClientConfig はクライアント設定。
type GrpcClientConfig struct {
    TimeoutMs        uint64 // デフォルト: 30000
    ConnectTimeoutMs uint64 // デフォルト: 5000
    Retry            RetryConfig
}

// GrpcClientBuilder はクライアント構築用ビルダー。
type GrpcClientBuilder struct{}

func NewGrpcClientBuilder(serviceName string) *GrpcClientBuilder
func (b *GrpcClientBuilder) TargetAddress(address string) *GrpcClientBuilder
func (b *GrpcClientBuilder) TargetService(service string) *GrpcClientBuilder
func (b *GrpcClientBuilder) Config(config GrpcClientConfig) *GrpcClientBuilder
func (b *GrpcClientBuilder) Discovery(config ServiceDiscoveryConfig) *GrpcClientBuilder
func (b *GrpcClientBuilder) Build() (*GrpcClientConnection, error)

// RetryConfig はリトライ設定。
type RetryConfig struct {
    Enabled      bool
    ADRReference string
    MaxAttempts  uint32
}

// CallOptions は呼び出しオプション。
type CallOptions struct {
    TimeoutMs uint64
    TraceID   string
    RequestID string
    TenantID  string
}
```

### 使用例

```go
import k1s0grpc "github.com/k1s0/framework/backend/go/k1s0-grpc-client"

conn, err := k1s0grpc.NewGrpcClientBuilder("my-service").
    TargetAddress("localhost:50051").
    Build()

opts := k1s0grpc.CallOptions{
    TimeoutMs: 5000,
    TraceID:   "abc123",
    RequestID: "req-001",
}
```

## C# 版（K1s0.Grpc.Client）

### 主要な型

```csharp
public class GrpcClientConfig
{
    public ulong TimeoutMs { get; set; } = 30000;
    public ulong ConnectTimeoutMs { get; set; } = 5000;
    public RetryConfig Retry { get; set; } = RetryConfig.Disabled();
}

public class GrpcClientBuilder
{
    public GrpcClientBuilder(string serviceName);
    public GrpcClientBuilder TargetAddress(string address);
    public GrpcClientBuilder TargetService(string service);
    public GrpcClientBuilder WithConfig(GrpcClientConfig config);
    public GrpcClientConnection Build();
}

public record RetryConfig(bool Enabled, string? AdrReference = null, uint MaxAttempts = 0)
{
    public static RetryConfig Disabled();
}

public record CallOptions(
    ulong? TimeoutMs = null, string? TraceId = null,
    string? RequestId = null, string? TenantId = null);
```

### 使用例

```csharp
using K1s0.Grpc.Client;

var conn = new GrpcClientBuilder("my-service")
    .TargetAddress("localhost:50051")
    .Build();

var options = new CallOptions(TimeoutMs: 5000, TraceId: "abc123", RequestId: "req-001");
```

## Python 版（k1s0-grpc-client）

### 主要な型

```python
@dataclass
class GrpcClientConfig:
    timeout_ms: int = 30000
    connect_timeout_ms: int = 5000
    retry: "RetryConfig" = field(default_factory=RetryConfig.disabled)

@dataclass
class RetryConfig:
    enabled: bool = False
    adr_reference: str | None = None
    max_attempts: int = 0

    @classmethod
    def disabled(cls) -> "RetryConfig": ...

class GrpcClientBuilder:
    def __init__(self, service_name: str) -> None: ...
    def target_address(self, address: str) -> "GrpcClientBuilder": ...
    def target_service(self, service: str) -> "GrpcClientBuilder": ...
    def config(self, config: GrpcClientConfig) -> "GrpcClientBuilder": ...
    def build(self) -> "GrpcClientConnection": ...

@dataclass
class CallOptions:
    timeout_ms: int | None = None
    trace_id: str | None = None
    request_id: str | None = None
    tenant_id: str | None = None
```

### 使用例

```python
from k1s0_grpc_client import GrpcClientBuilder, CallOptions

conn = GrpcClientBuilder("my-service").target_address("localhost:50051").build()
options = CallOptions(timeout_ms=5000, trace_id="abc123", request_id="req-001")
```

## Kotlin 版（k1s0-grpc-client）

### 主要な型

```kotlin
data class GrpcClientConfig(
    val timeoutMs: Long = 30000,
    val connectTimeoutMs: Long = 5000,
    val retry: RetryConfig = RetryConfig.disabled()
)

data class RetryConfig(
    val enabled: Boolean = false,
    val adrReference: String? = null,
    val maxAttempts: Int = 0
) {
    companion object {
        fun disabled(): RetryConfig
    }
}

class GrpcClientBuilder(private val serviceName: String) {
    fun targetAddress(address: String): GrpcClientBuilder
    fun targetService(service: String): GrpcClientBuilder
    fun config(config: GrpcClientConfig): GrpcClientBuilder
    fun discovery(config: ServiceDiscoveryConfig): GrpcClientBuilder
    fun build(): GrpcClientConnection
}

data class CallOptions(
    val timeoutMs: Long? = null,
    val traceId: String? = null,
    val requestId: String? = null,
    val tenantId: String? = null
)
```

### 使用例

```kotlin
import com.k1s0.grpc.client.*

val conn = GrpcClientBuilder("my-service")
    .targetAddress("localhost:50051")
    .build()

val options = CallOptions(timeoutMs = 5000, traceId = "abc123", requestId = "req-001")
```
