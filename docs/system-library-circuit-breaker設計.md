# k1s0-circuit-breaker ライブラリ設計

## 概要

サーキットブレーカーパターン実装ライブラリ。Closed/Open/HalfOpen の3状態管理と、OpenTelemetry メトリクス連携によるサーキットブレーカー状態の可視化を提供する。gRPC/HTTP のサービス間呼び出しで利用する。

失敗閾値を超えた呼び出し先を一定期間遮断し、HalfOpen 状態でプローブ呼び出しを行い自動復旧を判定する。スレッドセーフな状態遷移を `tokio::sync::Mutex` で保証し、全状態変化を OpenTelemetry メトリクスとして記録する。

**配置先**: `regions/system/library/rust/circuit-breaker/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `CircuitBreaker` | 構造体 | 失敗閾値・オープン時間・ハーフオープン試行数の設定と状態管理 |
| `CircuitBreakerState` | enum | `Closed`（正常）/ `Open`（遮断中）/ `HalfOpen`（試行中） |
| `CircuitBreakerConfig` | 構造体 | failure_threshold, success_threshold, open_duration 設定 |
| `CircuitBreakerMetrics` | 構造体 | OpenTelemetry メトリクス（状態変化・成功率） |
| `CircuitBreakerError` | enum | `CircuitOpen`・`ExecutionFailed` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-circuit-breaker"
version = "0.1.0"
edition = "2021"

[features]
metrics = ["opentelemetry"]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
opentelemetry = { version = "0.27", optional = true }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**Cargo.toml への追加行**:

```toml
k1s0-circuit-breaker = { path = "../../system/library/rust/circuit-breaker" }
# メトリクス連携を有効化する場合:
k1s0-circuit-breaker = { path = "../../system/library/rust/circuit-breaker", features = ["metrics"] }
```

**モジュール構成**:

```
circuit-breaker/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── breaker.rs      # CircuitBreaker・CircuitBreakerState
│   ├── config.rs       # CircuitBreakerConfig
│   ├── metrics.rs      # CircuitBreakerMetrics（OTel メトリクス）
│   └── error.rs        # CircuitBreakerError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use std::time::Duration;

// サーキットブレーカー設定（連続5回失敗で30秒遮断、HalfOpen で2回成功なら復旧）
let config = CircuitBreakerConfig::new()
    .failure_threshold(5)
    .success_threshold(2)
    .open_duration(Duration::from_secs(30))
    .half_open_max_calls(2);

let cb = CircuitBreaker::new(config);

// サーキットブレーカー経由でサービス呼び出し
let result = cb.execute(|| async {
    grpc_client.call_service(request.clone()).await
}).await?;

// 現在の状態を確認
let state = cb.state().await;
println!("CircuitBreaker state: {:?}", state);

// 手動リセット（管理操作用）
cb.reset().await;
```

## Go 実装

**配置先**: `regions/system/library/go/circuit-breaker/`

```
circuit-breaker/
├── circuit_breaker.go
├── config.go
├── state.go
├── metrics.go
├── circuit_breaker_test.go
├── go.mod
└── go.sum
```

**依存関係**: `go.opentelemetry.io/otel v1.34`

**主要インターフェース**:

```go
type CircuitBreakerState int

const (
    StateClosed   CircuitBreakerState = iota
    StateOpen
    StateHalfOpen
)

type CircuitBreakerConfig struct {
    FailureThreshold int
    SuccessThreshold int
    OpenDuration     time.Duration
    HalfOpenMaxCalls int
}

type CircuitBreaker struct {
    // unexported fields
}

func NewCircuitBreaker(config CircuitBreakerConfig) *CircuitBreaker

func (cb *CircuitBreaker) Execute(ctx context.Context, fn func() (any, error)) (any, error)

func (cb *CircuitBreaker) State() CircuitBreakerState

func (cb *CircuitBreaker) Reset()
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/circuit-breaker/`

```
circuit-breaker/
├── package.json        # "@k1s0/circuit-breaker", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState, CircuitBreakerError
└── __tests__/
    └── circuit-breaker.test.ts
```

**主要 API**:

```typescript
export type CircuitBreakerState = 'closed' | 'open' | 'half_open';

export interface CircuitBreakerConfig {
  failureThreshold: number;
  successThreshold: number;
  openDurationMs: number;
  halfOpenMaxCalls?: number;
}

export class CircuitBreaker {
  constructor(config: CircuitBreakerConfig);
  get state(): CircuitBreakerState;
  async execute<T>(fn: () => Promise<T>): Promise<T>;
  reset(): void;
}

export class CircuitOpenError extends Error {
  constructor(public readonly remainingMs: number);
}

export class ExecutionFailedError extends Error {
  constructor(message: string, public readonly cause?: Error);
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/circuit-breaker/`

```
circuit-breaker/
├── pubspec.yaml        # k1s0_circuit_breaker
├── analysis_options.yaml
├── lib/
│   ├── circuit_breaker.dart
│   └── src/
│       ├── breaker.dart      # CircuitBreaker
│       ├── config.dart       # CircuitBreakerConfig
│       ├── state.dart        # CircuitBreakerState enum
│       └── error.dart        # CircuitBreakerError
└── test/
    └── circuit_breaker_test.dart
```

**カバレッジ目標**: 90%以上

## C# 実装

**配置先**: `regions/system/library/csharp/circuit-breaker/`

```
circuit-breaker/
├── src/
│   ├── CircuitBreaker.csproj
│   ├── ICircuitBreaker.cs          # サーキットブレーカーインターフェース
│   ├── CircuitBreaker.cs           # 実装（スレッドセーフ）
│   ├── CircuitBreakerState.cs      # Closed / Open / HalfOpen enum
│   ├── CircuitBreakerConfig.cs     # 閾値・タイムアウト設定
│   ├── CircuitBreakerMetrics.cs    # OpenTelemetry メトリクス連携
│   └── CircuitBreakerException.cs  # 公開例外型
├── tests/
│   ├── CircuitBreaker.Tests.csproj
│   ├── Unit/
│   │   ├── CircuitBreakerTests.cs
│   │   └── CircuitBreakerConfigTests.cs
│   └── Integration/
│       └── CircuitBreakerIntegrationTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| OpenTelemetry | メトリクス連携 |

**名前空間**: `K1s0.System.CircuitBreaker`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `ICircuitBreaker` | interface | サーキットブレーカー操作の抽象インターフェース |
| `CircuitBreaker` | class | スレッドセーフな3状態管理実装 |
| `CircuitBreakerState` | enum | Closed / Open / HalfOpen |
| `CircuitBreakerConfig` | record | 失敗閾値・オープン時間・HalfOpen 試行数設定 |
| `CircuitBreakerMetrics` | class | OpenTelemetry メトリクス記録 |
| `CircuitBreakerException` | class | CircuitOpen・ExecutionFailed の例外型 |

**主要 API**:

```csharp
namespace K1s0.System.CircuitBreaker;

public enum CircuitBreakerState { Closed, Open, HalfOpen }

public record CircuitBreakerConfig(
    int FailureThreshold,
    int SuccessThreshold,
    TimeSpan OpenDuration,
    int HalfOpenMaxCalls = 1);

public interface ICircuitBreaker
{
    CircuitBreakerState State { get; }

    Task<T> ExecuteAsync<T>(
        Func<CancellationToken, Task<T>> fn,
        CancellationToken ct = default);

    void Reset();
}

public class CircuitBreaker : ICircuitBreaker
{
    public CircuitBreaker(CircuitBreakerConfig config);

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
- ターゲット: `K1s0CircuitBreaker`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開API

```swift
public enum CircuitBreakerState: Sendable {
    case closed
    case open(until: ContinuousClock.Instant)
    case halfOpen
}

public struct CircuitBreakerConfig: Sendable {
    public let failureThreshold: Int
    public let successThreshold: Int
    public let openDuration: Duration
    public let halfOpenMaxCalls: Int

    public init(
        failureThreshold: Int = 5,
        successThreshold: Int = 2,
        openDuration: Duration = .seconds(30),
        halfOpenMaxCalls: Int = 2
    )
}

public actor CircuitBreaker {
    public init(config: CircuitBreakerConfig)

    public var state: CircuitBreakerState { get }

    public func execute<T: Sendable>(
        _ operation: @Sendable () async throws -> T
    ) async throws -> T

    public func reset()
}
```

### エラー型

```swift
public enum CircuitBreakerError: Error, Sendable {
    case circuitOpen(remainingDuration: Duration)
    case executionFailed(underlying: Error)
}
```

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上

---

## Python 実装

**配置先**: `regions/system/library/python/circuit-breaker/`

### パッケージ構造

```
circuit-breaker/
├── pyproject.toml
├── src/
│   └── k1s0_circuit_breaker/
│       ├── __init__.py       # 公開 API（再エクスポート）
│       ├── breaker.py        # CircuitBreaker
│       ├── config.py         # CircuitBreakerConfig dataclass
│       ├── state.py          # CircuitBreakerState Enum
│       ├── exceptions.py     # CircuitBreakerError
│       └── py.typed
└── tests/
    ├── test_breaker.py
    └── test_state_transition.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `CircuitBreaker` | class | スレッドセーフな3状態管理実装 |
| `CircuitBreakerState` | Enum | CLOSED / OPEN / HALF_OPEN |
| `CircuitBreakerConfig` | dataclass | 失敗閾値・オープン時間・HalfOpen 試行数設定 |
| `CircuitBreakerError` | Exception | CircuitOpen・ExecutionFailed エラー基底クラス |

### 使用例

```python
import asyncio
from k1s0_circuit_breaker import CircuitBreaker, CircuitBreakerConfig

# サーキットブレーカー設定
config = CircuitBreakerConfig(
    failure_threshold=5,
    success_threshold=2,
    open_duration=30.0,
    half_open_max_calls=2,
)
cb = CircuitBreaker(config)

# サーキットブレーカー経由で非同期呼び出し
async def call_service():
    result = await cb.execute(grpc_client.call_service, request)
    return result

# 現在の状態を確認
print(f"State: {cb.state}")

# 手動リセット
cb.reset()
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

## テスト戦略

| テスト種別 | 対象 | ツール |
|-----------|------|--------|
| ユニットテスト（`#[cfg(test)]`） | 状態遷移ロジック（Closed→Open→HalfOpen→Closed）・閾値カウント・タイムアウト計算 | tokio::test |
| モックテスト | `mockall` による実行関数モック・失敗注入テスト | mockall (feature = "mock") |
| 並行性テスト | 複数タスクからの同時実行・状態レースコンディション検証 | tokio::test（複数スポーン） |
| プロパティテスト | ランダムな失敗シーケンスでの状態整合性検証 | proptest |

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-retry設計](system-library-retry設計.md) — リトライと組み合わせた利用パターン
- [可観測性設計](可観測性設計.md) — OpenTelemetry メトリクス設計
- [gRPC設計](gRPC設計.md) — サービス間呼び出し設計
