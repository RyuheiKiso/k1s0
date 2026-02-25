# k1s0-eventstore ライブラリ設計

## 概要

イベントソーシング向けイベント永続化・再生基盤ライブラリ。`EventStore` トレイトにより Append-only なイベントストリームへの `append`（追記）・`load`（読み込み）・`replay`（再生）を提供する。スナップショット対応で大量イベントの再生コストを抑制する。楽観的ロックによる競合制御を内包し、PostgreSQL をバックエンドとする。

**配置先**: `regions/system/library/rust/eventstore/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `EventStore` | トレイト | イベント永続化・読み込み・再生の抽象インターフェース |
| `PostgresEventStore` | 構造体 | PostgreSQL バックエンド実装 |
| `InMemoryEventStore` | 構造体 | テスト用インメモリ実装 |
| `MockEventStore` | 構造体 | テスト用モック（feature = "mock" で有効） |
| `EventEnvelope` | 構造体 | イベント本体 + メタデータ（stream_id・version・event_type・payload・metadata・occurred_at） |
| `StreamId` | 構造体 | ストリーム識別子（aggregate_type + aggregate_id） |
| `EventVersion` | 構造体 | ストリームバージョン（楽観的ロック用） |
| `Snapshot` | 構造体 | 集約状態のスナップショット（stream_id・version・state_payload・taken_at） |
| `SnapshotStore` | トレイト | スナップショット保存・読み込みの抽象インターフェース |
| `PostgresSnapshotStore` | 構造体 | PostgreSQL スナップショットストア実装 |
| `EventStoreConfig` | 構造体 | データベース URL・スキーマ・スナップショット間隔設定 |
| `EventStoreError` | enum | バージョン競合・デシリアライゼーションエラー等 |
| `ConcurrencyError` | 構造体 | 楽観的ロック競合（expected_version と actual_version を保持） |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-eventstore"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]
postgres = ["sqlx"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
sqlx = { version = "0.8", features = ["runtime-tokio-native-tls", "postgres", "uuid", "chrono", "json"], optional = true }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
testcontainers-modules = { version = "0.11", features = ["postgres"] }
```

**Cargo.toml への追加行**:

```toml
k1s0-eventstore = { path = "../../system/library/rust/eventstore", features = ["postgres"] }
# テスト時にモックを有効化する場合:
k1s0-eventstore = { path = "../../system/library/rust/eventstore", features = ["mock"] }
```

**モジュール構成**:

```
eventstore/
├── src/
│   ├── lib.rs              # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── store.rs            # EventStore トレイト・PostgresEventStore・InMemoryEventStore・MockEventStore
│   ├── envelope.rs         # EventEnvelope・StreamId・EventVersion
│   ├── snapshot.rs         # Snapshot・SnapshotStore トレイト・PostgresSnapshotStore
│   ├── config.rs           # EventStoreConfig
│   └── error.rs            # EventStoreError・ConcurrencyError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_eventstore::{
    EventStore, EventEnvelope, EventStoreConfig, PostgresEventStore,
    SnapshotStore, PostgresSnapshotStore, StreamId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
enum OrderEvent {
    OrderPlaced { order_id: String, total: f64 },
    ItemAdded { item_id: String, quantity: u32 },
    OrderShipped { tracking_number: String },
}

let config = EventStoreConfig::new("postgres://localhost/k1s0")
    .with_schema("eventstore")
    .with_snapshot_interval(50); // 50 イベントごとにスナップショット

let store = PostgresEventStore::new(config.clone()).await.unwrap();
let snapshot_store = PostgresSnapshotStore::new(config).await.unwrap();

let stream_id = StreamId::new("order", "order-uuid-1234");

// イベント追記（楽観的ロック: 現在バージョンが 2 であることを期待）
let events = vec![
    EventEnvelope::new(
        &stream_id,
        "OrderShipped",
        &OrderEvent::OrderShipped {
            tracking_number: "TRACK-001".to_string(),
        },
    ).unwrap(),
];
store.append(&stream_id, events, Some(2)).await.unwrap();

// イベント読み込み（バージョン 0 から全件）
let envelopes = store.load(&stream_id, 0, None).await.unwrap();

// スナップショット保存
let state = reconstruct_order_state(&envelopes);
snapshot_store.save(&stream_id, 3, &state).await.unwrap();

// スナップショットから再生（最新スナップショット以降のイベントのみロード）
if let Some(snap) = snapshot_store.load_latest(&stream_id).await.unwrap() {
    let delta = store.load(&stream_id, snap.version + 1, None).await.unwrap();
    // snap.state_payload から状態を復元し、delta を適用
}
```

## Go 実装

**配置先**: `regions/system/library/go/eventstore/`

```
eventstore/
├── eventstore.go       # EventStore・SnapshotStore インターフェース・StreamId・EventEnvelope・Snapshot・EventStoreError
├── memory.go           # テスト用インメモリ実装
├── postgres.go         # PostgreSQL 実装
├── eventstore_test.go
├── postgres_test.go
├── go.mod
└── go.sum
```

**依存関係**: `github.com/jackc/pgx/v5 v5.7.2`

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
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/eventstore/`

```
eventstore/
├── package.json        # "@k1s0/eventstore", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # EventStore, EventEnvelope, StreamId, Snapshot, SnapshotStore
└── __tests__/
    └── eventstore.test.ts
```

**主要 API**:

```typescript
export interface StreamId {
  aggregateType: string;
  aggregateId: string;
}

export interface EventEnvelope<T = unknown> {
  id: string;
  streamId: StreamId;
  version: number;
  eventType: string;
  payload: T;
  metadata: Record<string, unknown>;
  occurredAt: string;
}

export interface Snapshot<S = unknown> {
  streamId: StreamId;
  version: number;
  state: S;
  takenAt: string;
}

export interface EventStore {
  append(streamId: StreamId, events: Omit<EventEnvelope, 'id' | 'version' | 'occurredAt'>[], expectedVersion?: number): Promise<void>;
  load(streamId: StreamId, fromVersion?: number, toVersion?: number): Promise<EventEnvelope[]>;
  exists(streamId: StreamId): Promise<boolean>;
}

export interface SnapshotStore {
  save<S>(streamId: StreamId, version: number, state: S): Promise<void>;
  loadLatest<S>(streamId: StreamId): Promise<Snapshot<S> | null>;
}

export class ConcurrencyError extends Error {
  constructor(
    public readonly expectedVersion: number,
    public readonly actualVersion: number
  ) { super(`Concurrency conflict: expected ${expectedVersion}, got ${actualVersion}`); }
}
```

**カバレッジ目標**: 85%以上

## Dart 実装

**配置先**: `regions/system/library/dart/eventstore/`

```
eventstore/
├── pubspec.yaml        # k1s0_eventstore, postgres: ^3.5.0
├── analysis_options.yaml
├── lib/
│   ├── eventstore.dart
│   └── src/
│       ├── store.dart          # EventStore abstract・PostgresEventStore
│       ├── envelope.dart       # EventEnvelope・StreamId・EventVersion
│       ├── snapshot.dart       # Snapshot・SnapshotStore abstract
│       ├── config.dart         # EventStoreConfig
│       └── error.dart          # EventStoreError・ConcurrencyError
└── test/
    └── eventstore_test.dart
```

**カバレッジ目標**: 85%以上

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-messaging設計](system-library-messaging設計.md) — Kafka イベント発行との連携
- [system-library-outbox設計](system-library-outbox設計.md) — トランザクショナルアウトボックスパターン
- [system-saga-server設計](system-saga-server設計.md) — Saga オーケストレーションとのイベント連携
- [メッセージング設計](メッセージング設計.md) — Kafka トピック設計
- [system-database設計](system-database設計.md) — PostgreSQL スキーマ設計
- [コーディング規約](コーディング規約.md) — 命名規則・Linter 設定
