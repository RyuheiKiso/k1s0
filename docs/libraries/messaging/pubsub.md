# k1s0-pubsub ライブラリ設計

> **実装形態**: Go は独立パッケージ、Rust は bb-pubsub、TypeScript/Dart は building-blocks に統合
> - Go: `regions/system/library/go/pubsub/` (module: `github.com/k1s0-platform/system-library-go-pubsub`)
> - Rust: `regions/system/library/rust/bb-pubsub/`
> - TypeScript/Dart: `regions/system/library/{typescript,dart}/building-blocks/`

## 概要

PubSub Building Block ライブラリ。トピックベースのメッセージングを統一インターフェースで抽象化する。Kafka・Redis Pub/Sub・InMemory の 3 コンポーネントを提供し、アプリケーションコードは `PubSub` トレイト/インターフェースのみに依存する。KafkaPubSub は内部で k1s0-kafka（接続管理）と k1s0-messaging（EventProducer）をラップして利用する。

**配置先**: `regions/system/library/rust/bb-pubsub/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `PubSub` | トレイト | PubSub Building Block インターフェース（Component 継承） |
| `Message` | 構造体 | メッセージデータ（id・topic・data・metadata・timestamp） |
| `TopicConfig` | 構造体 | トピック設定（パーティション数・レプリケーション等） |
| `PubSubError` | enum | `PublishFailed`・`SubscribeFailed`・`DeserializeFailed`・`ConnectionFailed` |
| `KafkaPubSub` | 構造体 | Kafka 実装（k1s0-kafka + k1s0-messaging ラッパー） |
| `InMemoryPubSub` | 構造体 | InMemory 実装（テスト用） |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "bb-pubsub"
version = "0.1.0"
edition = "2021"

[features]
default = []
kafka = ["k1s0-messaging", "k1s0-kafka"]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
k1s0-bb-core = { path = "../bb-core" }
k1s0-kafka = { path = "../kafka", optional = true }
k1s0-messaging = { path = "../messaging", optional = true }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
tokio = { version = "1", features = ["sync"] }
tracing = "0.1"
uuid = { version = "1", features = ["v4"] }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `bb-pubsub = { path = "../../system/library/rust/bb-pubsub" }`

**モジュール構成**:

```
bb-pubsub/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── traits.rs       # PubSub トレイト定義・Message 型
│   ├── error.rs        # PubSubError
│   ├── kafka.rs        # KafkaPubSub（feature = "kafka"）
│   └── memory.rs       # InMemoryPubSub
└── Cargo.toml
```

**PubSub トレイト**:

```rust
use async_trait::async_trait;
use tokio_stream::Stream;
use std::pin::Pin;

#[async_trait]
pub trait PubSub: Component + Send + Sync {
    /// トピックにメッセージを発行する。
    async fn publish(&self, topic: &str, message: Message) -> Result<(), PubSubError>;

    /// トピックを購読し、メッセージストリームを返す。
    async fn subscribe(
        &self,
        topic: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Message, PubSubError>> + Send>>, PubSubError>;

    /// 全リソースを解放する。
    async fn close(&self) -> Result<(), PubSubError>;
}
```

**KafkaPubSub 内部構造**:

KafkaPubSub は以下の既存ライブラリを内部利用する:

- `k1s0-kafka`: Kafka ブローカーへの接続管理・TLS 設定・ヘルスチェック
- `k1s0-messaging`: `EventProducer` トレイトによるイベント発行・`EventEnvelope` によるメッセージシリアライズ

```rust
pub struct KafkaPubSub {
    kafka_config: k1s0_kafka::KafkaConfig,
    producer: k1s0_messaging::EventProducer,
    consumer_group: String,
}

#[async_trait]
impl Component for KafkaPubSub {
    fn name(&self) -> &str { "kafka" }
    fn version(&self) -> &str { "1.0.0" }

    async fn init(&mut self, metadata: Metadata) -> Result<(), ComponentError> {
        let brokers = metadata.properties.get("brokers")
            .ok_or_else(|| ComponentError::InitFailed("brokers required".into()))?;
        self.consumer_group = metadata.properties.get("consumerGroup")
            .cloned()
            .unwrap_or_default();
        // k1s0-kafka で接続初期化
        self.kafka_config = k1s0_kafka::KafkaConfig::new(brokers);
        Ok(())
    }

    async fn close(&self) -> Result<(), ComponentError> {
        // プロデューサー・コンシューマーのクローズ
        Ok(())
    }
}
```

**使用例**:

```rust
use k1s0_pubsub::{PubSub, Message, InMemoryPubSub, KafkaPubSub};
use tokio_stream::StreamExt;

// InMemory（テスト用）
let pubsub = InMemoryPubSub::new();

// Kafka（本番用）
// let pubsub = KafkaPubSub::new(kafka_config, producer);

let msg = Message::new("order.created", serde_json::json!({"order_id": "ORD-001"}));
pubsub.publish("orders", msg).await?;

let mut stream = pubsub.subscribe("orders").await?;
while let Some(Ok(message)) = stream.next().await {
    println!("received: {:?}", message);
}
```

## Go 実装

**配置先**: `regions/system/library/go/building-blocks/`

**主要インターフェース**:

```go
package pubsub

import "context"

// PubSub はメッセージング抽象化インターフェース。
type PubSub interface {
    buildingblocks.Component
    // msg.Topic に宛てた発行。トピックは Message 内に含む。
    Publish(ctx context.Context, msg *Message) error
    Subscribe(ctx context.Context, topic string) (<-chan *Message, error)
}

// Message はメッセージデータを表す。
type Message struct {
    ID        string            `json:"id"`
    Topic     string            `json:"topic"`
    Data      []byte            `json:"data"`
    Metadata  map[string]string `json:"metadata"`
    Timestamp time.Time         `json:"timestamp"`
}

// TopicConfig はトピック設定を表す。
type TopicConfig struct {
    Partitions        int
    ReplicationFactor int
}

// PubSubError はエラー種別を表す。
type ErrorKind int

const (
    PublishFailed ErrorKind = iota
    SubscribeFailed
    DeserializeFailed
    ConnectionFailed
)

type PubSubError struct {
    Kind    ErrorKind
    Message string
    Err     error
}

func (e *PubSubError) Error() string
func (e *PubSubError) Unwrap() error

// --- 実装 ---

// KafkaPubSub: KafkaEventProducer / KafkaEventConsumer インターフェース経由で注入（k1s0-messaging 互換）。
type KafkaPubSub struct{}
func NewKafkaPubSub(name string, producer KafkaEventProducer, consumer KafkaEventConsumer) *KafkaPubSub

// RedisPubSub: RedisPubSubClient インターフェース経由で注入（k1s0-cache 互換）。
type RedisPubSub struct{}
func NewRedisPubSub(name string, client RedisPubSubClient) *RedisPubSub

// InMemoryPubSub: テスト・開発用（外部ブローカー不要）。
type InMemoryPubSub struct{}
func NewInMemoryPubSub() *InMemoryPubSub
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/building-blocks/`

> **現在の実装**: `InMemoryPubSub` のみ。本番バックエンド（Kafka・Redis）は Go/Rust 側で提供し、TypeScript は主にテスト・開発用の InMemory を使用する。

**主要 API**:

```typescript
import { Component, Metadata, ComponentError } from '@k1s0/building-blocks';

export interface Message {
  readonly id: string;
  readonly topic: string;
  readonly data: unknown;
  readonly metadata: Record<string, string>;
  readonly timestamp: string;
}

export interface TopicConfig {
  readonly partitions?: number;
  readonly replicationFactor?: number;
}

export type PubSubErrorCode =
  | 'PUBLISH_FAILED'
  | 'SUBSCRIBE_FAILED'
  | 'DESERIALIZE_FAILED'
  | 'CONNECTION_FAILED';

export class PubSubError extends Error {
  constructor(
    message: string,
    public readonly code: PubSubErrorCode,
  ) {
    super(message);
  }
}

export interface PubSub extends Component {
  publish(topic: string, message: Message): Promise<void>;
  subscribe(
    topic: string,
    handler: (message: Message) => Promise<void>,
  ): Promise<{ unsubscribe(): Promise<void> }>;
  close(): Promise<void>;
}

export class KafkaPubSub implements PubSub { /* ... */ }
export class RedisPubSub implements PubSub { /* ... */ }
export class InMemoryPubSub implements PubSub { /* ... */ }
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/building-blocks/`

> **現在の実装**: `InMemoryPubSub` のみ。本番バックエンドは Go/Rust 側で提供する。

**主要 API**:

```dart
import 'package:k1s0_building_blocks/component.dart';

class Message {
  final String id;
  final String topic;
  final dynamic data;
  final Map<String, String> metadata;
  final DateTime timestamp;

  const Message({
    required this.id,
    required this.topic,
    required this.data,
    this.metadata = const {},
    required this.timestamp,
  });
}

class TopicConfig {
  final int? partitions;
  final int? replicationFactor;
  const TopicConfig({this.partitions, this.replicationFactor});
}

enum PubSubErrorCode {
  publishFailed,
  subscribeFailed,
  deserializeFailed,
  connectionFailed,
}

class PubSubError implements Exception {
  final String message;
  final PubSubErrorCode code;
  const PubSubError(this.message, this.code);
}

abstract class PubSub implements Component {
  Future<void> publish(String topic, Message message);
  Stream<Message> subscribe(String topic);
  Future<void> close();
}

class KafkaPubSub implements PubSub { /* k1s0-kafka + k1s0-messaging ラッパー */ }
class RedisPubSub implements PubSub { /* Redis Pub/Sub */ }
class InMemoryPubSub implements PubSub { /* テスト用 */ }
```

**カバレッジ目標**: 90%以上

## テスト戦略

### ユニットテスト

InMemoryPubSub を活用し、外部依存なしで PubSub トレイトの振る舞いを検証する。

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_publish_and_subscribe() {
        let pubsub = InMemoryPubSub::new();
        let mut stream = pubsub.subscribe("orders").await.unwrap();

        let msg = Message::new("orders", serde_json::json!({"id": "ORD-001"}));
        pubsub.publish("orders", msg).await.unwrap();

        let received = stream.next().await.unwrap().unwrap();
        assert_eq!(received.topic, "orders");
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let pubsub = InMemoryPubSub::new();
        let mut stream1 = pubsub.subscribe("events").await.unwrap();
        let mut stream2 = pubsub.subscribe("events").await.unwrap();

        let msg = Message::new("events", serde_json::json!({"type": "test"}));
        pubsub.publish("events", msg).await.unwrap();

        assert!(stream1.next().await.is_some());
        assert!(stream2.next().await.is_some());
    }

    #[tokio::test]
    async fn test_topic_isolation() {
        let pubsub = InMemoryPubSub::new();
        let mut stream = pubsub.subscribe("topic-a").await.unwrap();

        let msg = Message::new("topic-b", serde_json::json!({}));
        pubsub.publish("topic-b", msg).await.unwrap();

        // topic-a のストリームにはメッセージが届かない
        tokio::select! {
            _ = stream.next() => panic!("unexpected message"),
            _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {},
        }
    }
}
```

### 統合テスト

- testcontainers で Kafka・Redis を起動し、KafkaPubSub・RedisPubSub の実際の動作を検証
- メッセージの順序保証・パーティション分散を確認
- 接続断時の再接続・エラーハンドリングを検証

### コントラクトテスト

全 PubSub 実装（Kafka・Redis・InMemory）が同一の振る舞い仕様を満たすことを共通テストスイートで検証する。

**カバレッジ目標**: 90%以上

---

## Doc Sync (2026-03-21)

### RedisPubSub backpressure ポリシー [F10]

**backpressure 方針**

`redis_pubsub.go` の `Subscribe` ハンドラーは、バッファ付きチャネル（容量 64）を使用する。
コンシューマが追いつかずバッファが満杯になった場合は、メッセージをドロップして配信ループの継続を優先する。

- バッファ満杯時: メッセージをドロップし、`slog.Warn` でトピック名とバッファ容量をログ出力する
- コンテキストキャンセル時: エラーを返して処理を終了する

```go
select {
case ch <- msg:
case <-ctx.Done():
    return ctx.Err()
default:
    // バッファが満杯のためメッセージをドロップする。
    // コンシューマが追いつかない場合の backpressure 対策として、
    // ブロックせずにドロップして配信ループの継続を優先する。
    slog.Warn("PubSub メッセージをドロップしました: バッファ満杯",
        slog.String("topic", topic),
        slog.Int("buffer_cap", cap(ch)),
    )
}
```

この設計により、低速コンシューマが Redis のメッセージ受信ループをブロックするリスクを排除する。
ドロップが頻発する場合は `slog.Warn` ログを監視してコンシューマの処理速度改善またはバッファ拡大を検討する。

### goroutine チャネル送信の ctx.Done() 対応 [技術品質監査 High 3-1]

**背景・問題**

`kafka_pubsub.go` および `redis_pubsub.go` の `Subscribe` ハンドラー内で、
チャネルバッファが満杯の場合に `select { default: }` でメッセージを無言でドロップしていた。
バッファが一時的に満杯になった場合でも、受信側の処理が追いつけばメッセージを送信できるはずだが、
`default:` パターンではその機会を失う問題があった。

**対応内容**

Kafka・Redis 双方のハンドラーで `default:` ケースを除去し、`ctx.Done()` ケースに置き換えた後、
RedisPubSub は F10 対応で再度 `default:` ケースを追加して backpressure ドロップポリシーを明示化した。

**影響範囲**

- `regions/system/library/go/building-blocks/kafka_pubsub.go`（Subscribe ハンドラー）
- `regions/system/library/go/building-blocks/redis_pubsub.go`（Subscribe ハンドラー）

---

## 関連ドキュメント

- [Building Blocks 概要](../_common/building-blocks.md) — BB 設計思想・共通インターフェース
- [system-library-messaging設計](messaging.md) — Kafka イベント発行・購読（EventProducer）
- [system-library-kafka設計](kafka.md) — Kafka 接続管理
- [system-library-event-bus設計](event-bus.md) — インプロセスイベントバス
- [system-library-outbox設計](outbox.md) — アウトボックスパターン
- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
