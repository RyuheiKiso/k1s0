# k1s0-messaging ライブラリ設計

## 概要

Kafka イベント発行・購読の抽象化ライブラリ。`EventProducer` トレイトと `NoOpEventProducer`（テスト用）実装、`EventMetadata`、`EventEnvelope` を提供する。具体的な Kafka クライアント実装は依存せず、トレイト境界でモック差し替えが可能。

**配置先**: `regions/system/library/rust/messaging/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `EventProducer` | トレイト | イベント発行の抽象インターフェース（`async fn publish`） |
| `MockEventProducer` | 構造体 | テスト用モック（feature = "mock" で有効） |
| `EventEnvelope` | 構造体 | 発行イベントのラッパー（ペイロード + メタデータ） |
| `EventMetadata` | 構造体 | イベントID・相関ID・タイムスタンプ・ソースサービス名 |
| `MessagingConfig` | 構造体 | ブローカー・トピック・コンシューマーグループ設定 |
| `ConsumerConfig` | 構造体 | コンシューマー固有設定 |
| `EventConsumer` | トレイト | イベント購読の抽象インターフェース（`async fn subscribe`） |
| `MessagingError` | enum | 発行・購読エラー型 |

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
    let metadata = EventMetadata::new("auth-service");
    let payload = serde_json::json!({ "user_id": user_id });
    let envelope = EventEnvelope::new("k1s0.system.auth.user-created.v1", payload, metadata);
    producer.publish(envelope).await
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
