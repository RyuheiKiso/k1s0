# k1s0-messaging ライブラリ設計

## 概要

Kafka イベント発行・購読の抽象化ライブラリ。`EventProducer` トレイトと `NoOpEventProducer`（テスト用）実装、`EventMetadata`、`EventEnvelope` を提供する。具体的な Kafka クライアント実装は依存せず、トレイト境界でモック差し替えが可能。

**配置先**: `regions/system/library/rust/messaging/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `EventProducer` | トレイト | イベント発行の抽象インターフェース（`async fn publish`・`async fn publish_batch`） |
| `NoOpEventProducer` | 構造体 | テスト・スタブ用の何もしない実装（常に `Ok(())` を返す） |
| `MockEventProducer` | 構造体 | テスト用モック（feature = "mock" で有効） |
| `EventEnvelope` | 構造体 | 送信メッセージのラッパー（トピック・キー・バイト列ペイロード・ヘッダー） |
| `EventMetadata` | 構造体 | イベントID・イベント種別・発行元・タイムスタンプ・トレースID・相関ID・スキーマバージョン |
| `MessagingConfig` | 構造体 | ブローカー・セキュリティプロトコル・タイムアウト・バッチサイズ設定 |
| `ConsumerConfig` | 構造体 | グループID・トピックリスト・オートコミット・セッションタイムアウト設定 |
| `ConsumedMessage` | 構造体 | 受信メッセージ（トピック・パーティション・オフセット・キー(`Option<Vec<u8>>`)・ペイロード） |
| `EventConsumer` | トレイト | イベント購読インターフェース（`async fn receive` + `async fn commit`） |
| `MessagingError` | enum | ProducerError・ConsumerError・SerializationError・DeserializationError・ConnectionError・TimeoutError |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-messaging"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
mockall = "0.13"
```

**Cargo.toml への追加行**:

```toml
k1s0-messaging = { path = "../../system/library/rust/messaging" }
# テスト時にモックを有効化する場合:
k1s0-messaging = { path = "../../system/library/rust/messaging", features = ["mock"] }
```

**モジュール構成**:

```
messaging/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── config.rs       # MessagingConfig・ConsumerConfig
│   ├── consumer.rs     # EventConsumer トレイト
│   ├── error.rs        # MessagingError
│   ├── event.rs        # EventEnvelope・EventMetadata
│   └── producer.rs     # EventProducer トレイト・MockEventProducer
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_messaging::{EventEnvelope, EventMetadata, EventProducer};

// プロデューサーへのイベント発行
async fn publish_user_created<P: EventProducer>(
    producer: &P,
    user_id: &str,
) -> Result<(), k1s0_messaging::MessagingError> {
    let _meta = EventMetadata::new("auth.user-created", "auth-server")
        .with_correlation_id("corr-001");
    let payload = serde_json::json!({ "user_id": user_id });
    let envelope = EventEnvelope::json(
        "k1s0.system.auth.user-created.v1",
        user_id,
        &payload,
    ).map_err(|e| k1s0_messaging::MessagingError::SerializationError(e.to_string()))?;
    producer.publish(envelope).await
}

// コンシューマーからのメッセージ受信（手動コミット）
async fn consume_events<C: k1s0_messaging::EventConsumer>(consumer: &C) {
    loop {
        let msg = consumer.receive().await.unwrap();
        let value: serde_json::Value = msg.deserialize_json().unwrap();
        // 処理...
        consumer.commit(&msg).await.unwrap();
    }
}
```

## Go 実装

**配置先**: `regions/system/library/go/messaging/`

```
messaging/
├── messaging.go
├── noop.go
├── messaging_test.go
├── go.mod
└── go.sum
```

**依存関係**: `github.com/google/uuid v1.6.0`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type EventProducer interface {
    Publish(ctx context.Context, event EventEnvelope) error
    Close() error
}

type EventConsumer interface {
    Subscribe(ctx context.Context, topic string, handler EventHandler) error
    Close() error
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/messaging/`

```
messaging/
├── package.json        # "@k1s0/messaging", "type":"module"
├── tsconfig.json       # ES2022, Node16, strict, declaration
├── vitest.config.ts    # globals:true, __tests__/**/*.test.ts
├── src/
│   └── index.ts        # EventMetadata, EventEnvelope, EventHandler, EventProducer, EventConsumer, NoOpEventProducer, MessagingError
└── __tests__/
    └── messaging.test.ts
```

**依存関係**: `uuid` (v9+), `vitest` (dev)

**主要 API**:

```typescript
export interface EventMetadata {
  eventId: string;
  eventType: string;
  correlationId: string;
  traceId: string;
  timestamp: string;
  source: string;
}

export interface EventEnvelope {
  topic: string;
  payload: unknown;
  metadata: EventMetadata;
}

export type EventHandler = (event: EventEnvelope) => Promise<void>;

export interface EventProducer {
  publish(event: EventEnvelope): Promise<void>;
  close(): Promise<void>;
}

export interface EventConsumer {
  subscribe(topic: string, handler: EventHandler): Promise<void>;
  close(): Promise<void>;
}

// テスト用 NoOp 実装
export class NoOpEventProducer implements EventProducer {
  published: EventEnvelope[];
  async publish(event: EventEnvelope): Promise<void>;
  async close(): Promise<void>;
}

export class MessagingError extends Error {
  constructor(op: string, cause?: Error);
}
```

**カバレッジ目標**: 85%以上

## Dart 実装

**配置先**: `regions/system/library/dart/messaging/`

```
messaging/
├── pubspec.yaml        # k1s0_messaging, sdk >=3.4.0 <4.0.0, uuid: ^4.4.0
├── analysis_options.yaml
├── lib/
│   ├── messaging.dart  # エクスポート
│   └── src/
│       ├── types.dart  # EventMetadata, EventEnvelope
│       ├── producer.dart  # EventProducer abstract class, NoOpEventProducer
│       ├── consumer.dart  # EventConsumer abstract class, EventHandler typedef
│       └── error.dart  # MessagingError
└── test/
    └── messaging_test.dart
```

**依存関係**: `uuid: ^4.4.0`, `lints: ^4.0.0` (dev)

**主要 API**:

```dart
class EventMetadata {
  final String eventId;
  final String eventType;
  final String correlationId;
  final String traceId;
  final DateTime timestamp;
  final String source;

  factory EventMetadata.create(String eventType, String source, {String? correlationId, String? traceId});
}

class EventEnvelope {
  final String topic;
  final Object payload;
  final EventMetadata metadata;
}

typedef EventHandler = Future<void> Function(EventEnvelope event);

abstract class EventProducer {
  Future<void> publish(EventEnvelope event);
  Future<void> close();
}

abstract class EventConsumer {
  Future<void> subscribe(String topic, EventHandler handler);
  Future<void> close();
}

class NoOpEventProducer implements EventProducer {
  final List<EventEnvelope> published = [];
}

class MessagingError implements Exception {
  final String op;
  final Object? cause;
}
```

**カバレッジ目標**: 85%以上

## C# 実装

**配置先**: `regions/system/library/csharp/messaging/`

```
messaging/
├── src/
│   ├── Messaging.csproj
│   ├── IEventProducer.cs          # イベント発行インターフェース
│   ├── IEventConsumer.cs          # イベント購読インターフェース
│   ├── KafkaEventProducer.cs      # Kafka プロデューサー実装（IAsyncDisposable）
│   ├── KafkaEventConsumer.cs      # Kafka コンシューマー実装（IAsyncDisposable）
│   ├── EventEnvelope.cs           # 送信メッセージラッパー
│   ├── EventMetadata.cs           # イベントメタデータ（traceId, correlationId, schemaVersion）
│   ├── ConsumedMessage.cs         # 受信メッセージ（topic, partition, offset, key, payload）
│   ├── ConsumerConfig.cs          # コンシューマー設定
│   └── MessagingException.cs      # 公開例外型
├── tests/
│   ├── Messaging.Tests.csproj
│   ├── Unit/
│   │   ├── EventEnvelopeTests.cs
│   │   └── EventMetadataTests.cs
│   └── Integration/
│       └── KafkaEventProducerTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| Confluent.Kafka | Kafka クライアント |

**名前空間**: `K1s0.System.Messaging`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `IEventProducer` | interface | イベント発行の抽象（PublishAsync, PublishBatchAsync） |
| `IEventConsumer` | interface | イベント購読の抽象（ReceiveAsync, CommitAsync） |
| `KafkaEventProducer` | class | Kafka プロデューサー実装（IAsyncDisposable） |
| `KafkaEventConsumer` | class | Kafka コンシューマー実装（IAsyncDisposable） |
| `EventEnvelope` | record | 送信メッセージラッパー（topic・key・payload・headers） |
| `EventMetadata` | record | イベントメタデータ（eventId・eventType・traceId・correlationId・schemaVersion） |
| `ConsumedMessage` | record | 受信メッセージ（topic・partition・offset・key・payload） |
| `ConsumerConfig` | record | コンシューマー設定（groupId・topics・autoCommit・sessionTimeout） |

**主要 API**:

```csharp
namespace K1s0.System.Messaging;

public interface IEventProducer : IAsyncDisposable
{
    Task PublishAsync(
        EventEnvelope envelope,
        CancellationToken cancellationToken = default);

    Task PublishBatchAsync(
        IEnumerable<EventEnvelope> envelopes,
        CancellationToken cancellationToken = default);
}

public interface IEventConsumer : IAsyncDisposable
{
    Task<ConsumedMessage> ReceiveAsync(
        CancellationToken cancellationToken = default);

    Task CommitAsync(
        ConsumedMessage message,
        CancellationToken cancellationToken = default);
}
```

**カバレッジ目標**: 85%以上

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-config設計](system-library-config設計.md) — config ライブラリ
- [system-library-telemetry設計](system-library-telemetry設計.md) — telemetry ライブラリ
- [system-library-authlib設計](system-library-authlib設計.md) — authlib ライブラリ
- [system-library-kafka設計](system-library-kafka設計.md) — k1s0-kafka ライブラリ
- [system-library-correlation設計](system-library-correlation設計.md) — k1s0-correlation ライブラリ
- [system-library-outbox設計](system-library-outbox設計.md) — k1s0-outbox ライブラリ
- [system-library-schemaregistry設計](system-library-schemaregistry設計.md) — k1s0-schemaregistry ライブラリ
- [system-library-serviceauth設計](system-library-serviceauth設計.md) — k1s0-serviceauth ライブラリ
