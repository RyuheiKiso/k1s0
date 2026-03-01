# k1s0-distributed-lock ライブラリ設計

## 概要

分散ロック実装ライブラリ。`DistributedLock` トレイト（インターフェース）により `acquire`/`release`/`is_locked` の統一インターフェースを提供する。`LockGuard`（key・token）による安全な解放と TTL 付きロックをサポート。

InMemory（テスト用）・Redis（本番用）の 2 バックエンドをサポート。PostgreSQL（advisory lock）バックエンドは Phase 2 で追加予定。TTL 超過時の自動失効を実現する。

**配置先**: `regions/system/library/rust/distributed-lock/`

## 公開 API

最小共通 API（全 4 言語）:

| メソッド | 戻り値 | 説明 |
|---------|--------|------|
| `acquire(key, ttl)` | `LockGuard` | ロック取得。取得できない場合はエラー（`AlreadyLocked`） |
| `release(guard)` | `void` | ロック解放。トークン不一致の場合はエラー |
| `is_locked(key)` | `bool` | ロックが保持されているか確認 |

Rust 追加 API:

| メソッド | 説明 |
|---------|------|
| `extend(guard, ttl)` | TTL 延長（Rust・Go Redis 実装のみ） |

Rust 公開型:

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `DistributedLock` | トレイト | ロック操作の抽象インターフェース |
| `InMemoryLock` | 構造体 | テスト用インメモリ実装 |
| `RedisLock` | 構造体 | Redis SET NX PX 実装（feature = "redis" で有効） |
| `LockGuard` | 構造体 | ロックガード（key・token） |
| `LockError` | enum | `AlreadyLocked`・`LockNotFound`・`TokenMismatch`・`Internal` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-distributed-lock"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]
postgres = ["dep:sqlx"]
redis = ["dep:redis"]

[dependencies]
async-trait = "0.1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time", "macros"] }
uuid = { version = "1", features = ["v4"] }
mockall = { version = "0.13", optional = true }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"], optional = true }
redis = { version = "0.27", features = ["tokio-comp", "connection-manager", "script"], optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `k1s0-distributed-lock = { path = "../../system/library/rust/distributed-lock", features = ["postgres"] }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
distributed-lock/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── lock.rs         # DistributedLock トレイト・LockGuard・InMemoryLock
│   ├── redis.rs        # RedisLock（feature = "redis" で有効）
│   └── error.rs        # LockError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_distributed_lock::{DistributedLock, InMemoryLock, LockError};
use std::time::Duration;

let lock = InMemoryLock::new();

// ロック取得
let guard = lock.acquire("order:process:456", Duration::from_secs(30)).await?;
// クリティカルセクションの処理
process_order(order_id).await?;
// ロック解放
lock.release(guard).await?;

// ロック確認
let is_locked = lock.is_locked("order:process:456").await?;
```

## Go 実装

**配置先**: `regions/system/library/go/distributed-lock/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/redis/go-redis/v9`

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

**配置先**: `regions/system/library/typescript/distributed-lock/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export interface LockGuard {
  key: string;
  token: string;
}

export class LockError extends Error {
  constructor(message: string, public code: string);
}

export class InMemoryLock {
  async acquire(key: string, ttlMs: number): Promise<LockGuard>;
  async release(guard: LockGuard): Promise<void>;
  async isLocked(key: string): Promise<boolean>;
}
```

> PostgreSQL・Redis バックエンドは Phase 2 で追加予定。

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/distributed_lock/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要インターフェース**:

```dart
class LockGuard {
  final String key;
  final String token;
}

abstract class DistributedLock {
  Future<LockGuard> acquire(String key, Duration ttl);
  Future<void> release(LockGuard guard);
  Future<bool> isLocked(String key);
}

class InMemoryDistributedLock implements DistributedLock {
  // ... 上記メソッドすべてを実装
}
```

> PostgreSQL・Redis バックエンドは Phase 2 で追加予定。

**カバレッジ目標**: 90%以上

## テスト戦略

| テスト種別 | 対象 | ツール |
|-----------|------|--------|
| ユニットテスト（`#[cfg(test)]`） | `LockConfig` バリデーション・トークン生成・エラー型 | tokio::test |
| 統合テスト（PostgreSQL） | advisory lock 取得・競合・TTL 失効 | testcontainers (postgres イメージ) |
| 統合テスト（Redis） | SET NX PX・競合・TTL 失効・自動解放 | testcontainers (redis イメージ) |
| 並行性テスト | 複数タスクからの同時 `acquire` でロック一元化を検証 | tokio::test（複数スポーン） |

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-cache設計](cache.md) — Redis 共有の検討
- [system-library-saga設計](../resilience/saga.md) — Saga ステップの排他制御
- [system-database設計](../../servers/_common/database.md) — PostgreSQL 接続プール設計
