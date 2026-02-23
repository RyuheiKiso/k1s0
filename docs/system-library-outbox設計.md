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

## C# 実装

**配置先**: `regions/system/library/csharp/outbox/`

```
outbox/
├── src/
│   ├── Outbox.csproj
│   ├── IOutboxStore.cs            # アウトボックス永続化インターフェース
│   ├── PostgresOutboxStore.cs     # PostgreSQL 実装
│   ├── OutboxProcessor.cs         # BackgroundService 継承ポーリングプロセッサ
│   ├── OutboxMessage.cs           # アウトボックスメッセージ
│   ├── OutboxStatus.cs            # ステータス列挙型（Pending/Published/Failed）
│   └── OutboxException.cs         # 公開例外型
├── tests/
│   ├── Outbox.Tests.csproj
│   ├── Unit/
│   │   ├── OutboxProcessorTests.cs
│   │   └── OutboxMessageTests.cs
│   └── Integration/
│       └── PostgresOutboxStoreTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| Npgsql | PostgreSQL ドライバー |
| Dapper | 軽量 ORM |

**名前空間**: `K1s0.System.Outbox`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `IOutboxStore` | interface | アウトボックスメッセージの永続化抽象 |
| `PostgresOutboxStore` | class | PostgreSQL ベースの OutboxStore 実装 |
| `OutboxProcessor` | class | BackgroundService 継承、指数バックオフリトライ付きポーリングプロセッサ |
| `OutboxMessage` | record | アウトボックスメッセージ（topic・payload・status・retryCount） |
| `OutboxStatus` | enum | `Pending` / `Published` / `Failed` |
| `OutboxException` | class | outbox ライブラリの公開例外型 |

**主要 API**:

```csharp
namespace K1s0.System.Outbox;

public interface IOutboxStore
{
    Task SaveAsync(
        OutboxMessage message,
        CancellationToken cancellationToken = default);

    Task<IReadOnlyList<OutboxMessage>> FetchPendingAsync(
        int limit,
        CancellationToken cancellationToken = default);

    Task MarkPublishedAsync(
        Guid messageId,
        CancellationToken cancellationToken = default);

    Task MarkFailedAsync(
        Guid messageId,
        int retryCount,
        CancellationToken cancellationToken = default);
}

public class OutboxProcessor : BackgroundService
{
    public OutboxProcessor(
        IOutboxStore store,
        IEventProducer producer,
        TimeSpan pollInterval);

    protected override Task ExecuteAsync(
        CancellationToken stoppingToken);
}

public enum OutboxStatus
{
    Pending,
    Published,
    Failed,
}
```

**カバレッジ目標**: 85%以上

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

---

## Swift

### パッケージ構成
- ターゲット: `K1s0Outbox`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開API
```swift
// アウトボックスメッセージ（指数バックオフリトライ対応）
public struct OutboxMessage: Codable, Sendable, Identifiable {
    public let id: UUID
    public let topic: String
    public let key: String?
    public let payload: Data
    public let status: OutboxStatus
    public let retryCount: Int
    public let createdAt: Date
    public let scheduledAt: Date
}

public enum OutboxStatus: String, Codable, Sendable {
    case pending
    case processing
    case published
    case failed
}

// アウトボックスストアプロトコル
public protocol OutboxStore: Sendable {
    func save(_ message: OutboxMessage) async throws
    func fetchPending(limit: Int) async throws -> [OutboxMessage]
    func markPublished(id: UUID) async throws
    func markFailed(id: UUID, reason: String) async throws
}

// イベント発行プロトコル
public protocol OutboxPublisher: Sendable {
    func publish(_ message: OutboxMessage) async throws
}

// アウトボックス処理器（actor で並行安全）
public actor OutboxProcessor {
    public init(store: any OutboxStore, publisher: any OutboxPublisher, pollingInterval: Duration = .seconds(5))
    public func start() async
    public func stop()
}
```

### エラー型
```swift
public enum OutboxError: Error, Sendable {
    case saveFailed(underlying: Error)
    case publishFailed(messageId: UUID, underlying: Error)
    case maxRetriesExceeded(messageId: UUID)
    case storeUnavailable(underlying: Error)
}
```

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上
- [system-library-serviceauth設計](system-library-serviceauth設計.md) — k1s0-serviceauth ライブラリ

---

## Python 実装

**配置先**: `regions/system/library/python/outbox/`

### パッケージ構造

```
outbox/
├── pyproject.toml
├── src/
│   └── k1s0_outbox/
│       ├── __init__.py         # 公開 API（再エクスポート）
│       ├── models.py           # OutboxMessage, OutboxStatus, OutboxConfig, RetryConfig
│       ├── store.py            # OutboxStore ABC
│       ├── processor.py        # OutboxProcessor（asyncio ポーリング）
│       ├── in_memory_store.py  # InMemoryOutboxStore（テスト用）
│       ├── exceptions.py       # OutboxError, OutboxErrorCodes
│       └── py.typed
└── tests/
    ├── test_models.py
    ├── test_processor.py
    ├── test_in_memory_store.py
    └── test_exceptions.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `OutboxStore` | ABC | Outbox ストレージ抽象基底クラス（`save`, `fetch_pending`, `mark_published`, `mark_failed`, `increment_retry`） |
| `InMemoryOutboxStore` | class | テスト用インメモリ実装 |
| `OutboxProcessor` | class | asyncio Task ベースのポーリングプロセッサ（`start`, `stop`, `process_once`） |
| `OutboxMessage` | dataclass | Outbox メッセージ（id, topic, payload, status, retry_count, created_at, updated_at, error_message, headers） |
| `OutboxStatus` | StrEnum | メッセージステータス（PENDING, PUBLISHED, FAILED） |
| `OutboxConfig` | dataclass | Outbox 設定（polling_interval_seconds, batch_size, retry） |
| `RetryConfig` | dataclass | リトライ設定（max_retries, base_delay_seconds, max_delay_seconds, multiplier） |
| `OutboxError` | Exception | エラー基底クラス（code, message, cause） |
| `OutboxErrorCodes` | class | エラーコード定数（SAVE_FAILED, FETCH_FAILED, UPDATE_FAILED, PUBLISH_FAILED） |

### 使用例

```python
import asyncio
from k1s0_outbox import (
    OutboxMessage, OutboxProcessor, OutboxConfig,
)
from k1s0_outbox.in_memory_store import InMemoryOutboxStore

# Outbox メッセージを保存
store = InMemoryOutboxStore()
msg = OutboxMessage(topic="k1s0.system.auth.user-created.v1", payload=b'{"user_id":"u1"}')
asyncio.run(store.save(msg))

# プロセッサーでポーリング発行
class MyPublisher:
    def publish(self, message: OutboxMessage) -> None:
        print(f"Published: {message.topic}")

processor = OutboxProcessor(store, OutboxConfig(batch_size=50))
count = asyncio.run(processor.process_once(MyPublisher()))
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| asyncpg | >=0.29 | PostgreSQL 非同期ドライバー |
| pydantic | >=2.8 | データバリデーション |

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock
- カバレッジ目標: 85%以上
- 実行: `pytest` / `ruff check .`
