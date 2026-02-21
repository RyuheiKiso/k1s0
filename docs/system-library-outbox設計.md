# k1s0-outbox ライブラリ設計

## 概要

トランザクショナルアウトボックスパターンライブラリ。データベーストランザクションと Kafka メッセージ発行の原子性を保証する。`OutboxMessage`（指数バックオフリトライ）、`OutboxStore` トレイト、`OutboxPublisher` トレイト、`OutboxProcessor` を提供する。

**配置先**: `regions/system/library/rust/outbox/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `OutboxMessage` | 構造体 | アウトボックスに保存するメッセージ（トピック・ペイロード・ステータス・リトライ回数） |
| `OutboxStatus` | enum | メッセージのステータス（`Pending`・`Published`・`Failed`） |
| `OutboxStore` | トレイト | アウトボックスメッセージの永続化抽象（`save`・`fetch_pending`・`mark_published`） |
| `OutboxProcessor` | 構造体 | `OutboxStore` から未発行メッセージを取得し発行するポーリングプロセッサ |
| `OutboxError` | enum | 保存・取得・発行エラー型 |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-outbox"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time"] }
tracing = "0.1"
uuid = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
mockall = "0.13"
```

**Cargo.toml への追加行**:

```toml
k1s0-outbox = { path = "../../system/library/rust/outbox" }
```

**モジュール構成**:

```
outbox/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── error.rs        # OutboxError
│   ├── message.rs      # OutboxMessage・OutboxStatus（指数バックオフ計算含む）
│   ├── processor.rs    # OutboxProcessor（ポーリングループ・リトライ制御）
│   └── store.rs        # OutboxStore トレイト・OutboxPublisher トレイト
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_outbox::{OutboxMessage, OutboxProcessor, OutboxStore};

// ドメインイベント保存とメッセージ発行を同一トランザクションで実行
async fn create_user_with_event<S: OutboxStore>(
    store: &S,
    user_id: &str,
) -> Result<(), k1s0_outbox::OutboxError> {
    let payload = serde_json::json!({ "user_id": user_id });
    let msg = OutboxMessage::new("k1s0.system.auth.user-created.v1", payload);
    // DB トランザクション内で保存（Saga の一部として）
    store.save(&msg).await
}

// バックグラウンドで未発行メッセージをポーリング発行
let processor = OutboxProcessor::new(store, publisher, /* poll_interval */ Duration::from_secs(5));
processor.run().await;
```

## Go 実装

**配置先**: `regions/system/library/go/outbox/`

```
outbox/
├── outbox.go
├── processor.go
├── outbox_test.go
├── go.mod
└── go.sum
```

**依存関係**: `github.com/google/uuid v1.6.0`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type OutboxStore interface {
    SaveMessage(ctx context.Context, msg OutboxMessage) error
    GetPendingMessages(ctx context.Context, limit int) ([]OutboxMessage, error)
    UpdateStatus(ctx context.Context, id string, status OutboxStatus) error
}

type OutboxPublisher interface {
    Publish(ctx context.Context, msg OutboxMessage) error
}
```

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-config設計](system-library-config設計.md) — config ライブラリ
- [system-library-telemetry設計](system-library-telemetry設計.md) — telemetry ライブラリ
- [system-library-authlib設計](system-library-authlib設計.md) — authlib ライブラリ
- [system-library-messaging設計](system-library-messaging設計.md) — k1s0-messaging ライブラリ
- [system-library-kafka設計](system-library-kafka設計.md) — k1s0-kafka ライブラリ
- [system-library-correlation設計](system-library-correlation設計.md) — k1s0-correlation ライブラリ
- [system-library-schemaregistry設計](system-library-schemaregistry設計.md) — k1s0-schemaregistry ライブラリ
- [system-library-serviceauth設計](system-library-serviceauth設計.md) — k1s0-serviceauth ライブラリ
