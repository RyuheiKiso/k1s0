# k1s0-distributed-lock ライブラリ設計

## 概要

PostgreSQL advisory lock と Redis を使った分散ロック実装ライブラリ。`DistributedLock` トレイトにより `acquire`/`release`/`try_acquire` の統一インターフェースを提供する。RAII ガード（`LockGuard`）による自動解放と TTL 付きロックをサポート。

バックエンドを `DistributedLock` トレイトで抽象化しているため、PostgreSQL と Redis を用途に応じて切り替え可能。TTL 超過時の自動失効と、リトライ付き `acquire` でのスピンロック回避を実現する。

**配置先**: `regions/system/library/rust/distributed-lock/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `DistributedLock` | トレイト | ロック操作の抽象インターフェース |
| `PostgresDistributedLock` | 構造体 | PostgreSQL advisory lock 実装（sqlx 使用） |
| `RedisDistributedLock` | 構造体 | Redis SET NX PX 実装（deadpool-redis 使用） |
| `LockGuard` | 構造体 | RAII ガード（Drop で自動解放） |
| `LockConfig` | 構造体 | TTL・リトライ間隔・最大リトライ回数設定 |
| `LockError` | enum | `AlreadyLocked`・`Timeout`・`BackendError` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-distributed-lock"
version = "0.1.0"
edition = "2021"

[features]
postgres = ["sqlx"]
redis = ["deadpool-redis"]

[dependencies]
async-trait = "0.1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
uuid = { version = "1", features = ["v4"] }
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio"], optional = true }
deadpool-redis = { version = "0.18", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
testcontainers = "0.23"
```

**Cargo.toml への追加行**:

```toml
# PostgreSQL バックエンドを使用する場合:
k1s0-distributed-lock = { path = "../../system/library/rust/distributed-lock", features = ["postgres"] }
# Redis バックエンドを使用する場合:
k1s0-distributed-lock = { path = "../../system/library/rust/distributed-lock", features = ["redis"] }
```

**モジュール構成**:

```
distributed-lock/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── lock.rs         # DistributedLock トレイト・LockGuard
│   ├── postgres.rs     # PostgresDistributedLock
│   ├── redis.rs        # RedisDistributedLock
│   ├── config.rs       # LockConfig
│   └── error.rs        # LockError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_distributed_lock::{PostgresDistributedLock, LockConfig, DistributedLock};
use std::time::Duration;

// PostgreSQL advisory lock
let config = LockConfig::new()
    .ttl(Duration::from_secs(30))
    .retry_interval(Duration::from_millis(100))
    .max_retries(10);

let lock = PostgresDistributedLock::new(pool.clone(), config);

// ロック取得（RAII ガード）
let guard = lock.acquire("order:process:456").await?;
// クリティカルセクションの処理
process_order(order_id).await?;
// guard がスコープ外に出ると自動解放

// try_acquire でノンブロッキング取得
match lock.try_acquire("order:process:789").await {
    Ok(guard) => process_order_with_guard(guard).await?,
    Err(LockError::AlreadyLocked) => return Err(anyhow!("order already being processed")),
    Err(e) => return Err(e.into()),
}
```

## Go 実装

**配置先**: `regions/system/library/go/distributed-lock/`

```
distributed-lock/
├── distributedlock.go
├── distributedlock_test.go
├── go.mod
└── go.sum
```

**依存関係**: なし（標準ライブラリのみ）

**主要インターフェース**:

```go
var ErrAlreadyLocked = errors.New("既にロックされています")
var ErrTokenMismatch = errors.New("トークンが一致しません")
var ErrLockNotFound  = errors.New("ロックが見つかりません")

type LockGuard struct {
    Key   string
    Token string
}

type DistributedLock interface {
    Acquire(ctx context.Context, key string, ttl time.Duration) (*LockGuard, error)
    Release(ctx context.Context, guard *LockGuard) error
    IsLocked(ctx context.Context, key string) (bool, error)
}

type InMemoryLock struct{}

func NewInMemoryLock() *InMemoryLock
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/distributed-lock/`

```
distributed-lock/
├── package.json        # "@k1s0/distributed-lock", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # DistributedLock, PostgresDistributedLock, RedisDistributedLock, LockGuard, LockConfig, LockError
└── __tests__/
    ├── postgres-lock.test.ts
    └── redis-lock.test.ts
```

**主要 API**:

```typescript
export interface LockGuard {
  key: string;
  token: string;
  release(): Promise<void>;
}

export interface LockConfig {
  ttlMs: number;
  retryIntervalMs?: number;
  maxRetries?: number;
}

export interface DistributedLock {
  acquire(key: string): Promise<LockGuard>;
  tryAcquire(key: string): Promise<LockGuard | null>;
}

export class PostgresDistributedLock implements DistributedLock {
  constructor(pool: Pool, config: LockConfig);
  acquire(key: string): Promise<LockGuard>;
  tryAcquire(key: string): Promise<LockGuard | null>;
}

export class RedisDistributedLock implements DistributedLock {
  constructor(client: Redis, config: LockConfig);
  acquire(key: string): Promise<LockGuard>;
  tryAcquire(key: string): Promise<LockGuard | null>;
}

export class LockError extends Error {
  constructor(
    message: string,
    public readonly kind: 'already_locked' | 'timeout' | 'backend_error'
  );
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/distributed-lock/`

```
distributed-lock/
├── pubspec.yaml        # k1s0_distributed_lock
├── analysis_options.yaml
├── lib/
│   ├── distributed_lock.dart
│   └── src/
│       ├── lock.dart        # DistributedLock abstract・LockGuard
│       ├── postgres.dart    # PostgresDistributedLock
│       ├── redis.dart       # RedisDistributedLock
│       ├── config.dart      # LockConfig
│       └── error.dart       # LockError
└── test/
    ├── postgres_lock_test.dart
    └── redis_lock_test.dart
```

**カバレッジ目標**: 90%以上

## テスト戦略

| テスト種別 | 対象 | ツール |
|-----------|------|--------|
| ユニットテスト（`#[cfg(test)]`） | `LockConfig` バリデーション・トークン生成・エラー型 | tokio::test |
| 統合テスト（PostgreSQL） | advisory lock 取得・競合・TTL 失効 | testcontainers (postgres イメージ) |
| 統合テスト（Redis） | SET NX PX・競合・TTL 失効・自動解放 | testcontainers (redis イメージ) |
| 並行性テスト | 複数タスクからの同時 `acquire` でロック一元化を検証 | tokio::test（複数スポーン） |

## 関連ドキュメント

- [system-library-概要](../overview/概要.md) — ライブラリ一覧・テスト方針
- [system-library-cache設計](cache設計.md) — Redis 共有の検討
- [system-library-saga設計](../resilience/saga設計.md) — Saga ステップの排他制御
- [system-database設計](../../system-servers/common/database設計.md) — PostgreSQL 接続プール設計
