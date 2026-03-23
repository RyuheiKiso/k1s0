# メッセージングライブラリ 移行ガイド

## 概要

`building-blocks`、`pubsub`、`event-bus` の3ライブラリは非推奨（deprecated）となり、`messaging` ライブラリに統合されました（[ADR-0018](../../../architecture/adr/0018-messaging-abstraction-unification.md) 参照）。本ガイドでは各ライブラリからの移行手順を説明します。

## 移行対象ライブラリ

| 非推奨ライブラリ | 対応言語 | 移行先 |
|-----------------|---------|--------|
| `building-blocks` | Rust, Go, TypeScript | `messaging` |
| `pubsub` | Go | `messaging` |
| `event-bus` | Rust, Go, TypeScript | `messaging` |

## Rust: building-blocks → messaging

### 依存関係の変更
```toml
# 変更前
k1s0-building-blocks = { path = "../../library/rust/building-blocks" }

# 変更後
k1s0-messaging = { path = "../../library/rust/messaging" }
```

### import パスの変更
```rust
// 変更前
use k1s0_building_blocks::component::Component;

// 変更後
use k1s0_messaging::{EventProducer, EventConsumer, EventEnvelope, EventMetadata};
```

## Rust: event-bus → messaging

### 依存関係の変更
```toml
# 変更前
k1s0-event-bus = { path = "../../library/rust/event-bus" }

# 変更後
k1s0-messaging = { path = "../../library/rust/messaging" }
```

### import パスの変更
```rust
// 変更前
use k1s0_event_bus::{EventBus, InMemoryEventBus, DomainEvent, Event, EventHandler};

// 変更後
// インプロセスイベント配信は messaging の同期ディスパッチャー機能を使用
use k1s0_messaging::{EventProducer, EventConsumer, EventEnvelope};
```

## Go: building-blocks → messaging

### import パスの変更
```go
// 変更前
import "github.com/k1s0-platform/system-library-go-building-blocks"

// 変更後
import "github.com/k1s0-platform/system-library-go-messaging"
```

### 主要な型の対応
```go
// 変更前（building-blocks の PubSub インターフェース）
var _ = buildingblocks.NewRegistry()

// 変更後
import messaging "github.com/k1s0-platform/system-library-go-messaging"
// EventMetadata, EventEnvelope, EventProducer, EventConsumer を使用
```

## Go: pubsub → messaging

### import パスの変更
```go
// 変更前
import "github.com/k1s0-platform/system-library-go-pubsub"

// 変更後
import "github.com/k1s0-platform/system-library-go-messaging"
```

### 主要な型の対応
```go
// 変更前
var ps pubsub.PubSub
ps.Publish(ctx, topic, payload)
ps.Subscribe(ctx, topic, handler)

// 変更後
var producer messaging.EventProducer
producer.Publish(ctx, envelope)
var consumer messaging.EventConsumer
consumer.Subscribe(ctx, topic, handler)
```

## Go: event-bus → messaging

### import パスの変更
```go
// 変更前
import "github.com/k1s0-platform/system-library-go-event-bus"

// 変更後
import "github.com/k1s0-platform/system-library-go-messaging"
```

### 主要な型の対応
```go
// 変更前
var bus eventbus.EventBus
bus.Publish(ctx, event)

// 変更後
// インプロセスイベント配信は messaging の同期ディスパッチャー機能を使用
var producer messaging.EventProducer
producer.Publish(ctx, messaging.NewEventEnvelope(eventType, payload))
```

## TypeScript: building-blocks → messaging

### 依存関係の変更
```json
{
  "dependencies": {
    "@k1s0/messaging": "workspace:*"
  }
}
```

### import パスの変更
```typescript
// 変更前
import { EventBus } from '@k1s0/building-blocks';

// 変更後
import { MessageBus } from '@k1s0/messaging';
```

## TypeScript: event-bus → messaging

### 依存関係の変更
```json
{
  "dependencies": {
    "@k1s0/messaging": "workspace:*"
  }
}
```

### import パスの変更
```typescript
// 変更前
import { EventBus } from '@k1s0/event-bus';

// 変更後
import { MessageBus } from '@k1s0/messaging';
```

## FAQ

**Q: 移行期間はいつまでですか？**
A: 非推奨ライブラリは次のメジャーバージョンリリース時に削除予定です。早期移行を推奨します。

**Q: API の互換性はありますか？**
A: 一部 API の名称変更があります。詳細は `messaging` ライブラリの [API ドキュメント](./messaging.md) を参照してください。

**Q: インプロセスのドメインイベント配信（event-bus 相当）はどうすればよいですか？**
A: ADR-0018 に記載のとおり、`messaging` ライブラリの同期ディスパッチャー機能を拡充して対応します。移行が完了するまでは既存の `event-bus` ライブラリを継続使用できます。
