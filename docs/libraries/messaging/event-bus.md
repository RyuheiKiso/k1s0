# k1s0-event-bus ライブラリ設計

## 概要

インプロセスのドメインイベントバスライブラリ。`publish`/`subscribe` によるイベント駆動アーキテクチャをサポートし、eventstore ライブラリと組み合わせてドメインイベントの永続化と配信を行う。tokio::broadcast チャネルベースの実装で、サービス内の集約間・ユースケース間の疎結合な通信を実現する。

**配置先**: `regions/system/library/rust/event-bus/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `EventBus` | 構造体 | インプロセスイベントバス（DDD パターン対応、設定・タイムアウト管理付き）|
| `DomainEvent` | トレイト | ドメインイベントインターフェース（event_type・aggregate_id・occurred_at）|
| `EventHandler` | トレイト | レガシーイベントハンドラー（Rust: non-generic、`Event` 型固定。Go/TS/Dart: generic） |
| `DomainEventHandler<T>` | トレイト | ジェネリックなドメインイベントハンドラー（Rust のみ。EventBus とは直接統合されていない） |
| `EventSubscription` | 構造体 | サブスクリプション管理（Rust: Drop で自動解除）|
| `EventBusConfig` | 構造体 | チャネルバッファサイズ・ハンドラータイムアウト設定 |
| `EventBusError` | enum | `PublishFailed`・`HandlerFailed`・`ChannelClosed` |
| `Event` | 構造体 | 基本イベント構造体（`DomainEvent` 実装、レガシー互換）。全言語に存在 |
| `InMemoryEventBus` | 構造体 | レガシーイベントバス（後方互換性のため維持）。全言語に存在 |
| `MockEventHandler` | 構造体 | テスト用モックハンドラー（Rust のみ、`feature = "mock"`） |

> **設計ノート**: Rust の `EventHandler` は non-generic で `Event` 型を固定的に受け取る（`fn handle(&self, event: Event)`）。ジェネリック版は `DomainEventHandler<T: DomainEvent>` として分離されているが、`EventBus` の `subscribe` は `Arc<dyn EventHandler>` を受け取るため、`DomainEventHandler` は EventBus と直接統合されていない。Go/TS/Dart の `EventHandler` はジェネリック（`EventHandler[T]` / `EventHandler<T>`）で、各言語の `EventBus` と直接連携する。

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-event-bus"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time", "macros"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `k1s0-event-bus = { path = "../../system/library/rust/event-bus" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
event-bus/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── bus.rs          # EventBus
│   ├── event.rs        # DomainEvent トレイト
│   ├── handler.rs      # EventHandler トレイト・EventSubscription
│   ├── config.rs       # EventBusConfig
│   └── error.rs        # EventBusError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_event_bus::{EventBus, EventBusConfig, EventHandler, Event, EventBusError};
use async_trait::async_trait;
use std::sync::Arc;

// EventHandler 実装（non-generic、Event 型固定）
struct OrderCreatedHandler;

#[async_trait]
impl EventHandler for OrderCreatedHandler {
    fn event_type(&self) -> &str { "order.created" }

    async fn handle(&self, event: Event) -> Result<(), EventBusError> {
        println!("注文作成通知: aggregate_id={}", event.aggregate_id);
        Ok(())
    }
}

// バス初期化とイベント発行
let config = EventBusConfig::new()
    .buffer_size(1024)
    .handler_timeout(std::time::Duration::from_secs(5));

let bus = EventBus::new(config);

// ハンドラー登録（Arc<dyn EventHandler> を渡す。EventSubscription が Drop されると自動解除）
let _subscription = bus.subscribe(Arc::new(OrderCreatedHandler)).await;

// イベント発行（Event 構造体を使用）
let event = Event::with_aggregate_id(
    "order.created".to_string(),
    "ORD-001".to_string(),
    serde_json::json!({"user_id": "USR-123", "total_amount": 10000}),
);
bus.publish(event).await?;
```

> **デフォルトタイムアウトの言語間差異**: Rust のデフォルトハンドラータイムアウトは 30 秒、TS/Dart は 5 秒（5000ms）。

## Go 実装

**配置先**: `regions/system/library/go/event-bus/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `github.com/stretchr/testify v1.11.1`

**主要インターフェース**:

```go
// --- DDD ドメインイベント ---

type DomainEvent interface {
    EventType() string
    AggregateID() string
    OccurredAt() time.Time
}

type EventHandler[T DomainEvent] interface {
    Handle(ctx context.Context, event T) error
}

// EventHandlerFunc はハンドラー関数を EventHandler インターフェースに変換するアダプター。
type EventHandlerFunc[T DomainEvent] func(ctx context.Context, event T) error

func (f EventHandlerFunc[T]) Handle(ctx context.Context, event T) error

// --- EventBusError ---

type ErrorKind int

const (
    PublishFailed ErrorKind = iota
    HandlerFailed
    ChannelClosed
)

type EventBusError struct {
    Kind    ErrorKind
    Message string
    Err     error
}

func (e *EventBusError) Error() string
func (e *EventBusError) Unwrap() error

// --- EventBusConfig ---

type EventBusConfig struct {
    BufferSize     int
    HandlerTimeout time.Duration
}

func DefaultEventBusConfig() EventBusConfig

// --- EventBus (DDD パターン) ---

type EventBus struct { /* ... */ }

func NewEventBus(config EventBusConfig) *EventBus

// Subscribe はワイルドカード登録（"*" にマッチ）し、型アサーションでフィルタリングする。
func Subscribe[T DomainEvent](bus *EventBus, handler EventHandler[T]) *EventSubscription

// SubscribeType は指定したイベントタイプにハンドラーを登録する。
func SubscribeType[T DomainEvent](bus *EventBus, eventType string, handler EventHandler[T]) *EventSubscription

func Publish[T DomainEvent](ctx context.Context, bus *EventBus, event T) error

type EventSubscription struct { /* ... */ }
func (s *EventSubscription) Unsubscribe()

// --- レガシー API（後方互換性のため維持） ---

type Event struct {
    ID        string         `json:"id"`
    EventType string         `json:"event_type"`
    Payload   map[string]any `json:"payload"`
    Timestamp time.Time      `json:"timestamp"`
}

type Handler func(ctx context.Context, event Event) error

type LegacyEventBus interface {
    Subscribe(eventType string, handler Handler)
    Publish(ctx context.Context, event Event) error
    Unsubscribe(eventType string)
}

type InMemoryBus struct { /* ... */ }

func New() *InMemoryBus
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/event-bus/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export interface DomainEvent {
  readonly eventType: string;
  readonly aggregateId: string;
  readonly occurredAt: Date;
}

// レガシー互換の Event インターフェース
export interface Event extends DomainEvent {
  id: string;
  payload: Record<string, unknown>;
  timestamp: string;
}

export interface EventHandler<T extends DomainEvent = DomainEvent> {
  handle(event: T): Promise<void>;
}

export interface EventSubscription {
  readonly eventType: string;
  unsubscribe(): void;
}

export interface EventBusConfig {
  bufferSize?: number;
  handlerTimeoutMs?: number;
}

export type EventBusErrorCode = 'PUBLISH_FAILED' | 'HANDLER_FAILED' | 'CHANNEL_CLOSED';

export class EventBusError extends Error {
  public readonly code: EventBusErrorCode;
  constructor(message: string, code: EventBusErrorCode);
}

export class EventBus {
  constructor(config?: EventBusConfig);
  publish<T extends DomainEvent>(event: T): Promise<void>;
  subscribe<T extends DomainEvent>(
    eventType: string,
    handler: EventHandler<T>
  ): EventSubscription;
  close(): void;
}

// レガシー互換の InMemoryEventBus
export class InMemoryEventBus {
  constructor(config?: EventBusConfig);
  subscribe(eventType: string, handler: (event: Event) => Promise<void>): void;
  unsubscribe(eventType: string): void;
  publish(event: Event): Promise<void>;
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/event_bus/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

> **注**: Dart のパッケージ命名慣習によりディレクトリ名はアンダースコア `event_bus/` を使用（他言語はハイフン `event-bus/`）。

**主要 API**:

```dart
// DomainEvent - ドメインイベントの基底クラス
abstract class DomainEvent {
  String get eventType;
  String get aggregateId;
  DateTime get occurredAt;
}

// Event - レガシー互換のイベントクラス（DomainEvent を実装）
class Event implements DomainEvent {
  final String id;
  final String eventType;
  final String aggregateId;
  final DateTime occurredAt;
  final Map<String, dynamic> payload;
  final DateTime timestamp;

  const Event({
    required this.id,
    required this.eventType,
    required this.payload,
    required this.timestamp,
    this.aggregateId = '',
    DateTime? occurredAt,
  });
}

// EventHandler - 型付きイベントハンドラー
abstract class EventHandler<T extends DomainEvent> {
  Future<void> handle(T event);
}

// EventBusConfig
class EventBusConfig {
  final int bufferSize;       // デフォルト: 1024
  final int handlerTimeoutMs; // デフォルト: 5000
  const EventBusConfig({this.bufferSize = 1024, this.handlerTimeoutMs = 5000});
}

// EventBusErrorCode
enum EventBusErrorCode { publishFailed, handlerFailed, channelClosed }

// EventBusError
class EventBusError implements Exception {
  final String message;
  final EventBusErrorCode code;
  const EventBusError(this.message, this.code);
}

// EventSubscription
class EventSubscription {
  final String eventType;
  bool get isActive;
  void unsubscribe();
}

// EventBus - DDD パターン対応イベントバス
class EventBus {
  EventBus([EventBusConfig? config]);
  Future<void> publish<T extends DomainEvent>(T event);
  EventSubscription subscribe<T extends DomainEvent>(
    String eventType,
    EventHandler<T> handler,
  );
  void close();
}

// InMemoryEventBus - レガシー互換（関数ベース API）
class InMemoryEventBus {
  InMemoryEventBus([EventBusConfig? config]);
  void subscribe(String eventType, Future<void> Function(Event) handler);
  void unsubscribe(String eventType);
  Future<void> publish(Event event);
}
```

**カバレッジ目標**: 90%以上

## テスト戦略

### ユニットテスト（`#[cfg(test)]`）

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestHandler {
        event_type: String,
        call_count: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl EventHandler for TestHandler {
        fn event_type(&self) -> &str { &self.event_type }
        async fn handle(&self, _event: Event) -> Result<(), EventBusError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_publish_and_subscribe() {
        let bus = EventBus::new(EventBusConfig::default());
        let count = Arc::new(AtomicUsize::new(0));
        let handler = TestHandler {
            event_type: "order.created".to_string(),
            call_count: count.clone(),
        };

        let _sub = bus.subscribe(Arc::new(handler)).await;

        let event = Event::new("order.created".to_string(), serde_json::json!({}));
        bus.publish(event).await.unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_subscription_drop_unsubscribes() {
        let bus = EventBus::new(EventBusConfig::default());
        let count = Arc::new(AtomicUsize::new(0));
        let handler = TestHandler {
            event_type: "order.created".to_string(),
            call_count: count.clone(),
        };

        {
            let _sub = bus.subscribe(Arc::new(handler)).await;
            // _sub がスコープを抜けると Drop で自動解除
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let event = Event::new("order.created".to_string(), serde_json::json!({}));
        bus.publish(event).await.unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }
}
```

### 統合テスト

- eventstore ライブラリと組み合わせたイベント永続化 + バス発行の連携フローを検証
- 複数ハンドラーへの同時配信が正しく機能することを確認
- ハンドラータイムアウト時に `HandlerFailed` エラーが返却されることを確認

### モックテスト

- `EventHandler` トレイトを `mockall` でモック化し、特定イベント型のハンドラー呼び出し回数・引数を検証
- サービス層から EventBus への依存をモック化して、イベント発行ロジックを単体テスト

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-eventstore設計](../data/eventstore.md) — ドメインイベント永続化ライブラリ
- [system-library-messaging設計](messaging.md) — Kafka 外部メッセージングとの連携
- [system-library-outbox設計](outbox.md) — アウトボックスパターン実装
- [system-library-saga設計](../resilience/saga.md) — Saga オーケストレーションとのイベント連携
