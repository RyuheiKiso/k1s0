# k1s0-retry ライブラリ設計

## 概要

指数バックオフリトライ + サーキットブレーカーパターン実装ライブラリ。サービス間 gRPC/HTTP 呼び出しで利用する。`RetryConfig` と `CircuitBreaker` を組み合わせた `with_retry` 関数を提供する。OpenTelemetry メトリクス連携によりリトライ回数・サーキットブレーカー状態を計測する。

Rust 実装の `CircuitBreaker` は `k1s0-circuit-breaker` パッケージへ委譲する薄い互換ラッパーであり、閾値型や状態遷移ロジックは `k1s0-circuit-breaker` 側と統一される。

**配置先**: `regions/system/library/rust/retry/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `RetryConfig` | 構造体 | 最大試行回数・初期遅延・最大遅延・倍率・ジッター設定 |
| `CircuitBreaker` | 構造体 | 失敗閾値・オープン時間・ハーフオープン試行数設定 |
| `CircuitBreakerState` | enum | `Closed`（正常）/ `Open`（遮断中）/ `HalfOpen`（試行中） |
| `with_retry` | 関数 | RetryConfig に基づいて非同期クロージャをリトライ実行 |
| `RetryError<E>` | enum | `ExhaustedRetries { attempts, last_error }` / `CircuitBreakerOpen` / `Timeout` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-retry"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
rand = "0.8"
mockall = { version = "0.13", optional = true }
k1s0-circuit-breaker = { path = "../circuit-breaker" }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

`RetryError<E>` は generic error 型で、`E` に呼び出し元操作のエラー型を保持する。

**依存追加**: `k1s0-retry = { path = "../../system/library/rust/retry" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
retry/
├── src/
│   ├── lib.rs              # 公開 API（再エクスポート）
│   ├── policy.rs           # RetryConfig（指数バックオフ・ジッター設定）
│   ├── circuit_breaker.rs  # k1s0-circuit-breaker への互換ラッパー
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

**配置先**: `regions/system/library/go/retry/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

> サーキットブレーカーの実装は専用パッケージ `circuit-breaker` に統合済み。
> retry パッケージはリトライロジックのみを提供し、サーキットブレーカーが必要な場合は
> `github.com/k1s0-platform/system-library-go-circuit-breaker` を使用すること。

## TypeScript 実装

**配置先**: `regions/system/library/typescript/retry/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

// CircuitBreakerConfig・CircuitBreakerState・CircuitBreaker の定義は
// circuit-breaker.md を参照。retry ライブラリからは circuit-breaker を
// 依存として利用する。
// See: ../resilience/circuit-breaker.md
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/retry/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

// CircuitBreakerState・CircuitBreakerConfig・CircuitBreaker の定義は
// circuit-breaker.md を参照。retry ライブラリからは circuit-breaker を
// 依存として利用する。
// See: ../resilience/circuit-breaker.md
```

**カバレッジ目標**: 90%以上

## DefaultRetryConfig デフォルト値

各言語実装のデフォルト設定は以下で統一される（2026-03-04 時点の実装値）。

| 言語 | max_attempts | initial_delay | max_delay | multiplier | jitter |
| --- | --- | --- | --- | --- | --- |
| Go (`DefaultRetryConfig()`) | 3 | 100ms | 30s | 2.0 | true |
| Rust (`RetryConfig::default()`) | 3 | 100ms | 30s | 2.0 | true |
| TypeScript (`defaultRetryConfig`) | 3 | 100ms | 30s | 2.0 | true |
| Dart (`const RetryConfig()`) | 3 | 100ms | 30s | 2.0 | true |

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-serviceauth設計](../auth-security/serviceauth.md) — サービス間認証でリトライ活用
- [system-library-messaging設計](../messaging/messaging.md) — k1s0-messaging ライブラリ
- [gRPC設計](../../architecture/api/gRPC設計.md) — gRPC 通信設計
- [可観測性設計](../../architecture/observability/可観測性設計.md) — メトリクス・トレーシング設計
