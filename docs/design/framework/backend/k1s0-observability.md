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

## Go 版（k1s0-observability）

### 主要な型

```go
// ObservabilityConfig は観測性の初期化設定。
type ObservabilityConfig struct {
    ServiceName string
    Env         string
    Version     string
}

func NewObservabilityConfig(serviceName, env string) *ObservabilityConfig

// RequestContext はリクエストスコープのコンテキスト。
type RequestContext struct {
    TraceID   string
    RequestID string
    TenantID  string
}

func NewRequestContext() *RequestContext

// LogEntry は構造化ログエントリ。
type LogEntry struct {
    Level       string
    Message     string
    Timestamp   string
    ServiceName string
    Env         string
    TraceID     string
    RequestID   string
    Fields      map[string]interface{}
}

func Info(message string) *LogEntry
func Warn(message string) *LogEntry
func Error(message string) *LogEntry
func (e *LogEntry) WithContext(ctx *RequestContext) *LogEntry
func (e *LogEntry) ToJSON() (string, error)
```

### 使用例

```go
import k1s0obs "github.com/k1s0/framework/backend/go/k1s0-observability"

config := k1s0obs.NewObservabilityConfig("user-service", "dev")
ctx := k1s0obs.NewRequestContext()

entry := k1s0obs.Info("ユーザーを作成しました").WithContext(ctx)
fmt.Println(entry.ToJSON())
```

## C# 版（K1s0.Observability）

### 主要な型

```csharp
public class ObservabilityConfig
{
    public string ServiceName { get; }
    public string Env { get; }
    public string? Version { get; set; }

    public static ObservabilityConfigBuilder Builder();
    public RequestContext NewRequestContext();
}

public class RequestContext
{
    public string TraceId { get; }
    public string RequestId { get; }
    public string? TenantId { get; set; }
}

public class LogEntry
{
    public static LogEntry Info(string message);
    public static LogEntry Warn(string message);
    public static LogEntry Error(string message);
    public LogEntry WithContext(RequestContext ctx);
    public LogEntry WithService(ObservabilityConfig config);
    public string ToJson();
}
```

### 使用例

```csharp
using K1s0.Observability;

var config = ObservabilityConfig.Builder()
    .ServiceName("user-service")
    .Env("dev")
    .Build();

var ctx = config.NewRequestContext();
var entry = LogEntry.Info("ユーザーを作成しました")
    .WithContext(ctx)
    .WithService(config);

Console.WriteLine(entry.ToJson());
```

## Python 版（k1s0-observability）

OpenTelemetry ベースの観測性ライブラリ。

### 主要な型

```python
@dataclass
class ObservabilityConfig:
    service_name: str
    env: str
    version: str | None = None

    def new_request_context(self) -> "RequestContext": ...

@dataclass
class RequestContext:
    trace_id: str
    request_id: str
    tenant_id: str | None = None

class LogEntry:
    @classmethod
    def info(cls, message: str) -> "LogEntry": ...
    @classmethod
    def warn(cls, message: str) -> "LogEntry": ...
    @classmethod
    def error(cls, message: str) -> "LogEntry": ...
    def with_context(self, ctx: RequestContext) -> "LogEntry": ...
    def with_service(self, config: ObservabilityConfig) -> "LogEntry": ...
    def to_json(self) -> str: ...
```

### 使用例

```python
from k1s0_observability import ObservabilityConfig, LogEntry

config = ObservabilityConfig(service_name="user-service", env="dev")
ctx = config.new_request_context()

entry = LogEntry.info("ユーザーを作成しました").with_context(ctx).with_service(config)
print(entry.to_json())
```

## Kotlin 版（k1s0-observability）

OpenTelemetry ベースの観測性ライブラリ。

### 主要な型

```kotlin
data class ObservabilityConfig(
    val serviceName: String,
    val env: String,
    val version: String? = null
) {
    fun newRequestContext(): RequestContext

    class Builder {
        fun serviceName(name: String): Builder
        fun env(env: String): Builder
        fun build(): ObservabilityConfig
    }
}

data class RequestContext(
    val traceId: String,
    val requestId: String,
    val tenantId: String? = null
)

class LogEntry private constructor(val level: String, val message: String) {
    companion object {
        fun info(message: String): LogEntry
        fun warn(message: String): LogEntry
        fun error(message: String): LogEntry
    }
    fun withContext(ctx: RequestContext): LogEntry
    fun withService(config: ObservabilityConfig): LogEntry
    fun toJson(): String
}
```

### 使用例

```kotlin
import com.k1s0.observability.*

val config = ObservabilityConfig.Builder()
    .serviceName("user-service")
    .env("dev")
    .build()

val ctx = config.newRequestContext()
val entry = LogEntry.info("ユーザーを作成しました")
    .withContext(ctx)
    .withService(config)

println(entry.toJson())
```
