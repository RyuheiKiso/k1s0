# k1s0-retry ライブラリ設計

## 概要

指数バックオフリトライ + サーキットブレーカーパターン実装ライブラリ。サービス間 gRPC/HTTP 呼び出しで利用する。`RetryConfig` と `CircuitBreaker` を組み合わせた `with_retry` 関数を提供する。OpenTelemetry メトリクス連携によりリトライ回数・サーキットブレーカー状態を計測する。

**配置先**: `regions/system/library/rust/retry/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `RetryConfig` | 構造体 | 最大試行回数・初期遅延・最大遅延・倍率・ジッター設定 |
| `CircuitBreaker` | 構造体 | 失敗閾値・オープン時間・ハーフオープン試行数設定 |
| `CircuitBreakerState` | enum | `Closed`（正常）/ `Open`（遮断中）/ `HalfOpen`（試行中） |
| `with_retry` | 関数 | RetryConfig に基づいて非同期クロージャをリトライ実行 |
| `RetryError` | enum | 最大リトライ超過 |

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
│   ├── lib.rs              # 公開 API（再エクスポート）
│   ├── policy.rs           # RetryConfig（指数バックオフ・ジッター設定）
│   ├── circuit_breaker.rs  # CircuitBreaker・CircuitBreakerState
│   ├── retry.rs            # with_retry 関数
│   └── error.rs            # RetryError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_retry::{RetryConfig, with_retry};
use std::time::Duration;

// 指数バックオフリトライ
let config = RetryConfig::new(3)
    .with_initial_delay(Duration::from_millis(100))
    .with_max_delay(Duration::from_secs(5))
    .with_multiplier(2.0)
    .with_jitter(true);

let result = with_retry(&config, || async {
    grpc_client.call_service(request.clone()).await
}).await?;
```

## Go 実装

**配置先**: `regions/system/library/go/retry/`

```
retry/
├── retry.go
├── retry_test.go
├── go.mod
└── go.sum
```

**依存関係**: なし（標準ライブラリのみ）

**主要インターフェース**:

```go
type RetryConfig struct {
    MaxAttempts  int
    InitialDelay time.Duration
    MaxDelay     time.Duration
    Multiplier   float64
    Jitter       bool
}

func DefaultRetryConfig() *RetryConfig

func (c *RetryConfig) ComputeDelay(attempt int) time.Duration

func WithRetry[T any](ctx context.Context, config *RetryConfig, operation func(ctx context.Context) (T, error)) (T, error)

type RetryError struct {
    Attempts  int
    LastError error
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/retry/`

```
retry/
├── package.json        # "@k1s0/retry", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # RetryConfig, CircuitBreaker, CircuitBreakerState, withRetry, RetryError
└── __tests__/
    ├── retry.test.ts
    └── circuit-breaker.test.ts
```

**主要 API**:

```typescript
export interface RetryConfig {
  maxAttempts: number;
  initialDelayMs: number;
  maxDelayMs: number;
  multiplier: number;
  jitter: boolean;
}

export const defaultRetryConfig: RetryConfig;

export function computeDelay(config: RetryConfig, attempt: number): number;

export async function withRetry<T>(
  config: RetryConfig,
  operation: () => Promise<T>,
): Promise<T>;

export class RetryError extends Error {
  constructor(public readonly attempts: number, public readonly lastError: Error);
}

export interface CircuitBreakerConfig {
  failureThreshold: number;
  successThreshold: number;
  timeoutMs: number;
}

export const defaultCircuitBreakerConfig: CircuitBreakerConfig;

export type CircuitBreakerState = 'closed' | 'open' | 'half-open';

export class CircuitBreaker {
  constructor(config?: Partial<CircuitBreakerConfig>);
  getState(): CircuitBreakerState;
  isOpen(): boolean;
  recordSuccess(): void;
  recordFailure(): void;
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
│   ├── k1s0_retry.dart          # ライブラリエクスポート
│   ├── retry.dart
│   └── src/
│       ├── config.dart          # RetryConfig（指数バックオフ・ジッター設定）・computeDelay
│       ├── circuit_breaker.dart # CircuitBreaker・CircuitBreakerState・CircuitBreakerConfig
│       ├── retry.dart           # withRetry 関数
│       └── error.dart           # RetryError
└── test/
    ├── retry_test.dart
    └── circuit_breaker_test.dart
```

**主要 API**:

```dart
class RetryConfig {
  final int maxAttempts;
  final int initialDelayMs;
  final int maxDelayMs;
  final double multiplier;
  final bool jitter;
  const RetryConfig({...});
}

int computeDelay(RetryConfig config, int attempt);

Future<T> withRetry<T>(RetryConfig config, Future<T> Function() operation);

class RetryError implements Exception {
  final int attempts;
  final Object lastError;
}

enum CircuitBreakerState { closed, open, halfOpen }

class CircuitBreakerConfig {
  final int failureThreshold;
  final int successThreshold;
  final int timeoutMs;
}

class CircuitBreaker {
  CircuitBreaker({CircuitBreakerConfig? config});
  CircuitBreakerState get state;
  bool get isOpen;
  void recordSuccess();
  void recordFailure();
}
```

**カバレッジ目標**: 90%以上

## C# 実装

**配置先**: `regions/system/library/csharp/retry/`

```
retry/
├── RetryConfig.cs             # RetryConfig record（指数バックオフ・ジッター設定）
├── RetryPolicy.cs             # RetryPolicy static class（WithRetryAsync）
├── CircuitBreaker.cs          # CircuitBreaker・CircuitBreakerState・CircuitBreakerConfig
├── RetryError.cs              # RetryExhaustedException
├── K1s0.System.Retry.csproj
├── tests/
│   ├── K1s0.System.Retry.Tests.csproj
│   └── RetryPolicyTests.cs
└── .editorconfig
```

**名前空間**: `K1s0.System.Retry`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `RetryConfig` | record | 最大リトライ回数・初期遅延・最大遅延・倍率・ジッター設定 |
| `RetryPolicy` | static class | `WithRetryAsync` 静的メソッド |
| `CircuitBreaker` | class | 失敗閾値・成功閾値・タイムアウト・状態管理 |
| `CircuitBreakerState` | enum | Closed / Open / HalfOpen |
| `CircuitBreakerConfig` | class | FailureThreshold / SuccessThreshold / Timeout 設定 |
| `RetryExhaustedException` | class | リトライ超過例外 |

**主要 API**:

```csharp
namespace K1s0.System.Retry;

public record RetryConfig(
    int MaxAttempts = 3,
    TimeSpan? InitialDelay = null,
    TimeSpan? MaxDelay = null,
    double Multiplier = 2.0,
    bool Jitter = true)
{
    public TimeSpan ComputeDelay(int attempt);
}

public static class RetryPolicy
{
    public static Task<T> WithRetryAsync<T>(
        RetryConfig config, Func<Task<T>> operation);
    public static Task WithRetryAsync(
        RetryConfig config, Func<Task> operation);
}

public class RetryExhaustedException : Exception
{
    public int Attempts { get; }
    public Exception LastError { get; }
}

public enum CircuitBreakerState { Closed, Open, HalfOpen }

public class CircuitBreakerConfig
{
    public int FailureThreshold { get; init; }     // default: 5
    public int SuccessThreshold { get; init; }     // default: 2
    public TimeSpan Timeout { get; init; }         // default: 30s
}

public class CircuitBreaker
{
    public CircuitBreaker(CircuitBreakerConfig config);

    public CircuitBreakerState State { get; }

    public bool IsOpen();

    public Task RecordSuccessAsync();
    public Task RecordFailureAsync();
}
```

**カバレッジ目標**: 90%以上

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
│       ├── models.py             # RetryConfig dataclass
│       ├── memory.py             # CircuitBreaker, CircuitBreakerState
│       ├── client.py             # with_retry（非同期）
│       ├── exceptions.py         # RetryError
│       └── py.typed
└── tests/
    ├── test_policy.py
    ├── test_retry.py
    └── test_circuit_breaker.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `RetryConfig` | dataclass | 最大試行回数・初期遅延・最大遅延・倍率・ジッター設定 |
| `CircuitBreaker` | class | 失敗閾値・成功閾値・タイムアウト・状態管理 |
| `CircuitBreakerState` | Enum | CLOSED / OPEN / HALF_OPEN |
| `with_retry` | 関数 | RetryConfig による非同期リトライ実行 |
| `RetryError` | Exception | 最大リトライ超過エラー |

### 使用例

```python
import asyncio
from k1s0_retry import RetryConfig, CircuitBreaker, with_retry

# 指数バックオフリトライ
config = RetryConfig(
    max_attempts=3,
    initial_delay=0.1,
    max_delay=30.0,
    multiplier=2.0,
    jitter=True,
)

# 非同期リトライ
async def call_service():
    result = await with_retry(
        config,
        grpc_client.call_service,
    )
    return result

# サーキットブレーカー
cb = CircuitBreaker(
    failure_threshold=5,
    success_threshold=2,
    timeout=30.0,
)

# 状態確認・手動記録
print(f"State: {cb.state}")
cb.record_success()
cb.record_failure()
print(f"Is open: {cb.is_open()}")
```

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
