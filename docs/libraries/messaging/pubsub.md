# k1s0-pubsub ライブラリ設計

## 概要

PubSub Building Block ライブラリ。トピックベースのメッセージングを統一インターフェースで抽象化する。Kafka・Redis Pub/Sub・InMemory の 3 コンポーネントを提供し、アプリケーションコードは `PubSub` トレイト/インターフェースのみに依存する。KafkaPubSub は内部で k1s0-kafka（接続管理）と k1s0-messaging（EventProducer）をラップして利用する。

**配置先**: `regions/system/library/rust/pubsub/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `PubSub` | トレイト | PubSub Building Block インターフェース（Component 継承） |
| `Message` | 構造体 | メッセージデータ（id・topic・data・metadata・timestamp） |
| `TopicConfig` | 構造体 | トピック設定（パーティション数・レプリケーション等） |
| `PubSubError` | enum | `PublishFailed`・`SubscribeFailed`・`DeserializeFailed`・`ConnectionFailed` |
| `KafkaPubSub` | 構造体 | Kafka 実装（k1s0-kafka + k1s0-messaging ラッパー） |
| `RedisPubSub` | 構造体 | Redis Pub/Sub 実装 |
| `InMemoryPubSub` | 構造体 | InMemory 実装（テスト用） |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-pubsub"
version = "0.1.0"
edition = "2021"

[features]
kafka = ["k1s0-kafka", "k1s0-messaging"]
redis = ["k1s0-cache"]
mock = []

[dependencies]
async-trait = "0.1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tokio-stream = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
k1s0-kafka = { path = "../kafka", optional = true }
k1s0-messaging = { path = "../messaging", optional = true }
k1s0-cache = { path = "../cache", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `k1s0-pubsub = { path = "../../system/library/rust/pubsub" }`

**モジュール構成**:

```
pubsub/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── traits.rs       # PubSub トレイト定義
│   ├── message.rs      # Message・TopicConfig
│   ├── error.rs        # PubSubError
│   ├── kafka.rs        # KafkaPubSub（feature = "kafka"）
│   ├── redis.rs        # RedisPubSub（feature = "redis"）
│   └── in_memory.rs    # InMemoryPubSub
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

**配置先**: `regions/system/library/go/pubsub/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要インターフェース**:

```go
package pubsub

import "context"

// PubSub はメッセージング抽象化インターフェース。
type PubSub interface {
    buildingblocks.Component
    Publish(ctx context.Context, topic string, message *Message) error
    Subscribe(ctx context.Context, topic string) (<-chan *Message, error)
    Close(ctx context.Context) error
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

type KafkaPubSub struct { /* k1s0-kafka + k1s0-messaging 内部利用 */ }
func NewKafkaPubSub() *KafkaPubSub

type RedisPubSub struct { /* Redis Pub/Sub */ }
func NewRedisPubSub() *RedisPubSub

type InMemoryPubSub struct { /* テスト用 */ }
func NewInMemoryPubSub() *InMemoryPubSub
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/pubsub/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

**配置先**: `regions/system/library/dart/pubsub/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

## 関連ドキュメント

- [Building Blocks 概要](../_common/building-blocks.md) — BB 設計思想・共通インターフェース
- [system-library-messaging設計](messaging.md) — Kafka イベント発行・購読（EventProducer）
- [system-library-kafka設計](kafka.md) — Kafka 接続管理
- [system-library-event-bus設計](event-bus.md) — インプロセスイベントバス
- [system-library-outbox設計](outbox.md) — アウトボックスパターン
- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
