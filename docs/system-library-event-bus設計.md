# k1s0-event-bus ライブラリ設計

## 概要

インプロセスのドメインイベントバスライブラリ。`publish`/`subscribe` によるイベント駆動アーキテクチャをサポートし、eventstore ライブラリと組み合わせてドメインイベントの永続化と配信を行う。tokio::broadcast チャネルベースの実装で、サービス内の集約間・ユースケース間の疎結合な通信を実現する。

**配置先**: `regions/system/library/rust/event-bus/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `EventBus` | 構造体 | インプロセスイベントバス（tokio::broadcast ベース）|
| `DomainEvent` | トレイト | ドメインイベントインターフェース（event_type・aggregate_id・occurred_at）|
| `EventHandler` | トレイト | イベントハンドラーインターフェース |
| `EventSubscription` | 構造体 | サブスクリプション管理（Drop で自動解除）|
| `EventBusConfig` | 構造体 | チャネルバッファサイズ・ハンドラータイムアウト設定 |
| `EventBusError` | enum | `PublishFailed`・`HandlerFailed`・`ChannelClosed` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-event-bus"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
tokio = { version = "1", features = ["sync"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**Cargo.toml への追加行**:

```toml
k1s0-event-bus = { path = "../../system/library/rust/event-bus" }
```

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
use k1s0_event_bus::{DomainEvent, EventBus, EventBusConfig, EventHandler};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ドメインイベント定義
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrderCreated {
    order_id: String,
    user_id: String,
    total_amount: i64,
    occurred_at: DateTime<Utc>,
}

impl DomainEvent for OrderCreated {
    fn event_type(&self) -> &str { "order.created" }
    fn aggregate_id(&self) -> &str { &self.order_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
}

// イベントハンドラー定義
struct NotificationHandler;

#[async_trait]
impl EventHandler<OrderCreated> for NotificationHandler {
    async fn handle(&self, event: &OrderCreated) -> Result<(), k1s0_event_bus::EventBusError> {
        tracing::info!(order_id = %event.order_id, "注文作成通知を送信");
        Ok(())
    }
}

// バス初期化とイベント発行
let config = EventBusConfig::new()
    .buffer_size(1024)
    .handler_timeout(std::time::Duration::from_secs(5));

let bus = EventBus::new(config);

// ハンドラー登録（EventSubscription が Drop されると自動解除）
let _subscription = bus.subscribe(NotificationHandler).await;

// イベント発行
let event = OrderCreated {
    order_id: "ORD-001".to_string(),
    user_id: "USR-123".to_string(),
    total_amount: 10000,
    occurred_at: Utc::now(),
};
bus.publish(event).await?;
```

## Go 実装

**配置先**: `regions/system/library/go/event-bus/`

```
event-bus/
├── event_bus.go
├── event.go
├── handler.go
├── config.go
├── event_bus_test.go
├── go.mod
└── go.sum
```

**依存関係**: `github.com/google/uuid v1.6`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type DomainEvent interface {
    EventType() string
    AggregateID() string
    OccurredAt() time.Time
}

type EventHandler[T DomainEvent] interface {
    Handle(ctx context.Context, event T) error
}

type EventBus struct { /* ... */ }

func NewEventBus(config EventBusConfig) *EventBus

func Subscribe[T DomainEvent](bus *EventBus, handler EventHandler[T]) *EventSubscription

func Publish[T DomainEvent](ctx context.Context, bus *EventBus, event T) error

type EventSubscription struct { /* ... */ }
func (s *EventSubscription) Unsubscribe()

type EventBusConfig struct {
    BufferSize     int
    HandlerTimeout time.Duration
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/event-bus/`

```
event-bus/
├── package.json        # "@k1s0/event-bus", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # EventBus, DomainEvent, EventHandler, EventSubscription, EventBusConfig, EventBusError
└── __tests__/
    ├── event-bus.test.ts
    └── subscription.test.ts
```

**主要 API**:

```typescript
export interface DomainEvent {
  readonly eventType: string;
  readonly aggregateId: string;
  readonly occurredAt: Date;
}

export interface EventHandler<T extends DomainEvent> {
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

export class EventBus {
  constructor(config?: EventBusConfig);
  publish<T extends DomainEvent>(event: T): Promise<void>;
  subscribe<T extends DomainEvent>(
    eventType: string,
    handler: EventHandler<T>
  ): EventSubscription;
}

export class EventBusError extends Error {
  constructor(
    message: string,
    public readonly code: 'PUBLISH_FAILED' | 'HANDLER_FAILED' | 'CHANNEL_CLOSED'
  );
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/event-bus/`

```
event-bus/
├── pubspec.yaml        # k1s0_event_bus
├── analysis_options.yaml
├── lib/
│   ├── event_bus.dart
│   └── src/
│       ├── bus.dart            # EventBus
│       ├── event.dart          # DomainEvent abstract
│       ├── handler.dart        # EventHandler abstract・EventSubscription
│       ├── config.dart         # EventBusConfig
│       └── error.dart          # EventBusError
└── test/
    ├── event_bus_test.dart
    └── subscription_test.dart
```

**カバレッジ目標**: 90%以上

## C# 実装

**配置先**: `regions/system/library/csharp/event-bus/`

```
event-bus/
├── src/
│   ├── EventBus.csproj
│   ├── IEventBus.cs                # イベントバスインターフェース
│   ├── EventBus.cs                 # Channel<T> ベースの実装
│   ├── IDomainEvent.cs             # ドメインイベントインターフェース
│   ├── IEventHandler.cs            # イベントハンドラーインターフェース
│   ├── EventSubscription.cs        # サブスクリプション管理（IDisposable）
│   ├── EventBusConfig.cs           # バッファサイズ・タイムアウト設定
│   └── EventBusException.cs        # 公開例外型
├── tests/
│   ├── EventBus.Tests.csproj
│   ├── Unit/
│   │   ├── EventBusTests.cs
│   │   └── EventSubscriptionTests.cs
│   └── Integration/
│       └── EventBusIntegrationTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| （標準ライブラリのみ、外部依存なし） | System.Threading.Channels 使用 |

**名前空間**: `K1s0.System.EventBus`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `IEventBus` | interface | イベント発行・購読インターフェース |
| `EventBus` | class | System.Threading.Channels ベースの実装 |
| `IDomainEvent` | interface | ドメインイベント（EventType・AggregateId・OccurredAt）|
| `IEventHandler<T>` | interface | イベントハンドラーインターフェース |
| `EventSubscription` | class | サブスクリプション管理（IDisposable）|
| `EventBusConfig` | record | バッファサイズ・タイムアウト設定 |
| `EventBusException` | class | イベントバスエラーの公開例外型 |

**主要 API**:

```csharp
namespace K1s0.System.EventBus;

public interface IDomainEvent
{
    string EventType { get; }
    string AggregateId { get; }
    DateTimeOffset OccurredAt { get; }
}

public interface IEventHandler<in T> where T : IDomainEvent
{
    Task HandleAsync(T domainEvent, CancellationToken ct = default);
}

public interface IEventBus
{
    Task PublishAsync<T>(T domainEvent, CancellationToken ct = default)
        where T : IDomainEvent;

    IDisposable Subscribe<T>(IEventHandler<T> handler)
        where T : IDomainEvent;
}

public record EventBusConfig(
    int BufferSize = 1024,
    TimeSpan? HandlerTimeout = null);
```

**カバレッジ目標**: 90%以上

---

## Python 実装

**配置先**: `regions/system/library/python/event-bus/`

### パッケージ構造

```
event-bus/
├── pyproject.toml
├── src/
│   └── k1s0_event_bus/
│       ├── __init__.py       # 公開 API（再エクスポート）
│       ├── bus.py            # EventBus
│       ├── event.py          # DomainEvent ABC
│       ├── handler.py        # EventHandler ABC・EventSubscription
│       ├── config.py         # EventBusConfig
│       ├── exceptions.py     # EventBusError
│       └── py.typed
└── tests/
    ├── test_event_bus.py
    └── test_subscription.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `EventBus` | class | asyncio.Queue ベースのイベントバス実装 |
| `DomainEvent` | ABC | ドメインイベント抽象基底クラス（`event_type`・`aggregate_id`・`occurred_at`）|
| `EventHandler` | ABC | イベントハンドラー抽象基底クラス（`handle`）|
| `EventSubscription` | class | サブスクリプション管理（コンテキストマネージャー対応）|
| `EventBusConfig` | dataclass | バッファサイズ・ハンドラータイムアウト設定 |
| `EventBusError` | Exception | イベントバスエラー基底クラス |

### 使用例

```python
import asyncio
from dataclasses import dataclass
from datetime import datetime, timezone
from k1s0_event_bus import DomainEvent, EventBus, EventBusConfig, EventHandler

@dataclass
class OrderCreated(DomainEvent):
    order_id: str
    user_id: str
    total_amount: int

    @property
    def event_type(self) -> str:
        return "order.created"

    @property
    def aggregate_id(self) -> str:
        return self.order_id

    @property
    def occurred_at(self) -> datetime:
        return datetime.now(timezone.utc)

class NotificationHandler(EventHandler[OrderCreated]):
    async def handle(self, event: OrderCreated) -> None:
        print(f"注文作成通知を送信: {event.order_id}")

config = EventBusConfig(buffer_size=1024, handler_timeout=5.0)
bus = EventBus(config)

# ハンドラー登録（コンテキストマネージャーで自動解除）
async with bus.subscribe(NotificationHandler()) as subscription:
    event = OrderCreated(order_id="ORD-001", user_id="USR-123", total_amount=10000)
    await bus.publish(event)
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| （標準ライブラリのみ、外部依存なし） | — | asyncio.Queue / asyncio.Task 使用 |

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock
- カバレッジ目標: 90%以上
- 実行: `pytest` / `ruff check .`

## テスト戦略

### ユニットテスト（`#[cfg(test)]`）

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct TestEvent {
        id: String,
        occurred_at: chrono::DateTime<Utc>,
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &str { "test.event" }
        fn aggregate_id(&self) -> &str { &self.id }
        fn occurred_at(&self) -> chrono::DateTime<Utc> { self.occurred_at }
    }

    #[tokio::test]
    async fn test_publish_and_subscribe() {
        let bus = EventBus::new(EventBusConfig::default());
        let received = Arc::new(tokio::sync::Mutex::new(vec![]));

        let received_clone = received.clone();
        let _sub = bus.subscribe(move |event: TestEvent| {
            let r = received_clone.clone();
            async move {
                r.lock().await.push(event.id.clone());
                Ok(())
            }
        }).await;

        bus.publish(TestEvent { id: "evt-1".into(), occurred_at: Utc::now() }).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        assert_eq!(received.lock().await.len(), 1);
    }

    #[tokio::test]
    async fn test_subscription_drop_unsubscribes() {
        let bus = EventBus::new(EventBusConfig::default());
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter_clone = counter.clone();

        {
            let _sub = bus.subscribe(move |_: TestEvent| {
                counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                async { Ok(()) }
            }).await;
            bus.publish(TestEvent { id: "evt-1".into(), occurred_at: Utc::now() }).await.unwrap();
        } // _sub が Drop → 自動解除

        bus.publish(TestEvent { id: "evt-2".into(), occurred_at: Utc::now() }).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
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

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-eventstore設計](system-library-eventstore設計.md) — ドメインイベント永続化ライブラリ
- [system-library-messaging設計](system-library-messaging設計.md) — Kafka 外部メッセージングとの連携
- [system-library-outbox設計](system-library-outbox設計.md) — アウトボックスパターン実装
- [system-library-saga設計](system-library-saga設計.md) — Saga オーケストレーションとのイベント連携
