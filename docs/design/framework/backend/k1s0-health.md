# k1s0-health

## 目的

Kubernetes 対応のヘルスチェック機能を提供する。readiness/liveness/startup プローブをサポート。

## 設計原則

1. **3段階ステータス**: Healthy / Degraded / Unhealthy
2. **コンポーネント単位**: 各コンポーネント（DB、キャッシュ等）の個別ステータス
3. **K8s プローブ対応**: readiness/liveness/startup
4. **Graceful shutdown**: readiness 状態の動的切り替え

## 主要な型

### HealthStatus

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

### ComponentHealth

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

### HealthResponse

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

### ProbeHandler

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

### ReadinessState

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

## 使用例

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
