# k1s0-idempotency ライブラリ設計

## 概要

API リクエストの冪等性保証ライブラリ。`Idempotency-Key` ヘッダーを検出し、重複リクエストを自動で検出・抑制する。TTL 付きのレスポンスキャッシュにより、同一キーの再リクエスト時に同じレスポンスを返す。

**バックエンド可用性**:

| バックエンド | Go | Rust | TypeScript | Dart |
|------------|-----|------|------------|------|
| InMemory | YES | YES | YES | YES |
| Redis | YES | YES | NO | NO |
| PostgreSQL | NO | YES | NO | NO |
| Mock | NO | YES (feature) | NO | NO |
| Middleware (axum) | NO | YES | NO | NO |

> **注意**: PostgreSQL バックエンドは Rust のみ、Redis バックエンドは Go と Rust のみで利用可能。

**インターフェースの差異**: Go は `Set`/`MarkCompleted`/`MarkFailed` メソッドを使用し、TypeScript/Dart は `insert`/`update`/`delete` を使用する。Rust は両方のパターンをサポート（`set`/`mark_completed`/`mark_failed` が正規メソッド、`insert`/`update` が互換エイリアス）。

**配置先**: `regions/system/library/rust/idempotency/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `IdempotencyStore` | トレイト | キー管理の抽象インターフェース（`get`/`set`/`mark_completed`/`mark_failed`/`delete` + エイリアス `insert`/`update`） |
| `RedisIdempotencyStore` | 構造体 | Redis バックエンド実装（TTL 付き） |
| `PostgresIdempotencyStore` | 構造体 | PostgreSQL バックエンド実装 |
| `InMemoryIdempotencyStore` | 構造体 | メモリ内バックエンド実装（テスト/ローカル向け） |
| `MockIdempotencyStore` | 構造体 | テスト用モック（feature = "mock" で有効） |
| `IdempotencyRecord` | 構造体 | `key`, `status`, `request_hash`, `response_body`, `response_status`, `created_at`, `expires_at`, `completed_at` |
| `IdempotencyStatus` | enum | `Pending`（処理中）/ `Completed`（完了）/ `Failed`（失敗） |
| `IdempotencyError` | enum | `Duplicate { key }` / `NotFound { key }` / `InvalidStatusTransition { from, to }` / `SerializationError` / `StorageError(String)` |
| `IdempotencyMiddleware` | 構造体 | axum ミドルウェア実装（`Idempotency-Key` ヘッダー自動処理） |
| `IdempotencyState` | 構造体 | ミドルウェアの共有状態（store/config） |
| `IdempotencyConfig` | 構造体 | TTL・ヘッダー名設定（`ttl_secs: Option<i64>`, `header_name: String`） |
| `IDEMPOTENCY_KEY_HEADER` | 定数 | 冪等キーのヘッダー名（`idempotency-key`） |
| `idempotency_middleware` | 関数 | axum `from_fn_with_state` 用の関数型ミドルウェア |

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
tokio = { version = "1", features = ["sync", "time", "macros"] }
tracing = "0.1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
axum = "0.8"
http = "1"
http-body = "1"
http-body-util = "0.1"
bytes = "1"
tower = { version = "0.5", features = ["util"] }
pin-project-lite = "0.2"
deadpool-redis = { version = "0.18", optional = true }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "chrono"], optional = true }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
tower = { version = "0.5", features = ["util"] }
hyper = "1"
```

**依存追加**: `k1s0-idempotency = { path = "../../system/library/rust/idempotency", features = ["redis"] }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
idempotency/
├── src/
│   ├── lib.rs              # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── store.rs            # IdempotencyStore トレイト・RedisIdempotencyStore・PostgresIdempotencyStore
│   ├── memory.rs           # InMemoryIdempotencyStore
│   ├── middleware.rs       # axum IdempotencyMiddleware
│   ├── record.rs           # IdempotencyRecord・IdempotencyStatus
│   └── error.rs            # IdempotencyError
└── Cargo.toml
```

**IdempotencyStore トレイト**:

```rust
#[async_trait]
pub trait IdempotencyStore: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<IdempotencyRecord>, IdempotencyError>;
    async fn set(&self, record: IdempotencyRecord) -> Result<(), IdempotencyError>;
    async fn mark_completed(&self, key: &str, response_body: Option<String>, response_status: Option<u16>) -> Result<(), IdempotencyError>;
    async fn mark_failed(&self, key: &str, error_body: Option<String>, response_status: Option<u16>) -> Result<(), IdempotencyError>;
    async fn delete(&self, key: &str) -> Result<bool, IdempotencyError>;

    // 互換エイリアス（デフォルト実装）
    async fn insert(&self, record: IdempotencyRecord) -> Result<(), IdempotencyError>;  // set() に委譲
    async fn update(&self, key: &str, status: IdempotencyStatus, response_body: Option<String>, response_status: Option<u16>) -> Result<(), IdempotencyError>;  // status に応じて mark_completed/mark_failed に委譲
}
```

**IdempotencyRecord**:

```rust
pub struct IdempotencyRecord {
    pub key: String,
    pub status: IdempotencyStatus,
    pub request_hash: Option<String>,
    pub response_body: Option<String>,
    pub response_status: Option<u16>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl IdempotencyRecord {
    pub fn new(key: String, ttl_secs: Option<i64>) -> Self;
    pub fn is_expired(&self) -> bool;
    pub fn complete(self, response_body: Option<String>, response_status: Option<u16>) -> Self;
    pub fn fail(self, error: String) -> Self;
}
```

**IdempotencyConfig / IdempotencyState / IdempotencyMiddleware**:

```rust
pub struct IdempotencyConfig {
    pub ttl_secs: Option<i64>,      // デフォルト: Some(86_400) (24h)
    pub header_name: String,         // デフォルト: "idempotency-key"
}

impl IdempotencyConfig {
    // new() は無い。Default::default() を使用
    pub fn with_ttl(self, ttl: Duration) -> Self;
}

pub struct IdempotencyState {
    pub store: Arc<dyn IdempotencyStore>,
    pub config: IdempotencyConfig,
}

impl IdempotencyState {
    pub fn new(store: Arc<dyn IdempotencyStore>) -> Self;            // デフォルト config
    pub fn with_config(store: Arc<dyn IdempotencyStore>, config: IdempotencyConfig) -> Self;
}

pub struct IdempotencyMiddleware { /* ... */ }

impl IdempotencyMiddleware {
    pub fn new(store: Arc<dyn IdempotencyStore>, config: IdempotencyConfig) -> Self;
    pub fn state(&self) -> IdempotencyState;
}
```

**使用例**:

```rust
use std::sync::Arc;
use k1s0_idempotency::{IdempotencyMiddleware, IdempotencyConfig, RedisIdempotencyStore};
use axum::{Router, routing::post};

let store = RedisIdempotencyStore::new("redis://localhost:6379").await.unwrap();
let config = IdempotencyConfig::default()
    .with_ttl(std::time::Duration::from_secs(86400)); // 24時間

let app = Router::new()
    .route("/api/v1/orders", post(create_order))
    .layer(IdempotencyMiddleware::new(Arc::new(store), config));

// ハンドラーは通常通り実装。ミドルウェアが Idempotency-Key を自動処理
async fn create_order(/* ... */) -> impl IntoResponse {
    // 同一キーの2回目以降の呼び出しはミドルウェアが自動的にキャッシュ応答を返す
    // ...
}
```

## Go 実装

**配置先**: `regions/system/library/go/idempotency/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/redis/go-redis/v9 v9.7.0`, `github.com/stretchr/testify v1.11.1`

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
    Error      string
    CreatedAt  time.Time
    ExpiresAt  time.Time
}

func NewIdempotencyRecord(key string, ttl *time.Duration) *IdempotencyRecord

type IdempotencyStatus string

const (
    StatusPending   IdempotencyStatus = "pending"
    StatusCompleted IdempotencyStatus = "completed"
    StatusFailed    IdempotencyStatus = "failed"
)

type IdempotencyError struct {
    Code    string // DUPLICATE / NOT_FOUND / EXPIRED
    Message string
}

func NewDuplicateError(key string) *IdempotencyError
func NewNotFoundError(key string) *IdempotencyError
func NewExpiredError(key string) *IdempotencyError

type InMemoryIdempotencyStore struct{}
func NewInMemoryIdempotencyStore() *InMemoryIdempotencyStore

type RedisStoreOption func(*RedisIdempotencyStore)
func WithRedisKeyPrefix(prefix string) RedisStoreOption
func WithRedisDefaultTTL(ttl time.Duration) RedisStoreOption

type RedisIdempotencyStore struct{}
func NewRedisIdempotencyStore(client redis.Cmdable, opts ...RedisStoreOption) *RedisIdempotencyStore
func NewRedisIdempotencyStoreFromURL(url string, opts ...RedisStoreOption) (*RedisIdempotencyStore, error)
```

`NewRedisIdempotencyStore` のデフォルト TTL は `24時間`（`24 * time.Hour`）。

> **注意**: Go の `IdempotencyStore` は `delete` メソッドを持たない。TypeScript/Dart/Rust とは異なる。

### Redis バックエンドの並行安全性（Lua CAS）

`RedisIdempotencyStore` の `MarkCompleted` / `MarkFailed` メソッドは、
**Redis Lua スクリプトによるアトミック CAS（Compare-and-Swap）** で実装されている。

旧実装では `GET → メモリ更新 → SET` という非アトミックなパターンを使用していたため、
複数 goroutine が同じキーを並行操作した場合に TOCTOU 競合（Lost Update）が発生する問題があった。

新実装では `redis.NewScript(luaScript)` + `Script.Run()` を使用し、
`GET → cjson.decode → フィールド更新 → cjson.encode → SET KEEPTTL` を
1回のアトミック操作として実行する。これにより並行安全性が保証される。

- `KEEPTTL` オプションにより既存の TTL が透過的に保持される（Redis 6.0 以降が必要）
- Go の `[]byte` フィールド（`response`）は `base64.StdEncoding` でエンコードして Lua に渡す
- 詳細は [ADR-0010](../../architecture/adr/0010-idempotency-atomic-cas.md) を参照

## TypeScript 実装

**配置先**: `regions/system/library/typescript/idempotency/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export type IdempotencyStatus = 'pending' | 'completed' | 'failed';

export interface IdempotencyRecord {
  key: string;
  status: IdempotencyStatus;
  responseBody?: string;
  statusCode?: number;
  createdAt: Date;
  expiresAt?: Date;
  completedAt?: Date;
}

export function createIdempotencyRecord(key: string, ttlSecs?: number): IdempotencyRecord;

export interface IdempotencyStore {
  get(key: string): Promise<IdempotencyRecord | null>;
  insert(record: IdempotencyRecord): Promise<void>;
  update(key: string, status: IdempotencyStatus, body?: string, code?: number): Promise<void>;
  delete(key: string): Promise<boolean>;
}

export class InMemoryIdempotencyStore implements IdempotencyStore {
  get(key: string): Promise<IdempotencyRecord | null>;
  insert(record: IdempotencyRecord): Promise<void>;    // 重複キーで DuplicateKeyError をスロー
  update(key: string, status: IdempotencyStatus, body?: string, code?: number): Promise<void>;
  delete(key: string): Promise<boolean>;
}

export class IdempotencyError extends Error {
  constructor(message: string, public readonly code: string);
}

export class DuplicateKeyError extends Error {
  constructor(public readonly key: string);
}
```

> **注意**: TypeScript 実装には `RedisIdempotencyStore` と `IdempotencyConfig` は存在しない。`InMemoryIdempotencyStore` のみ。

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/idempotency/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**pubspec.yaml**:

```yaml
name: k1s0_idempotency
description: Idempotency key store for k1s0 platform
version: 0.1.0

environment:
  sdk: '>=3.4.0 <4.0.0'

dev_dependencies:
  lints: ^4.0.0
  test: ^1.24.0
```

**主要 API**:

```dart
enum IdempotencyStatus { pending, completed, failed }

class IdempotencyRecord {
  final String key;
  IdempotencyStatus status;
  String? responseBody;
  int? statusCode;
  final DateTime createdAt;
  final DateTime? expiresAt;
  DateTime? completedAt;

  IdempotencyRecord({
    required this.key,
    this.status = IdempotencyStatus.pending,
    this.responseBody,
    this.statusCode,
    DateTime? createdAt,
    this.expiresAt,
    this.completedAt,
  });

  factory IdempotencyRecord.create(String key, {int? ttlSecs});
}

abstract class IdempotencyStore {
  Future<IdempotencyRecord?> get(String key);
  Future<void> insert(IdempotencyRecord record);
  Future<void> update(String key, IdempotencyStatus status, {String? body, int? code});
  Future<bool> delete(String key);
}

class InMemoryIdempotencyStore implements IdempotencyStore {
  // insert() で重複キーは DuplicateKeyError をスロー
}

class IdempotencyError implements Exception {
  final String message;
  final String code;
  const IdempotencyError(this.message, this.code);
}

class DuplicateKeyError implements Exception {
  final String key;
  const DuplicateKeyError(this.key);
}
```

> **注意**: Dart 実装には `RedisIdempotencyStore` と `IdempotencyConfig` は存在しない。`InMemoryIdempotencyStore` のみ。

**カバレッジ目標**: 90%以上

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) -- ライブラリ一覧・テスト方針
- [system-library-cache設計](../data/cache.md) -- k1s0-cache ライブラリ
- [API設計](../../architecture/api/API設計.md) -- Idempotency-Key ヘッダー仕様
- [コーディング規約](../../architecture/conventions/コーディング規約.md) -- コーディング規約
