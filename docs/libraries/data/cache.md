# k1s0-cache ライブラリ設計

## 概要

Redis 分散キャッシュ抽象化ライブラリ。`CacheClient` トレイトにより `get`/`set`/`delete`/`exists`/`set_nx`/`expire` の統一インターフェースを提供する。Redis Cluster・Sentinel・スタンドアロンをサポート。

**配置先**: `regions/system/library/rust/cache/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `CacheClient` | トレイト | キャッシュ操作の抽象インターフェース（`get`/`set`/`delete`/`exists`/`set_nx`/`expire`） |
| `CacheEntry` | 構造体 | キャッシュエントリ（value・expires_at） |
| `LockGuard` | 構造体 | ロックガード（key・lock_value） |
| `MockCacheClient` | 構造体 | テスト用モック（feature = "mock" で有効） |
| `CacheError` | enum | 接続エラー・シリアライゼーションエラー等 |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-cache"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
deadpool-redis = { version = "0.18", features = ["cluster", "sentinel"] }
redis = { version = "0.27", features = ["tokio-comp", "cluster-async"] }
uuid = { version = "1", features = ["v4"] }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
testcontainers = "0.23"
```

**依存追加**: `k1s0-cache = { path = "../../system/library/rust/cache" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
cache/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── client.rs       # CacheClient トレイト・CacheEntry・LockGuard・MockCacheClient
│   └── error.rs        # CacheError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_cache::CacheClient;
use std::time::Duration;

// 値の設定（文字列ベース）
client.set("user:123", &serde_json::to_string(&user)?, Some(Duration::from_secs(600))).await?;

// 値の取得
let value: Option<String> = client.get("user:123").await?;

// キーが存在しない場合のみセット（分散ロック等に利用）
let acquired = client.set_nx("lock:order:456", "owner-id", Duration::from_secs(30)).await?;

// TTL 更新
client.expire("user:123", Duration::from_secs(900)).await?;
```

## Go 実装

**配置先**: `regions/system/library/go/cache/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: なし（標準ライブラリのみ）

**主要インターフェース**:

```go
type CacheClient interface {
    Get(ctx context.Context, key string) (*string, error)
    Set(ctx context.Context, key string, value string, ttl *time.Duration) error
    Delete(ctx context.Context, key string) (bool, error)
    Exists(ctx context.Context, key string) (bool, error)
    SetNX(ctx context.Context, key string, value string, ttl time.Duration) (bool, error)
    Expire(ctx context.Context, key string, ttl time.Duration) (bool, error)
}

type CacheError struct {
    Code    string
    Message string
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/cache/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export class CacheError extends Error {
  constructor(message: string, public readonly code: string);
}

export interface CacheClient {
  get(key: string): Promise<string | null>;
  set(key: string, value: string, ttlMs?: number): Promise<void>;
  delete(key: string): Promise<boolean>;
  exists(key: string): Promise<boolean>;
  setNX(key: string, value: string, ttlMs: number): Promise<boolean>;
}

export class InMemoryCacheClient implements CacheClient {
  get(key: string): Promise<string | null>;
  set(key: string, value: string, ttlMs?: number): Promise<void>;
  delete(key: string): Promise<boolean>;
  exists(key: string): Promise<boolean>;
  setNX(key: string, value: string, ttlMs: number): Promise<boolean>;
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/cache/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**カバレッジ目標**: 90%以上

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-featureflag-server設計](../../servers/featureflag/server.md) — キャッシュ利用例
- [system-ratelimit-server設計](../../servers/ratelimit/server.md) — Redis 共有の検討
- [system-library-correlation設計](../observability/correlation.md) — k1s0-correlation ライブラリ
