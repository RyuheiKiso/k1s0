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

**Cargo.toml への追加行**:

```toml
k1s0-cache = { path = "../../system/library/rust/cache" }
# テスト時にモックを有効化する場合:
k1s0-cache = { path = "../../system/library/rust/cache", features = ["mock"] }
```

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

**配置先**: `regions/system/library/go/cache/`

```
cache/
├── cache.go
├── cache_test.go
├── go.mod
└── go.sum
```

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

**配置先**: `regions/system/library/typescript/cache/`

```
cache/
├── package.json        # "@k1s0/cache", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # CacheClient, RedisCacheClient, InMemoryCacheClient, CacheConfig, LockGuard, CacheError
└── __tests__/
    └── cache.test.ts
```

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

**配置先**: `regions/system/library/dart/cache/`

```
cache/
├── pubspec.yaml        # k1s0_cache
├── analysis_options.yaml
├── lib/
│   ├── cache.dart
│   └── src/
│       ├── client.dart     # CacheClient abstract, RedisCacheClient, InMemoryCacheClient
│       ├── config.dart     # CacheConfig
│       ├── lock.dart       # LockGuard
│       └── error.dart      # CacheError
└── test/
    └── cache_test.dart
```

**カバレッジ目標**: 90%以上

## Python 実装

**配置先**: `regions/system/library/python/cache/`

### パッケージ構造

```
cache/
├── pyproject.toml
├── src/
│   └── k1s0_cache/
│       ├── __init__.py       # 公開 API（再エクスポート）
│       ├── client.py         # CacheClient ABC・RedisCacheClient・InMemoryCacheClient
│       ├── config.py         # CacheConfig
│       ├── lock.py           # LockGuard（コンテキストマネージャー）
│       ├── exceptions.py     # CacheError
│       └── py.typed
└── tests/
    ├── test_client.py
    └── test_lock.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `CacheClient` | ABC | キャッシュ操作抽象基底クラス（`get`, `set`, `delete`, `exists`, `acquire_lock`） |
| `RedisCacheClient` | class | redis-py 実装（コネクションプール付き） |
| `InMemoryCacheClient` | class | テスト用インメモリ実装 |
| `LockGuard` | class | 分散ロック RAII ガード（コンテキストマネージャー・`async with` 対応） |
| `CacheConfig` | dataclass | Redis URL・プール設定・デフォルト TTL |
| `CacheError` | Exception | キャッシュエラー基底クラス |

### 使用例

```python
import asyncio
from datetime import timedelta
from k1s0_cache import (
    RedisCacheClient, CacheConfig,
)

config = CacheConfig(
    redis_url="redis://localhost:6379",
    pool_size=10,
    default_ttl=timedelta(seconds=300),
)
client = RedisCacheClient(config)

# 値の設定
await client.set("user:123", user_data, ttl=timedelta(seconds=600))

# 値の取得
user = await client.get("user:123")

# 分散ロック（コンテキストマネージャー）
async with await client.acquire_lock("order:lock:456", ttl=timedelta(seconds=30)) as lock:
    # ロック取得中の処理
    pass
# スコープ外で自動解放
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| redis[hiredis] | >=5.2 | Redis クライアント（高速 C パーサー付き） |
| pydantic | >=2.10 | 設定バリデーション |

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock
- カバレッジ目標: 90%以上
- 実行: `pytest` / `ruff check .`

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-featureflag-server設計](system-featureflag-server設計.md) — キャッシュ利用例
- [system-ratelimit-server設計](system-ratelimit-server設計.md) — Redis 共有の検討
- [system-library-correlation設計](system-library-correlation設計.md) — k1s0-correlation ライブラリ
