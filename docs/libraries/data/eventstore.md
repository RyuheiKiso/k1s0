# k1s0-eventstore ライブラリ設計

## 概要

イベントソーシング向けイベント永続化基盤ライブラリ。`EventStore` トレイトにより Append-only なイベントストリームへの `append`（追記）・`load`（読み込み）・`load_from`（バージョン指定読み込み）を提供する。スナップショット対応で大量イベントの再生コストを抑制する。楽観的ロック（`expected_version`）による競合制御を内包する。

**配置先**: `regions/system/library/rust/eventstore/`

## 公開 API

最小共通 API（全 4 言語）:

| メソッド | 戻り値 | 説明 |
|---------|--------|------|
| `append(stream_id, events, expected_version?)` | `version: u64/uint64/number/int` | イベント追記。バージョン競合時はエラー |
| `load(stream_id)` | `Vec<EventEnvelope>` | ストリームの全イベント取得 |
| `load_from(stream_id, from_version)` | `Vec<EventEnvelope>` | 指定バージョン以降のイベント取得 |
| `exists(stream_id)` | `bool` | ストリームが存在するか確認 |
| `current_version(stream_id)` | `u64/uint64/number/int` | 現在のバージョン取得 |

Rust 公開型:

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `EventStore` | トレイト | イベント永続化・読み込みの抽象インターフェース |
| `InMemoryEventStore` | 構造体 | テスト用インメモリ実装 |
| `InMemorySnapshotStore` | 構造体 | テスト用インメモリスナップショット実装 |
| `PostgresEventStore` | 構造体 | PostgreSQL バックエンド実装（feature = "postgres" で有効） |
| `MockEventStore` | 構造体 | テスト用モック（feature = "mock" で有効） |
| `EventEnvelope` | 構造体 | イベント本体 + メタデータ（stream_id・version・event_type・payload・metadata・**recorded_at**） |
| `StreamId` | 構造体 | ストリーム識別子（単純な文字列ラッパー） |
| `Snapshot` | 構造体 | 集約状態のスナップショット（stream_id・version・state・created_at） |
| `SnapshotStore` | トレイト | スナップショット保存・読み込みの抽象インターフェース |
| `EventStoreError` | enum | バージョン競合・デシリアライゼーションエラー等 |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-eventstore"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]
postgres = ["dep:sqlx"]

[dependencies]
async-trait = "0.1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "macros"] }
tracing = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
mockall = { version = "0.13", optional = true }
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid", "json"], optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `k1s0-eventstore = { path = "../../system/library/rust/eventstore", features = ["postgres"] }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
eventstore/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── store.rs        # EventStore トレイト・MockEventStore
│   ├── envelope.rs     # EventEnvelope
│   ├── stream.rs       # StreamId
│   ├── memory.rs       # InMemoryEventStore・InMemorySnapshotStore
│   ├── postgres.rs     # PostgresEventStore（feature = "postgres"）
│   ├── snapshot.rs     # Snapshot・SnapshotStore トレイト
│   └── error.rs        # EventStoreError
├── tests/
│   └── eventstore_test.rs
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_eventstore::{EventStore, EventEnvelope, InMemoryEventStore, StreamId};

let store = InMemoryEventStore::default();
let stream_id = StreamId::new("order-uuid-1234");

// イベント追記（楽観的ロック: 現在バージョンが 0 であることを期待）
let events = vec![
    EventEnvelope::new(
        &stream_id,
        1,
        "OrderPlaced",
        serde_json::json!({"order_id": "order-uuid-1234", "total": 100.0}),
    ),
];
let version = store.append(&stream_id, events, Some(0)).await.unwrap();
// version == 1

// 全イベント読み込み
let envelopes = store.load(&stream_id).await.unwrap();

// バージョン 2 以降のみ読み込み
let delta = store.load_from(&stream_id, 2).await.unwrap();
```

## Go 実装

**配置先**: `regions/system/library/go/eventstore/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/lib/pq`（PostgreSQL ドライバー）。InMemory 実装は標準ライブラリのみ。

**主要インターフェース**:

```go
type StreamId struct {
    // unexported value field
}

func NewStreamId(value string) StreamId
func (s StreamId) String() string

type EventEnvelope struct {
    EventID    string
    StreamID   string
    Version    uint64
    EventType  string
    Payload    json.RawMessage
    Metadata   json.RawMessage
    RecordedAt time.Time
}

func NewEventEnvelope(streamID StreamId, version uint64, eventType string, payload json.RawMessage) *EventEnvelope

type Snapshot struct {
    StreamID  string
    Version   uint64
    State     json.RawMessage
    CreatedAt time.Time
}

type EventStore interface {
    Append(ctx context.Context, streamID StreamId, events []*EventEnvelope, expectedVersion *uint64) (uint64, error)
    Load(ctx context.Context, streamID StreamId) ([]*EventEnvelope, error)
    LoadFrom(ctx context.Context, streamID StreamId, fromVersion uint64) ([]*EventEnvelope, error)
    Exists(ctx context.Context, streamID StreamId) (bool, error)
    CurrentVersion(ctx context.Context, streamID StreamId) (uint64, error)
}

type SnapshotStore interface {
    SaveSnapshot(ctx context.Context, snapshot *Snapshot) error
    LoadSnapshot(ctx context.Context, streamID StreamId) (*Snapshot, error)
}

// エラー型
type EventStoreError struct {
    Code    string
    Message string
}

func (e *EventStoreError) Error() string
func NewVersionConflictError(expected, actual uint64) *EventStoreError
func NewStreamNotFoundError(streamID string) *EventStoreError

// InMemory 実装
func NewInMemoryEventStore() *InMemoryEventStore    // implements EventStore
func NewInMemorySnapshotStore() *InMemorySnapshotStore  // implements SnapshotStore

// PostgreSQL 実装
func NewPostgresEventStore(databaseURL string) (*PostgresEventStore, error)
func NewPostgresEventStoreFromDB(db *sql.DB) *PostgresEventStore
func (s *PostgresEventStore) Migrate(ctx context.Context) error  // イベントテーブル作成
func (s *PostgresEventStore) Close() error                       // DB 接続クローズ
// PostgresEventStore implements EventStore
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/eventstore/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export type StreamId = string;

export interface EventEnvelope {
  eventId: string;
  streamId: string;
  version: number;
  eventType: string;
  payload: unknown;
  metadata?: unknown;
  recordedAt: Date;
}

export interface Snapshot {
  streamId: string;
  version: number;
  state: unknown;
  createdAt: Date;
}

export interface EventStore {
  append(streamId: StreamId, events: Omit<EventEnvelope, 'eventId' | 'version' | 'recordedAt'>[], expectedVersion?: number): Promise<number>;
  load(streamId: StreamId): Promise<EventEnvelope[]>;
  loadFrom(streamId: StreamId, fromVersion: number): Promise<EventEnvelope[]>;
  exists(streamId: StreamId): Promise<boolean>;
  currentVersion(streamId: StreamId): Promise<number>;
}

export interface SnapshotStore {
  saveSnapshot(snapshot: Snapshot): Promise<void>;
  loadSnapshot(streamId: StreamId): Promise<Snapshot | null>;
}

export class VersionConflictError extends Error {
  constructor(public readonly expected: number, public readonly actual: number);
}

export class InMemoryEventStore implements EventStore { ... }
export class InMemorySnapshotStore implements SnapshotStore { ... }

// PostgreSQL 実装（pg パッケージ使用）
export class PostgresEventStore implements EventStore {
  constructor(pool: Pool);
  async migrate(): Promise<void>;  // イベントテーブル作成
}

export class PostgresSnapshotStore implements SnapshotStore {
  constructor(pool: Pool);
  async migrate(): Promise<void>;  // スナップショットテーブル作成
}
```

**カバレッジ目標**: 85%以上

## Dart 実装

**配置先**: `regions/system/library/dart/eventstore/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要インターフェース**:

```dart
abstract class EventStore {
  Future<int> append(
    String streamId,
    List<NewEvent> events, {
    int? expectedVersion,
  });
  Future<List<EventEnvelope>> load(String streamId);
  Future<List<EventEnvelope>> loadFrom(String streamId, int fromVersion);
  Future<bool> exists(String streamId);
  Future<int> currentVersion(String streamId);
}

class EventEnvelope {
  final String eventId;
  final String streamId;
  final int version;
  final String eventType;
  final Object? payload;
  final Object? metadata;
  final DateTime recordedAt;
}

class NewEvent {
  final String streamId;
  final String eventType;
  final Object? payload;
  final Object? metadata;
}

class VersionConflictError implements Exception {
  final int expected;
  final int actual;
}

class Snapshot {
  final String streamId;
  final int version;
  final Object? state;
  final DateTime createdAt;
}

abstract class SnapshotStore {
  Future<void> saveSnapshot(Snapshot snapshot);
  Future<Snapshot?> loadSnapshot(String streamId);
}

class InMemoryEventStore implements EventStore { ... }
class InMemorySnapshotStore implements SnapshotStore { ... }

// PostgreSQL 実装（postgres パッケージ使用）
class PostgresEventStore implements EventStore {
  PostgresEventStore(Connection conn);
  Future<void> migrate();  // イベントテーブル作成
}

class PostgresSnapshotStore implements SnapshotStore {
  PostgresSnapshotStore(Connection conn);
  Future<void> migrate();  // スナップショットテーブル作成
}
```

**カバレッジ目標**: 85%以上

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-messaging設計](../messaging/messaging.md) — Kafka イベント発行との連携
- [system-library-outbox設計](../messaging/outbox.md) — トランザクショナルアウトボックスパターン
- [system-saga-server設計](../../servers/saga/server.md) — Saga オーケストレーションとのイベント連携
- [メッセージング設計](../../architecture/messaging/メッセージング設計.md) — Kafka トピック設計
- [system-database設計](../../servers/_common/database.md) — PostgreSQL スキーマ設計
- [コーディング規約](../../architecture/conventions/コーディング規約.md) — 命名規則・Linter 設定
