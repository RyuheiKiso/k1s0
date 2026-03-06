# k1s0-messaging ライブラリ設計

## 概要

Kafka ベースのイベント送受信を共通化するライブラリ。  
`EventProducer` / `EventConsumer` / `EventEnvelope` / `EventMetadata` を言語横断でそろえ、サービス間イベント連携の実装差分を最小化する。

## 配置

- Rust: `regions/system/library/rust/messaging/`
- Go: `regions/system/library/go/messaging/`
- TypeScript: `regions/system/library/typescript/messaging/`
- Dart: `regions/system/library/dart/messaging/`

## 共通 API

| API | 目的 |
| --- | --- |
| `EventMetadata` | イベント ID、種別、相関 ID、トレース ID、発行時刻、発行元、`schema_version` を保持 |
| `EventEnvelope` | `topic` / `key` / `payload` / `headers` とメタデータを保持 |
| `EventProducer.publish` | 単一イベントを発行 |
| `EventProducer.publishBatch` | 複数イベントをバッチ発行 |
| `EventConsumer` | イベント受信（言語ごとに pull/push モデルが異なる） |
| `MessagingError` | メッセージング操作（publish/subscribe/deserialize 等）のラップエラー |
| `NoOpEventProducer` | テスト向け no-op 実装 |

## 言語差異

### EventConsumer モデル

- Rust: pull モデル (`receive` + `commit`)
- Go / TypeScript / Dart: push モデル (`subscribe(topic, handler)`)

### EventHandler の有無

- `EventHandler` 型は Go / TypeScript / Dart のみ定義
- Rust は pull モデルのため `EventHandler` を持たない

### `ConsumedMessage` の参照方法（Rust）

`ConsumedMessage` は `consumer.rs` で `pub struct` として公開される。  
`lib.rs` のトップレベル再エクスポート対象ではないため、次のパスで参照する。

```rust
use k1s0_messaging::consumer::ConsumedMessage;
```

### `EventEnvelope` の metadata フィールド

- Rust: `EventEnvelope` に `metadata: HashMap<String, String>` を持つ
- Go / TypeScript / Dart: `EventEnvelope` は `EventMetadata` を持つ

実装上の構造は異なるが、任意メタデータをエンベロープ単位で保持できる点は共通。

### Go `EventEnvelope` の `Headers`

Go 実装の `EventEnvelope` には `Headers map[string]string` フィールドがあり、Kafka メッセージヘッダーを付与できる。

```go
event := messaging.EventEnvelope{
    Metadata: messaging.NewEventMetadata("user.created.v1", "corr-123", "user-service"),
    Topic:    "k1s0.system.user.created.v1",
    Key:      "user-123",
    Payload:  map[string]any{"id": "user-123"},
    Headers: map[string]string{
        "x-tenant-id": "tenant-abc",
        "x-trace-id":  "trace-123",
    },
}
```

### Go `EventMetadata.SchemaVersion`

Go 実装の `EventMetadata` は `SchemaVersion int32` を持つ。`NewEventMetadata(...)` で生成した場合のデフォルト値は `1`。

### Go `EventMetadata.WithTraceId`

Go 実装は値レシーバのビルダーメソッド `WithTraceId(traceId string)` を提供する。`TraceId` をセットした新しい `EventMetadata` を返す。

### Go `MessagingError`

```go
type MessagingError struct {
    Op  string // 失敗した操作名（publish / subscribe / decode など）
    Err error  // 元エラー
}
```

### metadata 生成 API の `correlationId` 必須性

- Go: `NewEventMetadata(eventType, correlationId, source)` で必須
- TypeScript: `createEventMetadata(eventType, source, correlationId, traceId?)` で `correlationId` は必須、`traceId` はオプション（未指定時は UUID 自動生成）
- Dart: `EventMetadata.create(eventType, source, {required correlationId, traceId?})` で必須
- Rust: `EventMetadata::new(...).with_correlation_id(...)` のビルダーパターン（任意）

## 主要シグネチャ（抜粋）

### Rust

```rust
#[async_trait]
pub trait EventProducer {
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), MessagingError>;
    async fn publish_batch(&self, envelopes: Vec<EventEnvelope>) -> Result<(), MessagingError>;
}

#[async_trait]
pub trait EventConsumer {
    async fn receive(&self) -> Result<ConsumedMessage, MessagingError>;
    async fn commit(&self, msg: &ConsumedMessage) -> Result<(), MessagingError>;
}
```

### Go

```go
type EventProducer interface {
    Publish(ctx context.Context, event EventEnvelope) error
    PublishBatch(ctx context.Context, events []EventEnvelope) error
    Close() error
}

type EventHandler func(ctx context.Context, event EventEnvelope) error
```

### TypeScript

```ts
export interface EventProducer {
  publish(event: EventEnvelope): Promise<void>;
  publishBatch(events: EventEnvelope[]): Promise<void>;
  close(): Promise<void>;
}

export type EventHandler = (event: EventEnvelope) => Promise<void>;
```

### Dart

```dart
abstract class EventProducer {
  Future<void> publish(EventEnvelope event);
  Future<void> publishBatch(List<EventEnvelope> events);
  Future<void> close();
}

typedef EventHandler = Future<void> Function(EventEnvelope event);
```

## Rust 固有の型

### ConsumerConfig

```rust
pub struct ConsumerConfig {
    pub group_id: String,
    pub topics: Vec<String>,
    pub auto_commit: bool,
    pub session_timeout_ms: u64,
}
```

### MessagingConfig

```rust
pub struct MessagingConfig {
    pub brokers: Vec<String>,
    pub security_protocol: Option<String>,
    pub timeout_ms: u64,
    pub batch_size: usize,
}

impl MessagingConfig {
    /// brokers を `,` 区切りの単一文字列として返す。
    pub fn brokers_string(&self) -> String
}
```

### MessagingError バリアント

Rust の `MessagingError` は 9 バリアントの enum として定義される:

| バリアント | 説明 |
|-----------|------|
| `ProducerError` | プロデューサー操作の失敗 |
| `ConsumerError` | コンシューマー操作の失敗 |
| `SerializationError` | シリアライズ失敗 |
| `DeserializationError` | デシリアライズ失敗 |
| `ConnectionError` | ブローカー接続失敗 |
| `TimeoutError` | 操作タイムアウト |
| `PublishError` | メッセージ発行失敗 |
| `ConsumeError` | メッセージ受信失敗 |
| `CommitError` | オフセットコミット失敗 |

### EventMetadata 追加メソッド

```rust
impl EventMetadata {
    /// タイムスタンプを Unix ミリ秒に変換する。
    pub fn to_unix_millis(&self) -> i64

    /// Unix ミリ秒から DateTime<Utc> を生成する。
    pub fn from_unix_millis(millis: i64) -> DateTime<Utc>
}
```

### EventEnvelope::json コンストラクタ

```rust
impl EventEnvelope {
    /// payload を JSON シリアライズして EventEnvelope を生成する。
    pub fn json(topic: &str, key: &str, payload: &impl Serialize) -> Result<Self, MessagingError>
}
```

### Cargo features

Rust 実装では以下の Cargo features が利用可能:

| Feature | 説明 |
|---------|------|
| `kafka` | Kafka バックエンド（rdkafka ベースの `KafkaEventProducer` / `KafkaEventConsumer`） |
| `mock` | テスト用モック実装（`NoOpEventProducer` 等） |
| `protobuf` | Protocol Buffers シリアライズサポート |

## 関連ドキュメント

- [system-library 概要](../_common/概要.md)
- [k1s0-kafka 設計](kafka.md)
- [k1s0-outbox 設計](outbox.md)
- [system API 設計](../../architecture/api/API設計.md)
