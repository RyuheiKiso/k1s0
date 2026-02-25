# k1s0-idempotency ライブラリ設計

## 概要

API リクエストの冪等性保証ライブラリ。`Idempotency-Key` ヘッダーを検出し、重複リクエストを自動で検出・抑制する。PostgreSQL または Redis をバックエンドとして選択可能。TTL 付きのレスポンスキャッシュにより、同一キーの再リクエスト時に同じレスポンスを返す。

**配置先**: `regions/system/library/rust/idempotency/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `IdempotencyStore` | トレイト | キー管理の抽象インターフェース |
| `RedisIdempotencyStore` | 構造体 | Redis バックエンド実装（TTL 付き） |
| `PostgresIdempotencyStore` | 構造体 | PostgreSQL バックエンド実装 |
| `MockIdempotencyStore` | 構造体 | テスト用モック（feature = "mock" で有効） |
| `IdempotencyRecord` | 構造体 | キー・ステータス・キャッシュされたレスポンス・TTL |
| `IdempotencyStatus` | enum | `Pending`（処理中）/ `Completed`（完了）/ `Failed`（失敗） |
| `IdempotencyMiddleware` | 構造体 | axum ミドルウェア実装（`Idempotency-Key` ヘッダー自動処理） |
| `IdempotencyConfig` | 構造体 | TTL・ヘッダー名・バックエンド設定 |
| `IdempotencyError` | enum | 重複リクエスト・ストアエラー等 |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-idempotency"
version = "0.1.0"
edition = "2021"

[features]
redis = ["deadpool-redis"]
postgres = ["sqlx"]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
uuid = { version = "1", features = ["v4"] }
axum = { version = "0.7", features = ["macros"] }
deadpool-redis = { version = "0.18", optional = true }
sqlx = { version = "0.8", optional = true }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
axum-test = "16"
```

**Cargo.toml への追加行**:

```toml
k1s0-idempotency = { path = "../../system/library/rust/idempotency", features = ["redis"] }
# PostgreSQL バックエンドを使用する場合:
k1s0-idempotency = { path = "../../system/library/rust/idempotency", features = ["postgres"] }
# テスト時にモックを有効化する場合:
k1s0-idempotency = { path = "../../system/library/rust/idempotency", features = ["redis", "mock"] }
```

**モジュール構成**:

```
idempotency/
├── src/
│   ├── lib.rs              # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── store.rs            # IdempotencyStore トレイト・RedisIdempotencyStore・PostgresIdempotencyStore
│   ├── middleware.rs       # axum IdempotencyMiddleware
│   ├── record.rs           # IdempotencyRecord・IdempotencyStatus
│   ├── config.rs           # IdempotencyConfig
│   └── error.rs            # IdempotencyError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_idempotency::{IdempotencyMiddleware, IdempotencyConfig, RedisIdempotencyStore};
use axum::{Router, routing::post};

let store = RedisIdempotencyStore::new("redis://localhost:6379").await.unwrap();
let config = IdempotencyConfig::new()
    .with_ttl(std::time::Duration::from_secs(86400)); // 24時間

let app = Router::new()
    .route("/api/v1/orders", post(create_order))
    .layer(IdempotencyMiddleware::new(store, config));

// ハンドラーは通常通り実装。ミドルウェアが Idempotency-Key を自動処理
async fn create_order(/* ... */) -> impl IntoResponse {
    // 同一キーの2回目以降の呼び出しはミドルウェアが自動的にキャッシュ応答を返す
    // ...
}
```

## Go 実装

**配置先**: `regions/system/library/go/idempotency/`

```
idempotency/
├── idempotency.go
├── store.go
├── middleware.go
├── idempotency_test.go
├── go.mod
└── go.sum
```

**依存関係**: `github.com/redis/go-redis/v9 v9.7.0`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type IdempotencyStore interface {
    Get(ctx context.Context, key string) (*IdempotencyRecord, error)
    Set(ctx context.Context, key string, record *IdempotencyRecord) error
    MarkCompleted(ctx context.Context, key string, response []byte, statusCode int) error
    MarkFailed(ctx context.Context, key string, err error) error
}

type IdempotencyRecord struct {
    Key        string
    Status     IdempotencyStatus
    Response   []byte
    StatusCode int
    CreatedAt  time.Time
    ExpiresAt  time.Time
}

type IdempotencyStatus string

const (
    StatusPending   IdempotencyStatus = "pending"
    StatusCompleted IdempotencyStatus = "completed"
    StatusFailed    IdempotencyStatus = "failed"
)
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/idempotency/`

```
idempotency/
├── package.json        # "@k1s0/idempotency", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # IdempotencyStore, RedisIdempotencyStore, IdempotencyRecord, IdempotencyMiddleware, IdempotencyConfig, IdempotencyError
└── __tests__/
    └── idempotency.test.ts
```

**主要 API**:

```typescript
export interface IdempotencyStore {
  get(key: string): Promise<IdempotencyRecord | null>;
  set(key: string, record: IdempotencyRecord, ttlMs: number): Promise<void>;
  markCompleted(key: string, response: unknown, statusCode: number): Promise<void>;
  markFailed(key: string, error: string): Promise<void>;
}

export type IdempotencyStatus = 'pending' | 'completed' | 'failed';

export interface IdempotencyRecord {
  key: string;
  status: IdempotencyStatus;
  response?: unknown;
  statusCode?: number;
  createdAt: Date;
  expiresAt: Date;
}

export interface IdempotencyConfig {
  store: IdempotencyStore;
  ttlMs: number;
  headerName?: string; // デフォルト: "Idempotency-Key"
}

export class RedisIdempotencyStore implements IdempotencyStore {
  constructor(redisUrl: string);
  get(key: string): Promise<IdempotencyRecord | null>;
  set(key: string, record: IdempotencyRecord, ttlMs: number): Promise<void>;
  markCompleted(key: string, response: unknown, statusCode: number): Promise<void>;
  markFailed(key: string, error: string): Promise<void>;
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/idempotency/`

```
idempotency/
├── pubspec.yaml        # k1s0_idempotency
├── analysis_options.yaml
├── lib/
│   ├── idempotency.dart
│   └── src/
│       ├── store.dart          # IdempotencyStore abstract, RedisIdempotencyStore
│       ├── record.dart         # IdempotencyRecord, IdempotencyStatus
│       ├── config.dart         # IdempotencyConfig
│       └── error.dart          # IdempotencyError
└── test/
    └── idempotency_test.dart
```

**カバレッジ目標**: 90%以上

## Python 実装

**配置先**: `regions/system/library/python/idempotency/`

### パッケージ構造

```
idempotency/
├── pyproject.toml
├── src/
│   └── k1s0_idempotency/
│       ├── __init__.py       # 公開 API（再エクスポート）
│       ├── store.py          # IdempotencyStore ABC・RedisIdempotencyStore・PostgresIdempotencyStore
│       ├── record.py         # IdempotencyRecord・IdempotencyStatus
│       ├── config.py         # IdempotencyConfig
│       ├── middleware.py     # ASGI/FastAPI ミドルウェア
│       ├── exceptions.py     # IdempotencyError
│       └── py.typed
└── tests/
    ├── test_store.py
    └── test_middleware.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `IdempotencyStore` | ABC | キー管理抽象基底クラス（`get`, `set`, `mark_completed`, `mark_failed`） |
| `RedisIdempotencyStore` | class | Redis バックエンド実装（TTL 付き） |
| `PostgresIdempotencyStore` | class | PostgreSQL バックエンド実装（asyncpg 使用） |
| `IdempotencyRecord` | dataclass | キー・ステータス・レスポンス・TTL |
| `IdempotencyStatus` | Enum | PENDING / COMPLETED / FAILED |
| `IdempotencyConfig` | dataclass | TTL・ヘッダー名・バックエンド設定 |
| `IdempotencyError` | Exception | エラー基底クラス |

### 使用例

```python
import asyncio
from datetime import timedelta
from k1s0_idempotency import (
    RedisIdempotencyStore, IdempotencyConfig,
)

config = IdempotencyConfig(
    ttl=timedelta(hours=24),
    header_name="Idempotency-Key",
)
store = RedisIdempotencyStore(redis_url="redis://localhost:6379")

# レコード取得
record = await store.get("unique-request-key-123")

if record is not None and record.status == "completed":
    # キャッシュされたレスポンスを返す
    return record.response

# 新しいリクエストとして処理
await store.set("unique-request-key-123", IdempotencyRecord(
    key="unique-request-key-123",
    status="pending",
))

# 処理完了後にマーク
await store.mark_completed(
    "unique-request-key-123",
    response=response_body,
    status_code=201,
)
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| redis[hiredis] | >=5.2 | Redis クライアント（高速 C パーサー付き） |
| asyncpg | >=0.30 | PostgreSQL 非同期クライアント |

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock
- カバレッジ目標: 90%以上
- 実行: `pytest` / `ruff check .`

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-cache設計](system-library-cache設計.md) — k1s0-cache ライブラリ
- [API設計](API設計.md) — Idempotency-Key ヘッダー仕様
- [コーディング規約](コーディング規約.md) — コーディング規約
