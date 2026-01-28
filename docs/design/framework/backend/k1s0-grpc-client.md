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
