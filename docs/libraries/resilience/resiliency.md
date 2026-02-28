# k1s0-resiliency ライブラリ設計

## 概要

retry・circuit-breaker・bulkhead・timeout を統合した複合回復力パターンライブラリ。Resilience4j 相当の機能を単一の `ResiliencyPolicy` で一括設定し、`ResiliencyDecorator` でAny関数をラップして実行する。

個別ライブラリ（k1s0-retry、k1s0-circuit-breaker）を内部で組み合わせることでシンプルな API を実現する。OpenTelemetry メトリクス連携により回復力イベント（リトライ回数・サーキットブレーカー状態・バルクヘッド拒否数）を自動記録する。featureflag-server と連携してポリシーのホットリロードにも対応する。

**配置先**: `regions/system/library/rust/resiliency/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `ResiliencyPolicy` | 構造体 | retry・circuit-breaker・bulkhead・timeout の統合設定 |
| `RetryConfig` | 構造体 | 最大試行回数・指数バックオフ・リトライ対象エラー設定（正規定義: [retry.md](retry.md)） |
| `CircuitBreakerConfig` | 構造体 | 失敗閾値・復旧タイムアウト・HalfOpen 試行数（正規定義: [circuit-breaker.md](circuit-breaker.md)） |
| `BulkheadConfig` | 構造体 | 最大同時実行数・待機タイムアウト |
| `ResiliencyDecorator` | 構造体 | ポリシーを適用した関数実行器 |
| `ResiliencyMetrics` | 構造体 | OpenTelemetry メトリクス（回復力イベント全種別） |
| `ResiliencyError` | enum | `MaxRetriesExceeded`・`CircuitOpen`・`BulkheadFull`・`Timeout` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-resiliency"
version = "0.1.0"
edition = "2021"

[features]
metrics = ["opentelemetry"]
hot-reload = ["k1s0-featureflag"]

[dependencies]
async-trait = "0.1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
rand = "0.8"
opentelemetry = { version = "0.27", optional = true }
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
    .retry(RetryConfig {
        max_attempts: 3,
        backoff: ExponentialBackoff::new(Duration::from_millis(100), Duration::from_secs(5)),
        retryable_errors: vec!["network_error", "timeout"],
    })
    .circuit_breaker(CircuitBreakerConfig {
        failure_threshold: 5,
        recovery_timeout: Duration::from_secs(30),
        half_open_max_calls: 2,
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

// ホットリロード対応（featureflag-server と連携）
let reloadable_policy = ResiliencyPolicy::from_featureflag("payment-service-policy", &ff_client).await?;
```

## Go 実装

**配置先**: `regions/system/library/go/resiliency/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `go.opentelemetry.io/otel v1.34`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
// RetryConfig の正規定義は retry.md を参照。
// import: k1s0-retry パッケージの RetryConfig を使用する。

// CircuitBreakerConfig の正規定義は circuit-breaker.md を参照。
// import: k1s0-circuit-breaker パッケージの CircuitBreakerConfig を使用する。

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
// RetryConfig の正規定義は retry.md を参照。
// import { RetryConfig } from 'k1s0-retry';

// CircuitBreakerConfig の正規定義は circuit-breaker.md を参照。
// import { CircuitBreakerConfig } from 'k1s0-circuit-breaker';

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

export class ResiliencyDecorator {
  constructor(policy: ResiliencyPolicy);
  execute<T>(fn: () => Promise<T>): Promise<T>;
}

export function withResiliency<T>(
  policy: ResiliencyPolicy,
  fn: () => Promise<T>
): Promise<T>;

export class ResiliencyError extends Error {
  constructor(message: string, public readonly kind: 'retry_exceeded' | 'circuit_open' | 'bulkhead_full' | 'timeout');
}
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
// RetryConfig の正規定義は retry.md を参照。
// import 'package:k1s0_retry/retry.dart' show RetryConfig;

// CircuitBreakerConfig の正規定義は circuit-breaker.md を参照。
// import 'package:k1s0_circuit_breaker/circuit_breaker.dart' show CircuitBreakerConfig;

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

class ResiliencyDecorator {
  ResiliencyDecorator(ResiliencyPolicy policy);
  Future<T> execute<T>(Future<T> Function() fn);
}
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
