# k1s0-retry ライブラリ設計

## 概要

指数バックオフリトライ + サーキットブレーカーパターン実装ライブラリ。サービス間 gRPC/HTTP 呼び出しで利用する。`RetryPolicy` と `CircuitBreaker` を組み合わせた `with_retry` / `with_circuit_breaker` 関数を提供する。OpenTelemetry メトリクス連携によりリトライ回数・サーキットブレーカー状態を計測する。

**配置先**: `regions/system/library/rust/retry/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `RetryPolicy` | 構造体 | 最大リトライ回数・基底遅延・最大遅延・ジッター設定 |
| `CircuitBreaker` | 構造体 | 失敗閾値・オープン時間・ハーフオープン試行数設定 |
| `CircuitBreakerState` | enum | `Closed`（正常）/ `Open`（遮断中）/ `HalfOpen`（試行中） |
| `RetryableError` | トレイト | リトライ可能エラーの判定インターフェース |
| `with_retry` | 関数 | RetryPolicy に基づいて非同期クロージャをリトライ実行 |
| `with_circuit_breaker` | 関数 | CircuitBreaker 状態チェック + 実行 |
| `RetryMetrics` | 構造体 | OpenTelemetry メトリクス（リトライ回数・失敗率） |
| `RetryError` | enum | 最大リトライ超過・サーキットブレーカーオープン |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-retry"
version = "0.1.0"
edition = "2021"

[features]
metrics = ["opentelemetry"]

[dependencies]
async-trait = "0.1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
rand = "0.8"
opentelemetry = { version = "0.27", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**Cargo.toml への追加行**:

```toml
k1s0-retry = { path = "../../system/library/rust/retry" }
# メトリクス連携を有効化する場合:
k1s0-retry = { path = "../../system/library/rust/retry", features = ["metrics"] }
```

**モジュール構成**:

```
retry/
├── src/
│   ├── lib.rs              # 公開 API・使用例ドキュメント
│   ├── policy.rs           # RetryPolicy（指数バックオフ・ジッター設定）
│   ├── circuit_breaker.rs  # CircuitBreaker・CircuitBreakerState
│   ├── retry.rs            # with_retry・with_circuit_breaker 関数
│   ├── metrics.rs          # RetryMetrics（OTel メトリクス）
│   └── error.rs            # RetryError・RetryableError トレイト
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_retry::{RetryPolicy, CircuitBreaker, with_retry, with_circuit_breaker};
use std::time::Duration;

// 指数バックオフリトライ
let policy = RetryPolicy::new()
    .max_attempts(3)
    .base_delay(Duration::from_millis(100))
    .max_delay(Duration::from_secs(5))
    .with_jitter();

let result = with_retry(policy, || async {
    grpc_client.call_service(request.clone()).await
}).await?;

// サーキットブレーカー（連続5回失敗で30秒遮断）
let cb = CircuitBreaker::new()
    .failure_threshold(5)
    .open_duration(Duration::from_secs(30))
    .half_open_max_calls(2);

let result = with_circuit_breaker(&cb, || async {
    http_client.post("/api/v1/orders").send().await
}).await?;
```

## Go 実装

**配置先**: `regions/system/library/go/retry/`

```
retry/
├── retry.go
├── policy.go
├── circuit_breaker.go
├── metrics.go
├── retry_test.go
├── go.mod
└── go.sum
```

**依存関係**: `go.opentelemetry.io/otel v1.34`

**主要インターフェース**:

```go
type RetryableError interface {
    IsRetryable() bool
}

func WithRetry[T any](ctx context.Context, policy RetryPolicy, fn func() (T, error)) (T, error)
func WithCircuitBreaker[T any](ctx context.Context, cb *CircuitBreaker, fn func() (T, error)) (T, error)

type RetryPolicy struct {
    MaxAttempts int
    BaseDelay   time.Duration
    MaxDelay    time.Duration
    Jitter      bool
}

type CircuitBreaker struct {
    FailureThreshold int
    OpenDuration     time.Duration
    HalfOpenMaxCalls int
}

type CircuitBreakerState int

const (
    StateClosed   CircuitBreakerState = iota
    StateOpen
    StateHalfOpen
)
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/retry/`

```
retry/
├── package.json        # "@k1s0/retry", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # RetryPolicy, CircuitBreaker, CircuitBreakerState, withRetry, RetryError
└── __tests__/
    ├── retry.test.ts
    └── circuit-breaker.test.ts
```

**主要 API**:

```typescript
export interface RetryPolicy {
  maxAttempts: number;
  baseDelayMs: number;
  maxDelayMs: number;
  jitter?: boolean;
}

export interface CircuitBreakerConfig {
  failureThreshold: number;
  openDurationMs: number;
  halfOpenMaxCalls?: number;
}

export type CircuitBreakerState = 'closed' | 'open' | 'half_open';

export async function withRetry<T>(
  policy: RetryPolicy,
  fn: () => Promise<T>,
  isRetryable?: (error: unknown) => boolean
): Promise<T>;

export class CircuitBreaker {
  constructor(config: CircuitBreakerConfig);
  get state(): CircuitBreakerState;
  async execute<T>(fn: () => Promise<T>): Promise<T>;
  reset(): void;
}

export class RetryError extends Error {
  constructor(message: string, public readonly attempts: number, public readonly lastError?: Error);
}

export class CircuitBreakerOpenError extends Error {
  constructor(public readonly remainingMs: number);
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/retry/`

```
retry/
├── pubspec.yaml        # k1s0_retry
├── analysis_options.yaml
├── lib/
│   ├── retry.dart
│   └── src/
│       ├── policy.dart          # RetryPolicy（指数バックオフ・ジッター設定）
│       ├── circuit_breaker.dart # CircuitBreaker・CircuitBreakerState
│       ├── retry.dart           # withRetry・withCircuitBreaker 関数
│       ├── metrics.dart         # RetryMetrics
│       └── error.dart           # RetryError・CircuitBreakerOpenError
└── test/
    ├── retry_test.dart
    └── circuit_breaker_test.dart
```

**カバレッジ目標**: 90%以上

## C# 実装

**配置先**: `regions/system/library/csharp/retry/`

```
retry/
├── src/
│   ├── Retry.csproj
│   ├── RetryPolicy.cs             # リトライポリシー設定
│   ├── CircuitBreaker.cs          # サーキットブレーカー実装
│   ├── CircuitBreakerState.cs     # Closed / Open / HalfOpen enum
│   ├── RetryExtensions.cs         # WithRetryAsync 拡張メソッド
│   ├── RetryMetrics.cs            # OpenTelemetry メトリクス連携
│   └── RetryException.cs          # 公開例外型
├── tests/
│   ├── Retry.Tests.csproj
│   ├── Unit/
│   │   ├── RetryPolicyTests.cs
│   │   └── CircuitBreakerTests.cs
│   └── Integration/
│       └── RetryIntegrationTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| OpenTelemetry | メトリクス連携 |

**名前空間**: `K1s0.System.Retry`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `RetryPolicy` | record | 最大リトライ回数・基底遅延・最大遅延・ジッター設定 |
| `CircuitBreaker` | class | 失敗閾値・オープン時間・状態管理 |
| `CircuitBreakerState` | enum | Closed / Open / HalfOpen |
| `RetryExtensions` | static class | `WithRetryAsync` 拡張メソッド |
| `RetryMetrics` | class | OpenTelemetry メトリクス記録 |
| `RetryException` | class | リトライ超過・サーキットブレーカーオープンの例外型 |

**主要 API**:

```csharp
namespace K1s0.System.Retry;

public record RetryPolicy(
    int MaxAttempts,
    TimeSpan BaseDelay,
    TimeSpan MaxDelay,
    bool Jitter = true);

public static class RetryExtensions
{
    public static Task<T> WithRetryAsync<T>(
        this RetryPolicy policy,
        Func<CancellationToken, Task<T>> fn,
        Func<Exception, bool>? isRetryable = null,
        CancellationToken ct = default);
}

public enum CircuitBreakerState { Closed, Open, HalfOpen }

public class CircuitBreaker
{
    public CircuitBreaker(
        int failureThreshold,
        TimeSpan openDuration,
        int halfOpenMaxCalls = 1);

    public CircuitBreakerState State { get; }

    public Task<T> ExecuteAsync<T>(
        Func<CancellationToken, Task<T>> fn,
        CancellationToken ct = default);

    public void Reset();
}
```

**カバレッジ目標**: 90%以上

---

## Swift

### パッケージ構成
- ターゲット: `K1s0Retry`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開API

```swift
public struct RetryPolicy: Sendable {
    public let maxAttempts: Int
    public let baseDelay: Duration
    public let maxDelay: Duration
    public let jitter: Bool

    public init(
        maxAttempts: Int = 3,
        baseDelay: Duration = .milliseconds(100),
        maxDelay: Duration = .seconds(5),
        jitter: Bool = true
    )
}

public func withRetry<T: Sendable>(
    policy: RetryPolicy,
    operation: @Sendable () async throws -> T
) async throws -> T

public actor CircuitBreaker {
    public enum State: Sendable { case closed, open, halfOpen }

    public init(
        failureThreshold: Int = 5,
        openDuration: Duration = .seconds(30),
        halfOpenMaxCalls: Int = 2
    )

    public var state: State { get }

    public func execute<T: Sendable>(
        _ operation: @Sendable () async throws -> T
    ) async throws -> T

    public func reset()
}
```

### エラー型

```swift
public enum RetryError: Error, Sendable {
    case maxAttemptsExceeded(attempts: Int, lastError: Error)
    case circuitBreakerOpen(remainingDuration: Duration)
}
```

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上

---

## Python 実装

**配置先**: `regions/system/library/python/retry/`

### パッケージ構造

```
retry/
├── pyproject.toml
├── src/
│   └── k1s0_retry/
│       ├── __init__.py           # 公開 API（再エクスポート）
│       ├── policy.py             # RetryPolicy dataclass
│       ├── circuit_breaker.py    # CircuitBreaker, CircuitBreakerState
│       ├── retry.py              # with_retry（同期/非同期対応）
│       ├── metrics.py            # RetryMetrics（OTel 連携）
│       ├── exceptions.py         # RetryError, CircuitBreakerOpenError
│       └── py.typed
└── tests/
    ├── test_policy.py
    ├── test_retry.py
    └── test_circuit_breaker.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `RetryPolicy` | dataclass | 最大試行回数・基底遅延・最大遅延・ジッター設定 |
| `CircuitBreaker` | class | 失敗閾値・オープン時間・状態管理 |
| `CircuitBreakerState` | Enum | CLOSED / OPEN / HALF_OPEN |
| `with_retry` | 関数 | RetryPolicy によるリトライ実行（同期/非同期対応） |
| `RetryError` | Exception | 最大リトライ超過エラー |
| `CircuitBreakerOpenError` | Exception | サーキットブレーカーオープン時エラー |

### 使用例

```python
import asyncio
from k1s0_retry import RetryPolicy, CircuitBreaker, with_retry

# 指数バックオフリトライ
policy = RetryPolicy(
    max_attempts=3,
    base_delay=0.1,
    max_delay=5.0,
    jitter=True,
)

# 非同期リトライ
async def call_service():
    result = await with_retry(
        policy,
        grpc_client.call_service,
    )
    return result

# サーキットブレーカー
cb = CircuitBreaker(
    failure_threshold=5,
    open_duration=30.0,
    half_open_max_calls=2,
)

async def call_with_cb():
    result = await cb.execute(http_client.post, "/api/v1/orders")
    return result
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| opentelemetry-api | >=1.29 | メトリクス連携 |

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock
- カバレッジ目標: 90%以上
- 実行: `pytest` / `ruff check .`

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-serviceauth設計](system-library-serviceauth設計.md) — サービス間認証でリトライ活用
- [system-library-messaging設計](system-library-messaging設計.md) — k1s0-messaging ライブラリ
- [gRPC設計](gRPC設計.md) — gRPC 通信設計
- [可観測性設計](可観測性設計.md) — メトリクス・トレーシング設計
