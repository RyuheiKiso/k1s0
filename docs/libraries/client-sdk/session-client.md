# k1s0-session-client ライブラリ設計

## 概要

system-session-server（ポート 8102）へのセッション管理クライアントライブラリ。セッション作成・取得・更新・失効・ユーザーセッション一覧取得・全セッション失効を統一インターフェースで提供する。セッションはトークンベースで管理し、任意のメタデータ（key-value）を付与可能。全 Tier のサービスから共通利用し、JWT 認証と組み合わせてセッション状態の確認・管理を行う。

**配置先**: `regions/system/library/{rust,go,typescript,dart}/session-client/`

## 公開 API

### Rust

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `SessionClient` | トレイト | セッション CRUD・ユーザーセッション管理インターフェース |
| `InMemorySessionClient` | 構造体 | メモリ内セッション管理実装（テスト・開発用） |
| `MockSessionClient` | 構造体 | `feature = "mock"` 時のみ有効、mockall による自動生成モック |
| `Session` | 構造体 | id・user_id・token・expires_at・created_at・revoked・metadata |
| `CreateSessionRequest` | 構造体 | user_id・ttl_seconds・metadata |
| `RefreshSessionRequest` | 構造体 | id・ttl_seconds |
| `SessionError` | enum | `NotFound`・`Expired`・`Revoked`・`Connection`・`Internal` |

### Go

| 型 | 種別 | 説明 |
|----|------|------|
| `SessionClient` | interface | セッション CRUD・ユーザーセッション管理インターフェース |
| `InMemorySessionClient` | struct | メモリ内セッション管理実装（テスト・開発用） |
| `Session` | struct | ID・UserID・Token・ExpiresAt・CreatedAt・Revoked・Metadata |
| `CreateSessionRequest` | struct | UserID・TTLSeconds・Metadata |
| `RefreshSessionRequest` | struct | ID・TTLSeconds |

### TypeScript

| 型 | 種別 | 説明 |
|----|------|------|
| `SessionClient` | interface | セッション CRUD・ユーザーセッション管理インターフェース |
| `InMemorySessionClient` | class | メモリ内セッション管理実装（テスト・開発用） |
| `Session` | interface | id・userId・token・expiresAt・createdAt・revoked・metadata |
| `CreateSessionRequest` | interface | userId・ttlSeconds・metadata |
| `RefreshSessionRequest` | interface | id・ttlSeconds |

### Dart

| 型 | 種別 | 説明 |
|----|------|------|
| `SessionClient` | abstract class | セッション CRUD・ユーザーセッション管理インターフェース |
| `InMemorySessionClient` | class | メモリ内セッション管理実装（テスト・開発用） |
| `Session` | class | id・userId・token・expiresAt・createdAt・revoked・metadata |
| `CreateSessionRequest` | class | userId・ttlSeconds・metadata |
| `RefreshSessionRequest` | class | id・ttlSeconds |

## Counts

| 言語 | 公開関数/メソッド | 公開型 | エラー型/定数 |
|------|-----------------|--------|-------------|
| Rust | 8 | 6 | 5 |
| Go | 7 | 5 | 1 |
| TypeScript | 6 | 5 | 2 |
| Dart | 10 | 5 | 1 |
| **合計** | **31** | **21** | **9** |

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

**依存追加**: `k1s0-session-client = { path = "../../system/library/rust/session-client" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

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

> **注記（metadata の必須/任意）**: Rust の `CreateSessionRequest.metadata` は `HashMap<String, String>`（`Option` でないため必須フィールド）。空のメタデータを渡す場合は `HashMap::new()` を明示的に指定する必要がある。Go・TypeScript・Dart では省略可能（任意フィールド）。

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

> **注記（async）**: Rust では全トレイトメソッドが `async fn` として定義されており、`#[async_trait]` マクロを通じて非同期実装となる。呼び出し側は `.await` が必要。

> **注記（revoke_all の戻り値型）**: `revoke_all` の戻り値は Rust のみ `u32`（符号なし 32 ビット整数）。失効数が負にならないことを型で保証するための設計。Go は `int`、TypeScript は `number`、Dart は `int`（符号付き整数）を返す。

**`InMemorySessionClient` コンストラクタ**:

```rust
impl InMemorySessionClient {
    pub fn new() -> Self { /* ... */ }
}

impl Default for InMemorySessionClient {
    fn default() -> Self { Self::new() }
}
```

`InMemorySessionClient::new()` と `Default::default()` は等価。`Default` 実装により `InMemorySessionClient::default()` でも生成可能。

**`MockSessionClient`** (`feature = "mock"` 時のみ):

```rust
// Cargo.toml で feature = "mock" を有効にすると MockSessionClient が生成される
// [features]
// mock = ["mockall"]
use k1s0_session_client::MockSessionClient;
```

mockall の `#[automock]` により `SessionClient` トレイトのモック実装が自動生成される。テスト時に依存性注入用として使用する。

**エラー型**:

```rust
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("session not found: {0}")]
    NotFound(String),
    #[error("session expired")]
    Expired,
    #[error("session revoked")]
    Revoked,
    #[error("connection error: {0}")]
    Connection(String),
    #[error("internal error: {0}")]
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

**配置先**: `regions/system/library/go/session-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

**エラー仕様**:

Go の実装はエラー型を定義せず、インラインエラーを返す。

| 発生箇所 | 条件 | 返却値 |
|----------|------|--------|
| `Get` | セッションが存在しない | `fmt.Errorf("session not found: %s", id)` |
| `Refresh` | セッションが存在しない | `fmt.Errorf("session not found: %s", id)` |
| `Revoke` | セッションが存在しない | `fmt.Errorf("session not found: %s", id)` |

**注意**: `Get` でセッションが存在しない場合は `nil` セッションではなくエラーを返す。Rust の `Result<Option<Session>, SessionError>` と異なり、Go の `(*Session, error)` では not-found はエラーとして通知する（`*Session` が nil になるケースではなく、必ず error が non-nil になる）。

> **注記（Expired・Revoked 状態）**: Go の `InMemorySessionClient` は期限切れ（`ExpiresAt` 超過）や失効済み（`Revoked == true`）のセッションに対して `Get`/`Refresh` で追加チェックを行わず、データをそのまま返す。Rust では `SessionError::Expired` / `SessionError::Revoked` が明示的なエラーバリアントとして存在するが、Go の InMemory 実装にはこれに相当するエラー返却はない。本番実装（HTTP クライアント）ではサーバー側がこれらの状態を検証してエラーを返す想定。

## TypeScript 実装

**配置先**: `regions/system/library/typescript/session-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

**`InMemorySessionClient` コンストラクタ**:

```typescript
const client = new InMemorySessionClient();
```

引数なしのデフォルトコンストラクタで生成する。内部的に `Map<string, Session>` をセッションストアとして保持する。

**エラー仕様**:

TypeScript の実装はエラー型を定義せず、標準の `Error` をスローする。

| 発生箇所 | 条件 | スローされる値 |
|----------|------|--------------|
| `refresh` | セッションが存在しない | `new Error(\`Session not found: ${req.id}\`)` |
| `revoke` | セッションが存在しない | `new Error(\`Session not found: ${id}\`)` |

**注意**: `get` でセッションが存在しない場合は `null` を返す（エラーではない）。`refresh`・`revoke` は存在しない id が渡された場合にエラーをスローする。

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/session_client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**モジュール構成**:

```
session-client/
├── lib/
│   └── src/
│       ├── session.dart         # Session・CreateSessionRequest・RefreshSessionRequest
│       └── session_client.dart  # SessionClient (abstract)・InMemorySessionClient
└── pubspec.yaml
```

**主要 API**:

```dart
// lib/src/session.dart

class Session {
  final String id;
  final String userId;
  final String token;
  final DateTime expiresAt;
  final DateTime createdAt;
  final bool revoked;
  final Map<String, String> metadata;

  const Session({
    required this.id,
    required this.userId,
    required this.token,
    required this.expiresAt,
    required this.createdAt,
    this.revoked = false,
    this.metadata = const {},
  });

  Session copyWith({
    String? id,
    String? userId,
    String? token,
    DateTime? expiresAt,
    DateTime? createdAt,
    bool? revoked,
    Map<String, String>? metadata,
  });
}

class CreateSessionRequest {
  final String userId;
  final int ttlSeconds;
  final Map<String, String>? metadata;

  const CreateSessionRequest({
    required this.userId,
    required this.ttlSeconds,
    this.metadata,
  });
}

class RefreshSessionRequest {
  final String id;
  final int ttlSeconds;

  const RefreshSessionRequest({
    required this.id,
    required this.ttlSeconds,
  });
}

// lib/src/session_client.dart

abstract class SessionClient {
  Future<Session> create(CreateSessionRequest req);
  Future<Session?> get(String id);
  Future<Session> refresh(RefreshSessionRequest req);
  Future<void> revoke(String id);
  Future<List<Session>> listUserSessions(String userId);
  Future<int> revokeAll(String userId);
}

class InMemorySessionClient implements SessionClient {
  // 全メソッド実装
}
```

**エラー仕様**:

Dart の実装はエラー型を定義せず、標準の `StateError` をスローする。

| 発生箇所 | 条件 | スローされる値 |
|----------|------|--------------|
| `refresh` | セッションが存在しない | `StateError('Session not found: ${req.id}')` |

**注意**: `get` でセッションが存在しない場合は `null` を返す（エラーではなく `Future<Session?>`）。Dart の `Session.copyWith` は他言語に対応するメソッドがない Dart 固有の API である。

**依存追加** (`pubspec.yaml`):

```yaml
dependencies:
  k1s0_session_client:
    path: ../../system/library/dart/session_client
```

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

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-authlib設計](../auth-security/authlib.md) — JWT 認証ライブラリ（JWT 検証と組み合わせて利用）
- [system-library-cache設計](../data/cache.md) — Redis キャッシュライブラリ（セッションストアの内部実装）
- [認証設計.md](../../architecture/auth/認証設計.md) — 認証設計
- [JWT設計.md](../../architecture/auth/JWT設計.md) — JWT 設計
