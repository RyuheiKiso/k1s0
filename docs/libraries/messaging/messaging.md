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
| `MessagingError` | enum | ProducerError・ConsumerError・SerializationError・DeserializationError・ConnectionError・TimeoutError・PublishError・ConsumeError・CommitError |

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

**依存追加**: `k1s0-messaging = { path = "../../system/library/rust/messaging" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

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

**配置先**: `regions/system/library/go/messaging/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

**配置先**: `regions/system/library/typescript/messaging/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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
  schemaVersion: string;
}

export interface EventEnvelope {
  topic: string;
  key?: string;
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

**配置先**: `regions/system/library/dart/messaging/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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
  final String schemaVersion;

  factory EventMetadata.create(String eventType, String source, {String? correlationId, String? traceId});
}

class EventEnvelope {
  final String topic;
  final String? key;
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

## 設計ノート: EventConsumer の言語間 API パターン差異

Rust の `EventConsumer` は pull 型（`receive()` + `commit()`）を採用しているのに対し、Go/TypeScript/Dart は push 型（`subscribe(topic, handler)` コールバック）を採用している。これは言語特性に基づく意図的な設計差異である。

- **Rust**: 所有権モデルにより、メッセージのライフタイムを明示的に制御する必要がある。`receive()` で所有権を取得し、処理完了後に `commit()` で消費を確定する pull 型が自然にフィットする。
- **Go/TypeScript/Dart**: GC を持つ言語ではコールバックベースの push 型がイディオマティックであり、`subscribe(topic, handler)` パターンが開発者にとって直感的である。

両パターンとも at-least-once セマンティクスを保証し、メッセージ処理の信頼性は同等である。

## 設計ノート: EventEnvelope の metadata フィールドに関する言語差異

Rust の `EventEnvelope` は `metadata` フィールドを持たず、メタデータは JSON シリアライズ時にバイト列ペイロード（`Vec<u8>`）に含める設計である。一方、Go/TypeScript/Dart の `EventEnvelope` は `metadata` フィールドを直接保持し、構造体レベルでメタデータにアクセスできる。

- **Rust**: `EventEnvelope { topic, key, payload: Vec<u8>, headers }` — メタデータはペイロードの一部として扱う
- **Go/TypeScript/Dart**: `EventEnvelope { topic, payload, metadata: EventMetadata }` — メタデータを独立フィールドとして保持

## 設計ノート: trace_id / correlation_id の型差異

Rust の `EventMetadata` では `trace_id` と `correlation_id` は `Option<String>`（未設定可）であるのに対し、Go/TypeScript/Dart では必須フィールドとして定義されている（ファクトリメソッドで UUID 自動生成またはデフォルト値が設定される）。

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-config設計](../config/config.md) — config ライブラリ
- [system-library-telemetry設計](../observability/telemetry.md) — telemetry ライブラリ
- [system-library-authlib設計](../auth-security/authlib.md) — authlib ライブラリ
- [system-library-kafka設計](kafka.md) — k1s0-kafka ライブラリ
- [system-library-correlation設計](../observability/correlation.md) — k1s0-correlation ライブラリ
- [system-library-outbox設計](outbox.md) — k1s0-outbox ライブラリ
- [system-library-schemaregistry設計](../data/schemaregistry.md) — k1s0-schemaregistry ライブラリ

---
