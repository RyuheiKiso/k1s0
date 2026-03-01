# k1s0-outbox ライブラリ設計

## 概要

トランザクショナルアウトボックスパターンライブラリ。データベーストランザクションと Kafka メッセージ発行の原子性を保証する。`OutboxMessage`（指数バックオフリトライ）、`OutboxStore` トレイト、`OutboxPublisher` トレイト、`OutboxProcessor` を提供する。

**配置先**: `regions/system/library/rust/outbox/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `OutboxMessage` | 構造体 | アウトボックスに保存するメッセージ（`id`・`topic`・`partition_key`・`payload`・`status`・`retry_count`・`max_retries`・`last_error`・`created_at`・`process_after`） |
| `OutboxStatus` | enum | メッセージのステータス（`Pending`・`Processing`・`Delivered`・`Failed`・`DeadLetter`） |
| `OutboxStore` | トレイト | アウトボックスメッセージの永続化抽象（`save`・`fetch_pending`・`update`・`delete_delivered`） |
| `OutboxPublisher` | トレイト | アウトボックスメッセージの発行インターフェース（`publish`） |
| `OutboxProcessor` | 構造体 | `OutboxStore` から未発行メッセージを取得し `OutboxPublisher` 経由で発行するバッチプロセッサ（`batch_size` 指定、デフォルト 100） |
| `OutboxError` | enum | StoreError・PublishError・SerializationError・NotFound エラー型 |
| `PostgresOutboxStore` | 構造体 | PostgreSQL を使った `OutboxStore` 実装（feature = "postgres" で有効） |
| `OutboxMessage::new` / `createOutboxMessage` / `NewOutboxMessage` | ファクトリ | 新しい OutboxMessage を生成する（Rust: `OutboxMessage::new(topic, partition_key, payload)`、Go: `NewOutboxMessage(topic, partitionKey, payload)`、TS/Dart: `createOutboxMessage(topic, partitionKey, payload)`） |
| `mark_processing` / `mark_delivered` / `mark_failed` | メソッド | ステータス遷移メソッド。Rust/Go/Dart は OutboxMessage のメソッド、TS のみ外部関数（`markProcessing(msg)` 等） |
| `is_processable` | メソッド | メッセージが処理可能か判定（Pending/Failed かつ process_after 到来済み）。Dart は getter（`isProcessable`） |
| `canTransitionTo` | 関数 | ステータス遷移の妥当性チェック（TS/Dart のみ。Go/Rust は未実装） |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-outbox"
version = "0.1.0"
edition = "2021"

[features]
postgres = ["sqlx"]

[dependencies]
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
uuid = { version = "1", features = ["v4", "serde"] }
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"], optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
mockall = "0.13"
```

**依存追加**: `k1s0-outbox = { path = "../../system/library/rust/outbox" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
outbox/
├── src/
│   ├── lib.rs              # 公開 API（再エクスポート）
│   ├── error.rs            # OutboxError（StoreError・PublishError・SerializationError・NotFound）
│   ├── message.rs          # OutboxMessage・OutboxStatus（指数バックオフ計算含む）
│   ├── processor.rs        # OutboxPublisher トレイト・OutboxProcessor（バッチ処理・リトライ制御）
│   ├── store.rs            # OutboxStore トレイト
│   └── postgres_store.rs   # PostgresOutboxStore（feature = "postgres"）
└── Cargo.toml
```

**使用例**:

```rust
use std::sync::Arc;
use k1s0_outbox::{OutboxMessage, OutboxProcessor, OutboxStore};

// ドメインイベント保存とメッセージ発行を同一トランザクションで実行
async fn create_user_with_event<S: OutboxStore>(
    store: &S,
    user_id: &str,
) -> Result<(), k1s0_outbox::OutboxError> {
    let payload = serde_json::json!({ "user_id": user_id });
    let msg = OutboxMessage::new(
        "k1s0.system.auth.user-created.v1",
        user_id,   // partition_key
        payload,
    );
    // DB トランザクション内で保存（Saga の一部として）
    store.save(&msg).await
}

// バッチプロセッサで未発行メッセージを処理
let processor = OutboxProcessor::new(
    Arc::new(store),
    Arc::new(publisher),
    /* batch_size */ 10,
);
let processed = processor.process_batch().await?;
```

**PostgreSQL ストア**（feature = "postgres"）:

```rust
use k1s0_outbox::PostgresOutboxStore;
use sqlx::PgPool;

let pool = PgPool::connect("postgres://...").await?;
let store = PostgresOutboxStore::new(pool);
// OutboxStore トレイトの全メソッド（save, fetch_pending, update, delete_delivered）を実装
```

## Go 実装

**配置先**: `regions/system/library/go/outbox/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/google/uuid v1.6.0`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type OutboxStatus string // "PENDING" | "PROCESSING" | "DELIVERED" | "FAILED" | "DEAD_LETTER"

type OutboxMessage struct {
    ID           string
    Topic        string
    PartitionKey string
    Payload      json.RawMessage
    Status       OutboxStatus
    RetryCount   int
    MaxRetries   int
    LastError    string
    CreatedAt    time.Time
    ProcessAfter time.Time
}

func NewOutboxMessage(topic, partitionKey string, payload json.RawMessage) OutboxMessage
func (m *OutboxMessage) MarkProcessing()
func (m *OutboxMessage) MarkDelivered()
func (m *OutboxMessage) MarkFailed(errMsg string)
func (m *OutboxMessage) IsProcessable() bool

type OutboxStore interface {
    Save(ctx context.Context, msg *OutboxMessage) error
    FetchPending(ctx context.Context, limit int) ([]OutboxMessage, error)
    Update(ctx context.Context, msg *OutboxMessage) error
    DeleteDelivered(ctx context.Context, olderThanDays int) (int64, error)
}

type OutboxPublisher interface {
    Publish(ctx context.Context, msg *OutboxMessage) error
}

type OutboxProcessor struct { /* store, publisher, batchSize */ }
func NewOutboxProcessor(store OutboxStore, publisher OutboxPublisher, batchSize int) *OutboxProcessor
func (p *OutboxProcessor) ProcessBatch(ctx context.Context) (int, error)
func (p *OutboxProcessor) Run(ctx context.Context, interval time.Duration)
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/outbox/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export type OutboxStatus = 'PENDING' | 'PROCESSING' | 'DELIVERED' | 'FAILED' | 'DEAD_LETTER';

export type OutboxErrorCode = 'STORE_ERROR' | 'PUBLISH_ERROR' | 'SERIALIZATION_ERROR' | 'NOT_FOUND';

export interface OutboxMessage {
  id: string;
  topic: string;
  partitionKey: string;
  payload: string;
  status: OutboxStatus;
  retryCount: number;
  maxRetries: number;
  lastError: string | null;
  createdAt: Date;
  processAfter: Date;
}

export function createOutboxMessage(topic: string, partitionKey: string, payload: string): OutboxMessage;
export function markProcessing(msg: OutboxMessage): void;
export function markDelivered(msg: OutboxMessage): void;
export function markFailed(msg: OutboxMessage, error: string): void;
export function isProcessable(msg: OutboxMessage): boolean;
export function canTransitionTo(from: OutboxStatus, to: OutboxStatus): boolean;

export interface OutboxStore {
  save(msg: OutboxMessage): Promise<void>;
  fetchPending(limit: number): Promise<OutboxMessage[]>;
  update(msg: OutboxMessage): Promise<void>;
  deleteDelivered(olderThanDays: number): Promise<number>;
}

export interface OutboxPublisher {
  publish(msg: OutboxMessage): Promise<void>;
}

export class OutboxProcessor {
  constructor(store: OutboxStore, publisher: OutboxPublisher, batchSize?: number); // デフォルト: 100
  processBatch(): Promise<number>;
  run(intervalMs: number, signal?: AbortSignal): Promise<void>;
}

export class OutboxError extends Error {
  constructor(code: OutboxErrorCode, message?: string);
}
```

**カバレッジ目標**: 85%以上

## Dart 実装

**配置先**: `regions/system/library/dart/outbox/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `uuid: ^4.4.0`, `lints: ^4.0.0` (dev)

**主要 API**:

```dart
enum OutboxStatus { pending, processing, delivered, failed, deadLetter }

enum OutboxErrorCode { storeError, publishError, serializationError, notFound }

class OutboxMessage {
  final String id;
  final String topic;
  final String partitionKey;
  final String payload;
  OutboxStatus status;
  int retryCount;
  int maxRetries;
  String? lastError;
  final DateTime createdAt;
  DateTime processAfter;

  void markProcessing();
  void markDelivered();
  void markFailed(String error);
  bool get isProcessable;
}

OutboxMessage createOutboxMessage(String topic, String partitionKey, String payload);
bool canTransitionTo(OutboxStatus from, OutboxStatus to);

abstract class OutboxStore {
  Future<void> save(OutboxMessage msg);
  Future<List<OutboxMessage>> fetchPending(int limit);
  Future<void> update(OutboxMessage msg);
  Future<int> deleteDelivered(int olderThanDays);
}

abstract class OutboxPublisher {
  Future<void> publish(OutboxMessage msg);
}

class OutboxProcessor {
  OutboxProcessor(OutboxStore store, OutboxPublisher publisher, {int batchSize = 100});
  Future<int> processBatch();
  Future<void> run(Duration interval, {Future<void>? stopSignal});
}

class OutboxError implements Exception {
  final OutboxErrorCode code;
  final String? message;
  final Object? cause;
}
```

**カバレッジ目標**: 85%以上

## 設計ノート: OutboxProcessor の run() メソッドに関する言語差異

Rust の `OutboxProcessor` は `process_batch()` のみ提供し、`run()` メソッドは持たない。呼び出し側が `tokio::spawn` + `loop` + `tokio::time::sleep` で定期実行を制御する設計である。Go/TypeScript/Dart は `run()` メソッドで定期実行をサポートする。

- **Rust**: `process_batch()` のみ。定期実行は呼び出し側の責務
- **Go**: `ProcessBatch(ctx)` + `Run(ctx, interval)`
- **TypeScript**: `processBatch()` + `run(intervalMs, signal?)`
- **Dart**: `processBatch()` + `run(interval, {stopSignal?})`

## 設計ノート: OutboxError の言語間パターン差異

エラーの種類（StoreError・PublishError・SerializationError・NotFound）は全4言語で統一されているが、表現パターンは言語特性に合わせて異なる。

- **Rust**: `enum OutboxError { StoreError(String), PublishError(String), SerializationError(String), NotFound(String) }`
- **Go**: `struct OutboxError { Kind OutboxErrorKind, Message string, Err error }` + `OutboxErrorKind` iota enum + ヘルパーコンストラクタ（`NewStoreError`, `NewPublishError` 等）
- **TypeScript**: `class OutboxError extends Error { code: OutboxErrorCode }` + `type OutboxErrorCode = 'STORE_ERROR' | 'PUBLISH_ERROR' | 'SERIALIZATION_ERROR' | 'NOT_FOUND'`
- **Dart**: `class OutboxError implements Exception { code: OutboxErrorCode, message: String?, cause: Object? }` + `enum OutboxErrorCode { storeError, publishError, serializationError, notFound }`

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-config設計](../config/config.md) — config ライブラリ
- [system-library-telemetry設計](../observability/telemetry.md) — telemetry ライブラリ
- [system-library-authlib設計](../auth-security/authlib.md) — authlib ライブラリ
- [system-library-messaging設計](messaging.md) — k1s0-messaging ライブラリ
- [system-library-kafka設計](kafka.md) — k1s0-kafka ライブラリ
- [system-library-correlation設計](../observability/correlation.md) — k1s0-correlation ライブラリ
- [system-library-schemaregistry設計](../data/schemaregistry.md) — k1s0-schemaregistry ライブラリ

---
