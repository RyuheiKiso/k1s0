# k1s0-resilience

## 目的

依存先呼び出しのガードレールを提供する。タイムアウト、同時実行制限、バルクヘッド、サーキットブレーカをサポート。

## 設計原則

1. **タイムアウト必須**: 無制限待機を防ぐ
2. **同時実行制限**: リソース枯渇を防ぐ
3. **障害隔離**: バルクヘッドで障害の波及を防ぐ
4. **サーキットブレーカ**: 必要時のみ有効化（既定OFF）

## タイムアウト

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

## 同時実行制限

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

## バルクヘッド

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

## サーキットブレーカ

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

## ResilienceError

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

## 使用例

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

## Go 版（k1s0-resilience）

### 主要な型

```go
// TimeoutGuard はタイムアウトガード。
type TimeoutGuard struct {
    TimeoutMs uint64
}

func NewTimeoutGuard(timeoutMs uint64) (*TimeoutGuard, error)
func (g *TimeoutGuard) Execute(ctx context.Context, fn func(context.Context) error) error

// ConcurrencyLimiter は同時実行制限。
type ConcurrencyLimiter struct{}

func NewConcurrencyLimiter(maxConcurrent int) *ConcurrencyLimiter
func (l *ConcurrencyLimiter) Execute(ctx context.Context, fn func(context.Context) error) error

// Bulkhead はバルクヘッド。
type BulkheadConfig struct {
    DefaultLimit  int
    ServiceLimits map[string]int
}

type Bulkhead struct{}

func NewBulkhead(config BulkheadConfig) *Bulkhead
func (b *Bulkhead) Execute(ctx context.Context, service string, fn func(context.Context) error) error

// CircuitBreaker はサーキットブレーカ。
type CircuitState int

const (
    CircuitClosed   CircuitState = iota
    CircuitOpen
    CircuitHalfOpen
)

type CircuitBreakerConfig struct {
    Enabled          bool
    FailureThreshold uint32
    SuccessThreshold uint32
    ResetTimeoutSecs uint64
}

type CircuitBreaker struct{}

func NewCircuitBreaker(config CircuitBreakerConfig) *CircuitBreaker
func (cb *CircuitBreaker) Execute(ctx context.Context, fn func(context.Context) error) error
func (cb *CircuitBreaker) State() CircuitState
```

### 使用例

```go
import k1s0res "github.com/k1s0/framework/backend/go/k1s0-resilience"

guard, _ := k1s0res.NewTimeoutGuard(5000)
err := guard.Execute(ctx, func(ctx context.Context) error {
    return doWork(ctx)
})

cb := k1s0res.NewCircuitBreaker(k1s0res.CircuitBreakerConfig{
    Enabled: true, FailureThreshold: 5, SuccessThreshold: 3, ResetTimeoutSecs: 30,
})
err = cb.Execute(ctx, func(ctx context.Context) error { return callService(ctx) })
```

## C# 版（K1s0.Resilience）

### 主要な型

```csharp
public class TimeoutGuard
{
    public TimeoutGuard(ulong timeoutMs);
    public async Task<T> ExecuteAsync<T>(Func<CancellationToken, Task<T>> func);
}

public class ConcurrencyLimiter
{
    public ConcurrencyLimiter(int maxConcurrent);
    public async Task<T> ExecuteAsync<T>(Func<Task<T>> func);
}

public class BulkheadConfig
{
    public int DefaultLimit { get; set; }
    public Dictionary<string, int> ServiceLimits { get; set; }
}

public class Bulkhead
{
    public Bulkhead(BulkheadConfig config);
    public async Task<T> ExecuteAsync<T>(string service, Func<Task<T>> func);
}

public enum CircuitState { Closed, Open, HalfOpen }

public class CircuitBreakerConfig
{
    public bool Enabled { get; set; }
    public uint FailureThreshold { get; set; }
    public uint SuccessThreshold { get; set; }
    public ulong ResetTimeoutSecs { get; set; }
}

public class CircuitBreaker
{
    public CircuitBreaker(CircuitBreakerConfig config);
    public async Task<T> ExecuteAsync<T>(Func<Task<T>> func);
    public CircuitState State { get; }
}
```

### 使用例

```csharp
using K1s0.Resilience;

var guard = new TimeoutGuard(5000);
var result = await guard.ExecuteAsync(async ct => await DoWork(ct));

var cb = new CircuitBreaker(new CircuitBreakerConfig
{
    Enabled = true, FailureThreshold = 5, SuccessThreshold = 3, ResetTimeoutSecs = 30
});
var result = await cb.ExecuteAsync(async () => await CallService());
```

## Python 版（k1s0-resilience）

### 主要な型

```python
class TimeoutGuard:
    def __init__(self, timeout_ms: int) -> None: ...
    async def execute(self, coro: Coroutine[Any, Any, T]) -> T: ...

class ConcurrencyLimiter:
    def __init__(self, max_concurrent: int) -> None: ...
    async def execute(self, coro: Coroutine[Any, Any, T]) -> T: ...

@dataclass
class BulkheadConfig:
    default_limit: int
    service_limits: dict[str, int] = field(default_factory=dict)

class Bulkhead:
    def __init__(self, config: BulkheadConfig) -> None: ...
    async def execute(self, service: str, coro: Coroutine[Any, Any, T]) -> T: ...

class CircuitState(Enum):
    CLOSED = "closed"
    OPEN = "open"
    HALF_OPEN = "half_open"

@dataclass
class CircuitBreakerConfig:
    enabled: bool = False
    failure_threshold: int = 5
    success_threshold: int = 3
    reset_timeout_secs: int = 30

class CircuitBreaker:
    def __init__(self, config: CircuitBreakerConfig) -> None: ...
    async def execute(self, coro: Coroutine[Any, Any, T]) -> T: ...
    @property
    def state(self) -> CircuitState: ...
```

### 使用例

```python
from k1s0_resilience import TimeoutGuard, CircuitBreaker, CircuitBreakerConfig

guard = TimeoutGuard(timeout_ms=5000)
result = await guard.execute(do_work())

cb = CircuitBreaker(CircuitBreakerConfig(
    enabled=True, failure_threshold=5, success_threshold=3, reset_timeout_secs=30
))
result = await cb.execute(call_service())
```

## Kotlin 版（k1s0-resilience）

### 主要な型

```kotlin
class TimeoutGuard(private val timeoutMs: Long) {
    suspend fun <T> execute(block: suspend () -> T): T
}

class ConcurrencyLimiter(private val maxConcurrent: Int) {
    suspend fun <T> execute(block: suspend () -> T): T
}

data class BulkheadConfig(
    val defaultLimit: Int,
    val serviceLimits: Map<String, Int> = emptyMap()
)

class Bulkhead(private val config: BulkheadConfig) {
    suspend fun <T> execute(service: String, block: suspend () -> T): T
}

enum class CircuitState { Closed, Open, HalfOpen }

data class CircuitBreakerConfig(
    val enabled: Boolean = false,
    val failureThreshold: Int = 5,
    val successThreshold: Int = 3,
    val resetTimeoutSecs: Long = 30
)

class CircuitBreaker(private val config: CircuitBreakerConfig) {
    suspend fun <T> execute(block: suspend () -> T): T
    val state: CircuitState
}
```

### 使用例

```kotlin
import com.k1s0.resilience.*

val guard = TimeoutGuard(timeoutMs = 5000)
val result = guard.execute { doWork() }

val cb = CircuitBreaker(CircuitBreakerConfig(
    enabled = true, failureThreshold = 5, successThreshold = 3, resetTimeoutSecs = 30
))
val result = cb.execute { callService() }
```
