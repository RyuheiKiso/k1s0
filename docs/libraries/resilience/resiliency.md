# k1s0-resiliency ライブラリ設計

## 概要

retry・circuit-breaker・bulkhead・timeout を統合した複合回復力パターンライブラリ。Resilience4j 相当の機能を単一の `ResiliencyPolicy` で一括設定し、`ResiliencyDecorator` でAny関数をラップして実行する。

個別ライブラリ（k1s0-retry、k1s0-circuit-breaker）を内部で組み合わせることでシンプルな API を実現する。OpenTelemetry メトリクス連携により回復力イベント（リトライ回数・サーキットブレーカー状態・バルクヘッド拒否数）を自動記録する。featureflag-server と連携してポリシーのホットリロードにも対応する。

**配置先**: `regions/system/library/rust/resiliency/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `ResiliencyPolicy` | 構造体 | retry・circuit-breaker・bulkhead・timeout の統合設定 |
| `ResiliencyPolicyBuilder` | 構造体 | `retry` / `circuit_breaker` / `bulkhead` / `timeout` / `backoff` / `retryable_errors` を段階設定 |
| `RetryConfig` | 構造体 | 最大試行回数・指数バックオフ・リトライ対象エラー設定（正規定義: [retry.md](retry.md)） |
| `CircuitBreakerConfig` | 構造体 | 失敗閾値 (`failure_threshold`)・タイムアウト (`timeout`)・成功閾値 (`success_threshold`)（正規定義: [circuit-breaker.md](circuit-breaker.md)） |
| `BulkheadConfig` | 構造体 | 最大同時実行数・待機タイムアウト |
| `ExponentialBackoff` | 構造体 | リトライ遅延の指数バックオフ計算 |
| `Bulkhead` | 構造体 | セマフォベースの同時実行制御（Rust・Dart でスタンドアロンエクスポート） |
| `ResiliencyDecorator` | 構造体 | ポリシーを適用した関数実行器。`new()`, `with_metrics()`, `metrics()`, `execute()` |
| `ResiliencyMetrics` | 構造体 | OpenTelemetry メトリクス（回復力イベント全種別） |
| `ResiliencyError` | enum | `MaxRetriesExceeded`・`CircuitOpen`・`BulkheadFull`・`Timeout` |
| `ResiliencyPolicy.backoff` / `ResiliencyPolicy.retryable_errors` | フィールド | バックオフ戦略と再試行対象エラー文字列を保持 |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-resiliency"
version = "0.1.0"
edition = "2021"

[features]
metrics = ["opentelemetry"]
hot-reload = ["k1s0-featureflag", "serde", "serde_json"]

[dependencies]
async-trait = "0.1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time", "macros"] }
tracing = "0.1"
opentelemetry = { version = "0.27", optional = true }
serde = { version = "1", features = ["derive"], optional = true }
serde_json = { version = "1", optional = true }
k1s0-retry = { path = "../retry" }
k1s0-circuit-breaker = { path = "../circuit-breaker" }
k1s0-featureflag = { path = "../featureflag", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `k1s0-resiliency = { path = "../../system/library/rust/resiliency" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
resiliency/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── policy.rs       # ResiliencyPolicy・RetryConfig・CircuitBreakerConfig・BulkheadConfig
│   ├── decorator.rs    # ResiliencyDecorator（統合実行器）
│   ├── bulkhead.rs     # Bulkhead（セマフォベース同時実行制御）
│   ├── metrics.rs      # ResiliencyMetrics（OTel メトリクス）
│   └── error.rs        # ResiliencyError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_resiliency::{ResiliencyPolicy, RetryConfig, CircuitBreakerConfig, BulkheadConfig};
use std::time::Duration;

// 統合ポリシーを定義
let policy = ResiliencyPolicy::builder()
    .retry(RetryConfig::new(3).with_jitter(false))
    .backoff(ExponentialBackoff::new(Duration::from_millis(100), Duration::from_secs(5)))
    .retryable_errors(["network_error", "timeout"])
    .circuit_breaker(CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 2,
        timeout: Duration::from_secs(30),
    })
    .bulkhead(BulkheadConfig {
        max_concurrent_calls: 20,
        max_wait_duration: Duration::from_millis(500),
    })
    .timeout(Duration::from_secs(10))
    .build();

// デコレーターでラップして実行
let decorator = policy.decorate();

let result = decorator.execute(|| async {
    grpc_client.call_service(request.clone()).await
}).await?;

// メトリクス付きデコレーター
let metrics = Arc::new(ResiliencyMetrics::new());
let decorator = ResiliencyDecorator::with_metrics(policy, metrics.clone());
let stats = decorator.metrics(); // Arc<ResiliencyMetrics> を取得

// ホットリロード対応（hot-reload feature 有効時のみ）
// featureflag の variant value に JSON 形式でポリシーを格納:
// {"retry":{"max_attempts":3},"circuit_breaker":{"failure_threshold":5,"timeout_ms":30000},"timeout_ms":10000}
let policy = ResiliencyPolicy::from_featureflag("payment-service-policy", &ff_client).await?;
```

## Go 実装

**配置先**: `regions/system/library/go/resiliency/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `go.opentelemetry.io/otel v1.34`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
// Sentinel errors
var (
    ErrMaxRetriesExceeded = errors.New("max retries exceeded")
    ErrCircuitBreakerOpen = errors.New("circuit breaker is open")
    ErrBulkheadFull       = errors.New("bulkhead full")
    ErrTimeout            = errors.New("operation timed out")
)

type ResiliencyError struct {
    Kind    string
    Message string
    Cause   error
}
func (e *ResiliencyError) Error() string
func (e *ResiliencyError) Unwrap() error

type RetryConfig struct {
    MaxAttempts int
    BaseDelay   time.Duration
    MaxDelay    time.Duration
    Jitter      bool
}

type CircuitBreakerConfig struct {
    FailureThreshold int
    RecoveryTimeout  time.Duration
    HalfOpenMaxCalls int
}

type BulkheadConfig struct {
    MaxConcurrentCalls int
    MaxWaitDuration    time.Duration
}

type ResiliencyPolicy struct {
    Retry          *RetryConfig
    CircuitBreaker *CircuitBreakerConfig
    Bulkhead       *BulkheadConfig
    Timeout        time.Duration
}

type ResiliencyDecorator struct {
    // unexported fields
}

func NewResiliencyDecorator(policy ResiliencyPolicy) *ResiliencyDecorator

func Execute[T any](ctx context.Context, d *ResiliencyDecorator, fn func() (T, error)) (T, error)
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/resiliency/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export interface RetryConfig {
  maxAttempts: number;
  baseDelayMs: number;
  maxDelayMs: number;
  jitter?: boolean;
}

export interface CircuitBreakerConfig {
  failureThreshold: number;
  recoveryTimeoutMs: number;
  halfOpenMaxCalls?: number;
}

export interface BulkheadConfig {
  maxConcurrentCalls: number;
  maxWaitDurationMs: number;
}

export interface ResiliencyPolicy {
  retry?: RetryConfig;
  circuitBreaker?: CircuitBreakerConfig;
  bulkhead?: BulkheadConfig;
  timeoutMs?: number;
}

export type ResiliencyErrorKind =
  | 'retry_exceeded'
  | 'circuit_open'
  | 'bulkhead_full'
  | 'timeout';

export class ResiliencyError extends Error {
  constructor(
    message: string,
    public readonly kind: ResiliencyErrorKind,
    public readonly cause?: Error,
  );
}

export class ResiliencyDecorator {
  constructor(policy: ResiliencyPolicy);
  execute<T>(fn: () => Promise<T>): Promise<T>;
}

// 便利関数: デコレーターを内部で生成して即実行
export function withResiliency<T>(
  policy: ResiliencyPolicy,
  fn: () => Promise<T>,
): Promise<T>;
```

**カバレッジ目標**: 85%以上

## Dart 実装

**配置先**: `regions/system/library/dart/resiliency/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  meta: ^1.14.0
```

**主要インターフェース**:

```dart
class RetryConfig {
  final int maxAttempts;
  final Duration baseDelay;
  final Duration maxDelay;
  final bool jitter;
}

class CircuitBreakerConfig {
  final int failureThreshold;
  final Duration recoveryTimeout;
  final int halfOpenMaxCalls;
}

class BulkheadConfig {
  final int maxConcurrentCalls;
  final Duration maxWaitDuration;
}

class ResiliencyPolicy {
  final RetryConfig? retry;
  final CircuitBreakerConfig? circuitBreaker;
  final BulkheadConfig? bulkhead;
  final Duration? timeout;
}

// エラー型（sealed class 階層）
sealed class ResiliencyError {
  final String message;
}
class MaxRetriesExceededError extends ResiliencyError { /* attempts, lastError */ }
class CircuitBreakerOpenError extends ResiliencyError { /* remainingDuration */ }
class BulkheadFullError extends ResiliencyError { /* maxConcurrent */ }
class TimeoutError extends ResiliencyError { /* after */ }

// スタンドアロン Bulkhead
class Bulkhead {
  Bulkhead(int maxConcurrent, Duration maxWait);
  Future<void> acquire();
  void release();
}

class ResiliencyDecorator {
  ResiliencyDecorator(ResiliencyPolicy policy);
  Future<T> execute<T>(Future<T> Function() fn);
}

// 便利関数: デコレーターを内部で生成して即実行
Future<T> withResiliency<T>(
  ResiliencyPolicy policy,
  Future<T> Function() fn,
);
```

**カバレッジ目標**: 85%以上

## テスト戦略

| テスト種別 | 対象 | ツール |
|-----------|------|--------|
| ユニットテスト（`#[cfg(test)]`） | 各ポリシー要素の単独動作・組み合わせ動作の検証 | tokio::test |
| バルクヘッドテスト | 最大同時実行数の上限制御・待機タイムアウト発火確認 | tokio::test（複数スポーン） |
| 統合テスト | retry→circuit-breaker の連携・全要素統合シナリオ | tokio::test |
| カオステスト | ランダム失敗注入によるポリシー組み合わせの安定性検証 | proptest |
| ホットリロードテスト | featureflag 変更イベントによるポリシー動的更新の検証 | モック featureflag-server |

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-retry設計](retry.md) — k1s0-retry ライブラリ（内部依存）
- [system-library-circuit-breaker設計](circuit-breaker.md) — k1s0-circuit-breaker ライブラリ（内部依存）
- [可観測性設計](../../architecture/observability/可観測性設計.md) — OpenTelemetry メトリクス設計
- [gRPC設計](../../architecture/api/gRPC設計.md) — サービス間呼び出し設計
