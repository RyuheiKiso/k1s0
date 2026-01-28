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
