# k1s0-session-client ライブラリ設計

## 概要

セッション管理クライアントライブラリ。セッション作成・取得・更新・失効・ユーザーセッション一覧取得・全セッション失効を統一インターフェースで提供する。セッションはトークンベースで管理し、任意のメタデータ（key-value）を付与可能。全 Tier のサービスから共通利用し、JWT 認証と組み合わせてセッション状態の確認・管理を行う。

**配置先**: `regions/system/library/rust/session-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `SessionClient` | トレイト | セッション CRUD・ユーザーセッション管理インターフェース |
| `InMemorySessionClient` | 構造体 | メモリ内セッション管理実装（テスト・開発用） |
| `Session` | 構造体 | id・user_id・token・expires_at・created_at・revoked・metadata |
| `CreateSessionRequest` | 構造体 | user_id・ttl_seconds・metadata |
| `RefreshSessionRequest` | 構造体 | id・ttl_seconds |
| `SessionError` | enum | `NotFound`・`Expired`・`Revoked`・`Connection`・`Internal` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-session-client"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
tokio = { version = "1", features = ["sync"] }
mockall = { version = "0.13", optional = true }
```

**Cargo.toml への追加行**:

```toml
k1s0-session-client = { path = "../../system/library/rust/session-client" }
```

**モジュール構成**:

```
session-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── client.rs       # SessionClient トレイト・InMemorySessionClient
│   ├── model.rs        # Session・CreateSessionRequest・RefreshSessionRequest
│   └── error.rs        # SessionError
└── Cargo.toml
```

**データモデル**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked: bool,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CreateSessionRequest {
    pub user_id: String,
    pub ttl_seconds: i64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RefreshSessionRequest {
    pub id: String,
    pub ttl_seconds: i64,
}
```

**トレイト**:

```rust
#[async_trait]
pub trait SessionClient: Send + Sync {
    async fn create(&self, req: CreateSessionRequest) -> Result<Session, SessionError>;
    async fn get(&self, id: &str) -> Result<Option<Session>, SessionError>;
    async fn refresh(&self, req: RefreshSessionRequest) -> Result<Session, SessionError>;
    async fn revoke(&self, id: &str) -> Result<(), SessionError>;
    async fn list_user_sessions(&self, user_id: &str) -> Result<Vec<Session>, SessionError>;
    async fn revoke_all(&self, user_id: &str) -> Result<u32, SessionError>;
}
```

**エラー型**:

```rust
pub enum SessionError {
    NotFound(String),
    Expired,
    Revoked,
    Connection(String),
    Internal(String),
}
```

**使用例**:

```rust
use k1s0_session_client::{
    CreateSessionRequest, InMemorySessionClient, RefreshSessionRequest, SessionClient,
};
use std::collections::HashMap;

let client = InMemorySessionClient::new();

// セッションの作成
let session = client.create(CreateSessionRequest {
    user_id: "user-1".to_string(),
    ttl_seconds: 3600,
    metadata: HashMap::new(),
}).await?;
tracing::info!(id = %session.id, token = %session.token, "セッション作成完了");

// セッションの取得
if let Some(session) = client.get(&session.id).await? {
    tracing::info!(user_id = %session.user_id, "セッション有効");
}

// セッションの更新（TTL延長）
let refreshed = client.refresh(RefreshSessionRequest {
    id: session.id.clone(),
    ttl_seconds: 7200,
}).await?;
tracing::info!(expires_at = %refreshed.expires_at, "セッション更新完了");

// ユーザーの全セッション一覧取得
let sessions = client.list_user_sessions("user-1").await?;
tracing::info!(count = sessions.len(), "セッション一覧取得");

// セッションの失効（ログアウト）
client.revoke(&session.id).await?;

// ユーザーの全セッション失効（強制ログアウト）
let revoked_count = client.revoke_all("user-1").await?;
tracing::info!(revoked_count = revoked_count, "全セッション失効完了");
```

## Go 実装

**配置先**: `regions/system/library/go/session-client/`

```
session-client/
├── session_client.go
├── session_client_test.go
└── go.mod
```

**主要インターフェース**:

```go
type Session struct {
    ID        string            `json:"id"`
    UserID    string            `json:"user_id"`
    Token     string            `json:"token"`
    ExpiresAt time.Time         `json:"expires_at"`
    CreatedAt time.Time         `json:"created_at"`
    Revoked   bool              `json:"revoked"`
    Metadata  map[string]string `json:"metadata,omitempty"`
}

type CreateSessionRequest struct {
    UserID     string            `json:"user_id"`
    TTLSeconds int64             `json:"ttl_seconds"`
    Metadata   map[string]string `json:"metadata,omitempty"`
}

type RefreshSessionRequest struct {
    ID         string `json:"id"`
    TTLSeconds int64  `json:"ttl_seconds"`
}

type SessionClient interface {
    Create(ctx context.Context, req CreateSessionRequest) (*Session, error)
    Get(ctx context.Context, id string) (*Session, error)
    Refresh(ctx context.Context, req RefreshSessionRequest) (*Session, error)
    Revoke(ctx context.Context, id string) error
    ListUserSessions(ctx context.Context, userID string) ([]*Session, error)
    RevokeAll(ctx context.Context, userID string) (int, error)
}

type InMemorySessionClient struct{ /* ... */ }
func NewInMemorySessionClient() *InMemorySessionClient
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/session-client/`

```
session-client/
├── package.json        # "@k1s0/session-client", "type":"module"
├── tsconfig.json
├── src/
│   ├── types.ts        # Session, CreateSessionRequest, RefreshSessionRequest
│   ├── client.ts       # SessionClient, InMemorySessionClient
│   └── index.ts
└── __tests__/
    └── client.test.ts
```

**主要 API**:

```typescript
export interface Session {
  id: string;
  userId: string;
  token: string;
  expiresAt: Date;
  createdAt: Date;
  revoked: boolean;
  metadata: Record<string, string>;
}

export interface CreateSessionRequest {
  userId: string;
  ttlSeconds: number;
  metadata?: Record<string, string>;
}

export interface RefreshSessionRequest {
  id: string;
  ttlSeconds: number;
}

export interface SessionClient {
  create(req: CreateSessionRequest): Promise<Session>;
  get(id: string): Promise<Session | null>;
  refresh(req: RefreshSessionRequest): Promise<Session>;
  revoke(id: string): Promise<void>;
  listUserSessions(userId: string): Promise<Session[]>;
  revokeAll(userId: string): Promise<number>;
}

export class InMemorySessionClient implements SessionClient {
  // 全メソッド実装
}
```

**カバレッジ目標**: 90%以上

## テスト戦略

### ユニットテスト（Rust）

```rust
#[tokio::test]
async fn test_create() {
    let client = InMemorySessionClient::new();
    let session = client
        .create(CreateSessionRequest {
            user_id: "user-1".to_string(),
            ttl_seconds: 3600,
            metadata: HashMap::new(),
        })
        .await
        .unwrap();

    assert_eq!(session.user_id, "user-1");
    assert!(!session.revoked);
    assert!(!session.id.is_empty());
    assert!(!session.token.is_empty());
}

#[tokio::test]
async fn test_revoke_all() {
    let client = InMemorySessionClient::new();
    for _ in 0..2 {
        client.create(CreateSessionRequest {
            user_id: "user-1".to_string(),
            ttl_seconds: 3600,
            metadata: HashMap::new(),
        }).await.unwrap();
    }
    let count = client.revoke_all("user-1").await.unwrap();
    assert_eq!(count, 2);
}
```

### モックテスト

```rust
// feature = "mock" 有効時に MockSessionClient が自動生成される
use k1s0_session_client::MockSessionClient;
```

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-authlib設計](system-library-authlib設計.md) — JWT 認証ライブラリ（JWT 検証と組み合わせて利用）
- [system-library-cache設計](system-library-cache設計.md) — Redis キャッシュライブラリ（セッションストアの内部実装）
- [認証設計.md](認証設計.md) — 認証設計
- [JWT設計.md](JWT設計.md) — JWT 設計
