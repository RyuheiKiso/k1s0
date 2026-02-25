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

## TypeScript 実装

**配置先**: `regions/system/library/typescript/outbox/`

```
outbox/
├── package.json        # "@k1s0/outbox", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # OutboxMessage, OutboxStatus, OutboxStore, OutboxPublisher, OutboxProcessor, OutboxError
└── __tests__/
    └── outbox.test.ts
```

**主要 API**:

```typescript
export type OutboxStatus = 'PENDING' | 'PROCESSING' | 'DELIVERED' | 'FAILED';

export interface OutboxMessage {
  id: string;
  topic: string;
  eventType: string;
  payload: string;
  status: OutboxStatus;
  retryCount: number;
  scheduledAt: Date;
  createdAt: Date;
  updatedAt: Date;
  correlationId: string;
}

export function createOutboxMessage(topic: string, eventType: string, payload: string, correlationId: string): OutboxMessage;
export function nextScheduledAt(retryCount: number): Date;
export function canTransitionTo(from: OutboxStatus, to: OutboxStatus): boolean;

export interface OutboxStore {
  saveMessage(msg: OutboxMessage): Promise<void>;
  getPendingMessages(limit: number): Promise<OutboxMessage[]>;
  updateStatus(id: string, status: OutboxStatus): Promise<void>;
  updateStatusWithRetry(id: string, status: OutboxStatus, retryCount: number, scheduledAt: Date): Promise<void>;
}

export interface OutboxPublisher {
  publish(msg: OutboxMessage): Promise<void>;
}

export class OutboxProcessor {
  constructor(store: OutboxStore, publisher: OutboxPublisher, batchSize?: number);
  processBatch(): Promise<number>;
  run(intervalMs: number, signal?: AbortSignal): Promise<void>;
}

export class OutboxError extends Error {
  constructor(op: string, cause?: Error);
}
```

**カバレッジ目標**: 85%以上

## Dart 実装

**配置先**: `regions/system/library/dart/outbox/`

```
outbox/
├── pubspec.yaml        # k1s0_outbox, uuid: ^4.4.0
├── analysis_options.yaml
├── lib/
│   ├── outbox.dart
│   └── src/
│       ├── message.dart    # OutboxMessage, OutboxStatus, 状態遷移検証, 指数バックオフ
│       ├── store.dart      # OutboxStore abstract, OutboxPublisher abstract
│       ├── processor.dart  # OutboxProcessor（ポーリングループ）
│       └── error.dart      # OutboxError
└── test/
    └── outbox_test.dart
```

**カバレッジ目標**: 85%以上

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-config設計](system-library-config設計.md) — config ライブラリ
- [system-library-telemetry設計](system-library-telemetry設計.md) — telemetry ライブラリ
- [system-library-authlib設計](system-library-authlib設計.md) — authlib ライブラリ
- [system-library-messaging設計](system-library-messaging設計.md) — k1s0-messaging ライブラリ
- [system-library-kafka設計](system-library-kafka設計.md) — k1s0-kafka ライブラリ
- [system-library-correlation設計](system-library-correlation設計.md) — k1s0-correlation ライブラリ
- [system-library-schemaregistry設計](system-library-schemaregistry設計.md) — k1s0-schemaregistry ライブラリ

---
