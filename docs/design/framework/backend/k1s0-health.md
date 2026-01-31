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

## Go 版（k1s0-health）

### 主要な型

```go
// HealthStatus はヘルスチェックのステータス。
type HealthStatus int

const (
    Healthy   HealthStatus = iota
    Degraded
    Unhealthy
)

func (s HealthStatus) ToHTTPStatusCode() int
func (s HealthStatus) IsServing() bool

// ComponentHealth はコンポーネント単位のヘルス。
type ComponentHealth struct {
    Name      string
    Status    HealthStatus
    LatencyMs *uint64
    Error     string
}

func NewHealthy(name string) *ComponentHealth
func NewDegraded(name, err string) *ComponentHealth
func NewUnhealthy(name, err string) *ComponentHealth

// HealthResponse はヘルスレスポンス。
type HealthResponse struct {
    Status     HealthStatus
    Service    string
    Version    string
    Components []*ComponentHealth
}

// ProbeHandler は K8s プローブハンドラー。
type ProbeHandler struct{}

func NewProbeHandler(serviceName string) *ProbeHandler
func (h *ProbeHandler) Liveness() *HealthResponse
func (h *ProbeHandler) Readiness() *HealthResponse
func (h *ProbeHandler) Startup() *HealthResponse
```

### 使用例

```go
import k1s0health "github.com/k1s0/framework/backend/go/k1s0-health"

handler := k1s0health.NewProbeHandler("my-service")
resp := handler.Readiness()
```

## C# 版（K1s0.Health）

### 主要な型

```csharp
public enum HealthStatus { Healthy, Degraded, Unhealthy }

public record ComponentHealth(string Name, HealthStatus Status,
    ulong? LatencyMs = null, string? Error = null)
{
    public static ComponentHealth Healthy(string name);
    public static ComponentHealth Degraded(string name, string error);
    public static ComponentHealth Unhealthy(string name, string error);
}

public class HealthResponse
{
    public HealthStatus Status { get; }
    public string Service { get; }
    public string? Version { get; set; }
    public List<ComponentHealth> Components { get; }
}

public class ProbeHandler
{
    public ProbeHandler(string serviceName);
    public HealthResponse Liveness();
    public HealthResponse Readiness();
    public HealthResponse Startup();
}
```

### 使用例

```csharp
using K1s0.Health;

var handler = new ProbeHandler("my-service");
var response = handler.Readiness();
```

## Python 版（k1s0-health）

FastAPI ベースのヘルスチェック機能。

### 主要な型

```python
from enum import Enum

class HealthStatus(Enum):
    HEALTHY = "healthy"
    DEGRADED = "degraded"
    UNHEALTHY = "unhealthy"

    def to_http_status_code(self) -> int: ...
    def is_serving(self) -> bool: ...

@dataclass
class ComponentHealth:
    name: str
    status: HealthStatus
    latency_ms: int | None = None
    error: str | None = None

    @classmethod
    def healthy(cls, name: str) -> "ComponentHealth": ...
    @classmethod
    def degraded(cls, name: str, error: str) -> "ComponentHealth": ...
    @classmethod
    def unhealthy(cls, name: str, error: str) -> "ComponentHealth": ...

@dataclass
class HealthResponse:
    status: HealthStatus
    service: str
    version: str | None = None
    components: list[ComponentHealth] = field(default_factory=list)

class ProbeHandler:
    def __init__(self, service_name: str) -> None: ...
    def liveness(self) -> HealthResponse: ...
    def readiness(self) -> HealthResponse: ...
    def startup(self) -> HealthResponse: ...
```

### 使用例

```python
from k1s0_health import ProbeHandler

handler = ProbeHandler("my-service")
response = handler.readiness()
```

## Kotlin 版（k1s0-health）

Ktor ベースのヘルスチェック機能。

### 主要な型

```kotlin
enum class HealthStatus {
    Healthy, Degraded, Unhealthy;

    fun toHttpStatusCode(): Int
    fun isServing(): Boolean
}

data class ComponentHealth(
    val name: String,
    val status: HealthStatus,
    val latencyMs: Long? = null,
    val error: String? = null
) {
    companion object {
        fun healthy(name: String): ComponentHealth
        fun degraded(name: String, error: String): ComponentHealth
        fun unhealthy(name: String, error: String): ComponentHealth
    }
}

data class HealthResponse(
    val status: HealthStatus,
    val service: String,
    val version: String? = null,
    val components: List<ComponentHealth> = emptyList()
)

class ProbeHandler(private val serviceName: String) {
    fun liveness(): HealthResponse
    fun readiness(): HealthResponse
    fun startup(): HealthResponse
}
```

### 使用例

```kotlin
import com.k1s0.health.*

val handler = ProbeHandler("my-service")
val response = handler.readiness()
```
