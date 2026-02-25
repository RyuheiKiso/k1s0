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
| `CircuitBreakerConfig` | 構造体 | failure_threshold, success_threshold, timeout 設定 |
| `CircuitBreakerMetrics` | 構造体 | OpenTelemetry メトリクス（状態変化・成功率） |
| `CircuitBreakerError` | enum | `Open`・`Inner` |

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
│   ├── config.rs       # CircuitBreakerConfig（failure_threshold, success_threshold, timeout）
│   ├── metrics.rs      # CircuitBreakerMetrics（OTel メトリクス）
│   └── error.rs        # CircuitBreakerError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use std::time::Duration;

// サーキットブレーカー設定（連続5回失敗で30秒遮断、HalfOpen で3回成功なら復旧）
let config = CircuitBreakerConfig {
    failure_threshold: 5,
    success_threshold: 3,
    timeout: Duration::from_secs(30),
};

let cb = CircuitBreaker::new(config);

// サーキットブレーカー経由でサービス呼び出し
let result = cb.call(|| async {
    grpc_client.call_service(request.clone()).await
}).await?;

// 現在の状態を確認
let state = cb.state().await;
println!("CircuitBreaker state: {:?}", state);

// メトリクス取得
let metrics = cb.metrics().await;
println!("failures: {}, successes: {}", metrics.failure_count, metrics.success_count);
```

## Go 実装

**配置先**: `regions/system/library/go/circuit-breaker/`

```
circuit-breaker/
├── circuitbreaker.go
├── circuitbreaker_test.go
├── go.mod
└── go.sum
```

**依存関係**: なし（標準ライブラリのみ）

**主要インターフェース**:

```go
type State int

const (
    StateClosed   State = iota
    StateOpen
    StateHalfOpen
)

type Config struct {
    FailureThreshold uint32
    SuccessThreshold uint32
    Timeout          time.Duration
}

type CircuitBreaker struct {
    // unexported fields
}

func New(cfg Config) *CircuitBreaker

func (cb *CircuitBreaker) Call(fn func() error) error

func (cb *CircuitBreaker) State() State

func (cb *CircuitBreaker) IsOpen() bool

func (cb *CircuitBreaker) RecordSuccess()

func (cb *CircuitBreaker) RecordFailure()
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
export type CircuitState = 'closed' | 'open' | 'half-open';

export interface CircuitBreakerConfig {
  failureThreshold: number;
  successThreshold: number;
  timeoutMs: number;
}

export class CircuitBreakerError extends Error {
  constructor();
}

export class CircuitBreaker {
  constructor(config: CircuitBreakerConfig);
  get state(): CircuitState;
  isOpen(): boolean;
  recordSuccess(): void;
  recordFailure(): void;
  async call<T>(fn: () => Promise<T>): Promise<T>;
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
├── CircuitBreaker.cs           # 実装（スレッドセーフ）
├── ICircuitBreaker.cs          # CircuitState enum・CircuitBreakerConfig・CircuitBreakerOpenException
├── K1s0.System.CircuitBreaker.csproj
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
| `CircuitBreaker` | class | スレッドセーフな3状態管理実装 |
| `CircuitState` | enum | Closed / Open / HalfOpen |
| `CircuitBreakerConfig` | record | 失敗閾値・成功閾値・タイムアウト設定 |
| `CircuitBreakerOpenException` | class | サーキットブレーカーオープン時の例外型 |

**主要 API**:

```csharp
namespace K1s0.System.CircuitBreaker;

public enum CircuitState { Closed, Open, HalfOpen }

public record CircuitBreakerConfig(int FailureThreshold, int SuccessThreshold, TimeSpan Timeout);

public class CircuitBreakerOpenException : Exception { }

public class CircuitBreaker
{
    public CircuitBreaker(CircuitBreakerConfig config);

    public CircuitState State { get; }

    public bool IsOpen { get; }

    public void RecordSuccess();

    public void RecordFailure();

    public Task<T> CallAsync<T>(Func<Task<T>> fn, CancellationToken ct = default);
}
```

**カバレッジ目標**: 90%以上

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
