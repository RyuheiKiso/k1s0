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

**依存追加**: `k1s0-circuit-breaker = { path = "../../system/library/rust/circuit-breaker" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

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

**配置先**: `regions/system/library/go/circuit-breaker/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

**配置先**: `regions/system/library/typescript/circuit-breaker/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

**配置先**: `regions/system/library/dart/circuit-breaker/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**カバレッジ目標**: 90%以上

## テスト戦略

| テスト種別 | 対象 | ツール |
|-----------|------|--------|
| ユニットテスト（`#[cfg(test)]`） | 状態遷移ロジック（Closed→Open→HalfOpen→Closed）・閾値カウント・タイムアウト計算 | tokio::test |
| モックテスト | `mockall` による実行関数モック・失敗注入テスト | mockall (feature = "mock") |
| 並行性テスト | 複数タスクからの同時実行・状態レースコンディション検証 | tokio::test（複数スポーン） |
| プロパティテスト | ランダムな失敗シーケンスでの状態整合性検証 | proptest |

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-retry設計](retry.md) — リトライと組み合わせた利用パターン
- [可観測性設計](../../architecture/observability/可観測性設計.md) — OpenTelemetry メトリクス設計
- [gRPC設計](../../architecture/api/gRPC設計.md) — サービス間呼び出し設計
