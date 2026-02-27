# k1s0-audit-client ライブラリ設計

## 概要

汎用監査ログクライアントライブラリ。テナント・アクター・アクション・リソース情報を含む監査イベントを統一インターフェースで記録し、バッファリング送信により本来の業務処理への影響を最小化する。全 Tier のサービスから共通利用する。

**配置先**: `regions/system/library/rust/audit-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `AuditClient` | トレイト | 監査ログ記録・フラッシュインターフェース |
| `BufferedAuditClient` | 構造体 | メモリバッファリング実装（record→flush で一括取得） |
| `AuditEvent` | 構造体 | id・tenant_id・actor_id・action・resource_type・resource_id・metadata・timestamp |
| `AuditError` | enum | `SerializationError`・`SendError`・`Internal` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-audit-client"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
tokio = { version = "1", features = ["sync"] }
mockall = { version = "0.13", optional = true }
```

**Cargo.toml への追加行**:

```toml
k1s0-audit-client = { path = "../../system/library/rust/audit-client" }
```

**モジュール構成**:

```
audit-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── client.rs       # AuditClient トレイト
│   ├── buffered.rs     # BufferedAuditClient
│   ├── event.rs        # AuditEvent
│   └── error.rs        # AuditError
└── Cargo.toml
```

**データモデル**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub tenant_id: String,
    pub actor_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub metadata: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl AuditEvent {
    pub fn new(
        tenant_id: impl Into<String>,
        actor_id: impl Into<String>,
        action: impl Into<String>,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
        metadata: serde_json::Value,
    ) -> Self;
}
```

**トレイト・実装**:

```rust
#[async_trait]
pub trait AuditClient: Send + Sync {
    async fn record(&self, event: AuditEvent) -> Result<(), AuditError>;
    async fn flush(&self) -> Result<Vec<AuditEvent>, AuditError>;
}

pub struct BufferedAuditClient { /* Mutex<Vec<AuditEvent>> */ }

impl BufferedAuditClient {
    pub fn new() -> Self;
}
```

**エラー型**:

```rust
pub enum AuditError {
    SerializationError(serde_json::Error),
    SendError(String),
    Internal(String),
}
```

**使用例**:

```rust
use k1s0_audit_client::{AuditClient, AuditEvent, BufferedAuditClient};

let client = BufferedAuditClient::new();

let event = AuditEvent::new(
    "tenant-1",
    "user-1",
    "create",
    "document",
    "doc-123",
    serde_json::json!({"key": "value"}),
);

client.record(event).await?;

// バッファ内のイベントを一括取得（バッファはクリアされる）
let events = client.flush().await?;
```

## Go 実装

**配置先**: `regions/system/library/go/audit-client/`

```
audit-client/
├── auditclient.go
├── auditclient_test.go
└── go.mod
```

**主要インターフェース**:

```go
type AuditEvent struct {
    ID           string    `json:"id"`
    TenantID     string    `json:"tenant_id"`
    ActorID      string    `json:"actor_id"`
    Action       string    `json:"action"`
    ResourceType string    `json:"resource_type"`
    ResourceID   string    `json:"resource_id"`
    Timestamp    time.Time `json:"timestamp"`
}

type AuditClient interface {
    Record(ctx context.Context, event AuditEvent) error
    Flush(ctx context.Context) ([]AuditEvent, error)
}

type BufferedClient struct{ /* ... */ }
func NewBufferedClient() *BufferedClient
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/audit-client/`

```
audit-client/
├── package.json        # "@k1s0/audit-client", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # AuditClient, BufferedAuditClient, AuditEvent
└── __tests__/
    └── audit-client.test.ts
```

**主要 API**:

```typescript
export interface AuditEvent {
  id: string;
  tenantId: string;
  actorId: string;
  action: string;
  resourceType: string;
  resourceId: string;
  timestamp: string;
}

export interface AuditClient {
  record(event: AuditEvent): Promise<void>;
  flush(): Promise<AuditEvent[]>;
}

export class BufferedAuditClient implements AuditClient {
  async record(event: AuditEvent): Promise<void>;
  async flush(): Promise<AuditEvent[]>;
}
```

**カバレッジ目標**: 90%以上

## テスト戦略

### ユニットテスト

```rust
#[tokio::test]
async fn test_record_and_flush() {
    let client = BufferedAuditClient::new();
    let event = AuditEvent::new(
        "tenant-1",
        "user-1",
        "create",
        "document",
        "doc-123",
        serde_json::json!({"key": "value"}),
    );
    client.record(event).await.unwrap();
    let flushed = client.flush().await.unwrap();
    assert_eq!(flushed.len(), 1);
    assert_eq!(flushed[0].tenant_id, "tenant-1");
}

#[tokio::test]
async fn test_flush_empties_buffer() {
    let client = BufferedAuditClient::new();
    let event = AuditEvent::new("t", "u", "a", "r", "id", serde_json::json!({}));
    client.record(event).await.unwrap();
    let _ = client.flush().await.unwrap();
    let flushed = client.flush().await.unwrap();
    assert!(flushed.is_empty());
}
```

### モックテスト

```rust
// feature = "mock" 有効時に MockAuditClient が自動生成される
use k1s0_audit_client::MockAuditClient;

#[tokio::test]
async fn test_with_mock() {
    let mut mock = MockAuditClient::new();
    mock.expect_record()
        .once()
        .returning(|_| Ok(()));
    mock.record(event).await.unwrap();
}
```

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](../overview/概要.md) — ライブラリ一覧・テスト方針
- [system-library-serviceauth設計](../auth-security/serviceauth設計.md) — サービス間認証ライブラリ
- [system-library-authlib設計](../auth-security/authlib設計.md) — 認証ライブラリ
- [system-library-correlation設計](correlation設計.md) — トレース ID 伝播（trace_id 連携）
- [認証設計](../../auth/design/認証設計.md) — 認証・認可全体設計
- [可観測性設計](../../observability/overview/可観測性設計.md) — ログ・メトリクス設計
