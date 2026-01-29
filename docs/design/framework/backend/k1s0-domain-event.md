# k1s0-domain-event

ドメインイベントの発行・購読・Outbox パターンを提供するライブラリ。

## 概要

マイクロサービス間でドメインイベントを安全に伝搬するための抽象と実装を提供する。

- **DomainEvent trait**: すべてのドメインイベントが実装する基底 trait
- **EventEnvelope**: イベント本体 + メタデータ（ID、タイムスタンプ、相関 ID 等）
- **EventPublisher / EventSubscriber**: 発行・購読の抽象 trait
- **InMemoryEventBus**: テスト・シングルプロセス向けのインメモリ実装
- **Outbox パターン**: トランザクション保証付きイベント発行（`outbox` feature）

## アーキテクチャ

```
DomainEvent (trait)
    ↓
EventEnvelope (メタデータ + JSON payload)
    ↓
EventPublisher ──→ EventSubscriber ──→ EventHandler
    │
    └── InMemoryEventBus (tokio::broadcast)
    └── OutboxRelay (polling + publish)
```

## Rust

### Tier

Tier 2（k1s0-db に依存可能）

### Feature フラグ

| Feature | 説明 | 追加依存 |
|---------|------|----------|
| `default` | コア機能のみ（イベント定義、Publisher、Subscriber、InMemoryBus） | - |
| `outbox` | Outbox パターン（OutboxStore trait、OutboxRelay） | sqlx, k1s0-db |
| `full` | 全機能 | - |

### 主要型

| 型 | モジュール | 説明 |
|----|-----------|------|
| `DomainEvent` | `event` | イベント基底 trait |
| `EventMetadata` | `envelope` | イベントメタデータ |
| `EventEnvelope` | `envelope` | イベント + メタデータの格納構造 |
| `EventPublisher` | `publisher` | 発行 trait (async) |
| `EventSubscriber` | `subscriber` | 購読 trait (async) |
| `EventHandler` | `subscriber` | ハンドラ trait (async) |
| `SubscriptionHandle` | `subscriber` | 購読キャンセル用ハンドル |
| `InMemoryEventBus` | `bus` | tokio::broadcast ベースのインメモリバス |
| `OutboxEntry` | `outbox::entry` | Outbox テーブルエントリ |
| `OutboxStore` | `outbox::store` | Outbox 永続化 trait |
| `OutboxRelay` | `outbox::relay` | ポーリングリレー |

### エラー型

| エラー | 用途 |
|--------|------|
| `PublishError` | イベント発行失敗 |
| `SubscribeError` | 購読登録失敗 |
| `HandlerError` | ハンドラ処理失敗 |
| `OutboxError` | Outbox 操作失敗 |

### 使用例

```rust
use k1s0_domain_event::{DomainEvent, EventEnvelope, EventPublisher, EventHandler, HandlerError};
use k1s0_domain_event::bus::InMemoryEventBus;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct OrderCreated { order_id: String }

impl DomainEvent for OrderCreated {
    fn event_type(&self) -> &str { "order.created" }
    fn aggregate_id(&self) -> Option<&str> { Some(&self.order_id) }
}

// 発行
let bus = InMemoryEventBus::default();
let event = OrderCreated { order_id: "ord-123".into() };
let envelope = EventEnvelope::from_event(&event, "order-service").unwrap();
bus.publish(envelope).await.unwrap();
```

## Go

### パッケージ構成

| パッケージ | 説明 |
|-----------|------|
| `domainevent` | イベント定義、Publisher/Subscriber インターフェース |
| `domainevent/bus` | InMemoryEventBus |
| `domainevent/outbox` | Outbox パターン（Entry, Store, Relay） |

### 主要型

| 型 | 説明 |
|----|------|
| `DomainEvent` | イベントインターフェース |
| `BaseDomainEvent` | デフォルト実装付き基底構造体 |
| `EventMetadata` | メタデータ構造体 |
| `EventEnvelope` | エンベロープ構造体 |
| `EventPublisher` | 発行インターフェース |
| `EventSubscriber` | 購読インターフェース |
| `EventHandler` | ハンドラインターフェース |

## Outbox パターン

### SQL スキーマ

`sql/outbox_events.sql` に PostgreSQL 用の DDL を提供。

### フロー

1. ビジネストランザクション内で `OutboxStore.insert()` を呼ぶ
2. `OutboxRelay` がポーリングで pending エントリを取得
3. `EventPublisher` 経由で発行
4. 成功したら `mark_published`、失敗したら `mark_failed`
5. `max_retries` 超過で永続的失敗とする

## 設計判断

- **InMemoryEventBus は tokio::broadcast を使用**: 複数 subscriber への同時配信が自然
- **EventEnvelope は JSON ペイロード**: 型安全性よりもスキーマ進化の柔軟性を優先
- **Outbox は feature gate**: DB 依存を持たないコアだけでも使えるように分離
