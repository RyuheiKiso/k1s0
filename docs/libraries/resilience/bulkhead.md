# k1s0-bulkhead ライブラリ設計

## 概要

バルクヘッドパターン実装ライブラリ。最大同時実行数を制限し、待機タイムアウトで溢れたリクエストを拒否する。gRPC/HTTP のサービス間呼び出しで利用する。

セマフォベースの同時実行制御により、特定のサービス呼び出しがリソースを独占することを防ぎ、障害の波及範囲を限定する。許可枠が全て使用中の場合、設定した待機時間を超えるとリクエストを即座に拒否し、呼び出し元のスレッド/タスクがブロックされ続けることを防ぐ。

**配置先**: `regions/system/library/rust/bulkhead/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `BulkheadConfig` | 構造体 | max_concurrent_calls（デフォルト: 20）・max_wait_duration（デフォルト: 500ms）の設定 |
| `Bulkhead` | 構造体 | セマフォベースの同時実行制御。`new()`, `acquire()`, `release()`, `call()`, `metrics()` |
| `BulkheadError` / `BulkheadFullError` | エラー | バルクヘッドが満杯で待機タイムアウト時に返されるエラー |
| `BulkheadMetrics` | 構造体（Rust のみ） | OpenTelemetry メトリクス（拒否数・現在の同時実行数） |
| `Bulkhead::acquire()` | メソッド | 許可枠を取得。満杯の場合は max_wait_duration まで待機し、超過で BulkheadFullError を返す |
| `Bulkhead::release()` | メソッド | 許可枠を解放し、待機中のリクエストに枠を譲る |
| `Bulkhead::call()` | メソッド | acquire → fn 実行 → release を一括で行うヘルパー |
| `Bulkhead::metrics()` | メソッド（Rust のみ） | 現在の rejected_count / active_count を返す |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-bulkhead"
version = "0.1.0"
edition = "2021"

[features]
metrics = ["opentelemetry"]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time", "macros"] }
opentelemetry = { version = "0.27", optional = true }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

`metrics` feature はオプトインで有効化する。無効時は OpenTelemetry 依存なしで利用できる。

**依存追加**: `k1s0-bulkhead = { path = "../../system/library/rust/bulkhead" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
bulkhead/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── bulkhead.rs     # Bulkhead（tokio::sync::Semaphore + tokio::time::timeout）
│   ├── config.rs       # BulkheadConfig（max_concurrent_calls, max_wait_duration）
│   ├── metrics.rs      # BulkheadMetrics（OTel メトリクス）
│   └── error.rs        # BulkheadError / BulkheadFullError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_bulkhead::{Bulkhead, BulkheadConfig};
use std::time::Duration;

// バルクヘッド設定（最大20同時実行、500ms待機タイムアウト）
let config = BulkheadConfig {
    max_concurrent_calls: 20,
    max_wait_duration: Duration::from_millis(500),
};

let bh = Bulkhead::new(config);

// バルクヘッド経由でサービス呼び出し
let result = bh.call(|| async {
    grpc_client.call_service(request.clone()).await
}).await?;

// メトリクス取得
let metrics = bh.metrics().await;
println!("rejected: {}, active: {}", metrics.rejected_count, metrics.active_count);
```

## Go 実装

**配置先**: `regions/system/library/go/bulkhead/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: なし（標準ライブラリのみ）

**主要インターフェース**:

```go
type Config struct {
    MaxConcurrentCalls int
    MaxWaitDuration    time.Duration
}

type Bulkhead struct {
    // unexported fields
}

func New(cfg Config) *Bulkhead

func (b *Bulkhead) Acquire(ctx context.Context) error

func (b *Bulkhead) Release()

func (b *Bulkhead) Call(ctx context.Context, fn func() error) error

// センチネルエラー: バルクヘッドが満杯で待機タイムアウト時に返される
var ErrFull = errors.New("bulkhead is full")
```

> Go 実装は `BulkheadMetrics` 型を提供しない（N/A）。メトリクス連携は呼び出し側で状態変化を観測して実装する。

## TypeScript 実装

**配置先**: `regions/system/library/typescript/bulkhead/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export interface BulkheadConfig {
  maxConcurrentCalls: number;
  maxWaitDurationMs: number;
}

export class BulkheadFullError extends Error {
  constructor();
}

export class Bulkhead {
  constructor(config: BulkheadConfig);
  async acquire(): Promise<void>;
  release(): void;
  async call<T>(fn: () => Promise<T>): Promise<T>;
}
```

**カバレッジ目標**: 90%以上

> TypeScript 実装は `BulkheadMetrics` 型を提供しない（N/A）。メトリクス連携は呼び出し側で状態変化を観測して実装する。

## Dart 実装

**配置先**: `regions/system/library/dart/bulkhead/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```dart
class BulkheadConfig {
  final int maxConcurrentCalls;
  final Duration maxWaitDuration;

  const BulkheadConfig({
    required this.maxConcurrentCalls,
    required this.maxWaitDuration,
  });
}

class BulkheadFullException implements Exception {
  const BulkheadFullException();
}

class Bulkhead {
  Bulkhead(BulkheadConfig config);

  Future<void> acquire();
  void release();
  Future<T> call<T>(Future<T> Function() fn);
}
```

**カバレッジ目標**: 90%以上

> Dart 実装は `BulkheadMetrics` 型を提供しない（N/A）。メトリクス連携は呼び出し側で状態変化を観測して実装する。

## テスト戦略

| テスト種別 | 対象 | ツール |
|-----------|------|--------|
| ユニットテスト（`#[cfg(test)]`） | acquire/release の基本動作・満杯時の BulkheadFullError 返却・待機タイムアウト検証 | tokio::test |
| モックテスト | `mockall` による実行関数モック・エラー注入テスト | mockall (feature = "mock") |
| 並行性テスト | 複数タスクからの同時実行・上限制御検証・セマフォの公平性確認 | tokio::test（複数スポーン） |
| メトリクステスト | 拒否数・現在の同時実行数の記録検証 | tokio::test (feature = "metrics") |

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-resiliency設計](resiliency.md) — レジリエンス全体設計
- [system-library-circuit-breaker設計](circuit-breaker.md) — サーキットブレーカーと組み合わせた利用パターン
- [可観測性設計](../../architecture/observability/可観測性設計.md) — OpenTelemetry メトリクス設計
- [gRPC設計](../../architecture/api/gRPC設計.md) — サービス間呼び出し設計
