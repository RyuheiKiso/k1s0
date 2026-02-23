# k1s0-session-client ライブラリ設計

## 概要

system-session-server（ポート 8102）へのセッション管理クライアントライブラリ。セッション作成・取得・更新・失効・ユーザーセッション一覧取得・全セッション失効を統一インターフェースで gRPC 経由で提供する。全 Tier のサービスから共通利用し、JWT 認証と組み合わせてセッション状態の確認・管理を行う。

**配置先**: `regions/system/library/rust/session-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `SessionClient` | トレイト | セッション CRUD・ユーザーセッション管理インターフェース |
| `GrpcSessionClient` | 構造体 | gRPC 経由の session-server 接続実装 |
| `Session` | 構造体 | セッションID・ユーザーID・デバイス情報・有効期限 |
| `CreateSessionRequest` | 構造体 | ユーザーID・デバイスID・デバイス情報・IPアドレス |
| `CreateSessionResponse` | 構造体 | セッションID・ユーザーID・有効期限・作成日時 |
| `RefreshSessionResponse` | 構造体 | セッションID・新しい有効期限 |
| `UserSessions` | 構造体 | セッション一覧・総件数 |
| `SessionError` | enum | `NotFound`・`Expired`・`AlreadyRevoked`・`ServerError`・`Timeout` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-session-client"
version = "0.1.0"
edition = "2021"

[features]
grpc = ["tonic"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
tonic = { version = "0.12", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
mockall = "0.13"
```

**Cargo.toml への追加行**:

```toml
k1s0-session-client = { path = "../../system/library/rust/session-client" }
# gRPC 経由を有効化する場合:
k1s0-session-client = { path = "../../system/library/rust/session-client", features = ["grpc"] }
```

**モジュール構成**:

```
session-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── client.rs       # SessionClient トレイト
│   ├── grpc.rs         # GrpcSessionClient
│   ├── model.rs        # Session・CreateSessionRequest・CreateSessionResponse・RefreshSessionResponse・UserSessions
│   └── error.rs        # SessionError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_session_client::{CreateSessionRequest, GrpcSessionClient, SessionClient};

// クライアントの構築
let client = GrpcSessionClient::new("http://session-server:8080").await?;

// セッションの作成
let request = CreateSessionRequest::new("usr_01JABCDEF1234567890", "device_abc123")
    .device_name("MacBook Pro")
    .device_type("desktop")
    .ip_address("192.168.1.1");

let response = client.create_session(request).await?;
tracing::info!(
    session_id = %response.session_id,
    expires_at = %response.expires_at,
    "セッション作成完了"
);

// セッションの取得・有効性確認
match client.get_session(&response.session_id).await {
    Ok(session) => {
        tracing::info!(
            user_id = %session.user_id,
            device_name = ?session.device_name,
            "セッション有効"
        );
    }
    Err(SessionError::NotFound(_)) => {
        tracing::warn!("セッションが見つからない");
    }
    Err(SessionError::Expired(_)) => {
        tracing::warn!("セッションが有効期限切れ");
    }
    Err(e) => return Err(e.into()),
}

// セッションの更新（TTL延長）
let refreshed = client.refresh_session(&response.session_id).await?;
tracing::info!(expires_at = %refreshed.expires_at, "セッション更新完了");

// ユーザーの全セッション一覧取得
let user_sessions = client.list_user_sessions("usr_01JABCDEF1234567890").await?;
tracing::info!(count = user_sessions.total_count, "セッション一覧取得");

// セッションの失効（ログアウト）
client.revoke_session(&response.session_id).await?;

// ユーザーの全セッション失効（強制ログアウト）
let revoked = client.revoke_all_sessions("usr_01JABCDEF1234567890").await?;
tracing::info!(revoked_count = revoked, "全セッション失効完了");
```

## Go 実装

**配置先**: `regions/system/library/go/session-client/`

```
session-client/
├── session_client.go
├── grpc_client.go
├── model.go
├── session_client_test.go
├── go.mod
└── go.sum
```

**依存関係**: `google.golang.org/grpc v1.70`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type SessionClient interface {
    CreateSession(ctx context.Context, req CreateSessionRequest) (CreateSessionResponse, error)
    GetSession(ctx context.Context, sessionID string) (Session, error)
    RefreshSession(ctx context.Context, sessionID string) (RefreshSessionResponse, error)
    RevokeSession(ctx context.Context, sessionID string) error
    ListUserSessions(ctx context.Context, userID string) (UserSessions, error)
    RevokeAllSessions(ctx context.Context, userID string) (int, error)
}

type CreateSessionRequest struct {
    UserID     string
    DeviceID   string
    DeviceName string
    DeviceType string
    UserAgent  string
    IPAddress  string
}

type Session struct {
    SessionID      string
    UserID         string
    DeviceID       string
    DeviceName     string
    DeviceType     string
    IPAddress      string
    ExpiresAt      time.Time
    CreatedAt      time.Time
    LastAccessedAt time.Time
}

type CreateSessionResponse struct {
    SessionID string
    UserID    string
    DeviceID  string
    ExpiresAt time.Time
    CreatedAt time.Time
}

type RefreshSessionResponse struct {
    SessionID string
    ExpiresAt time.Time
}

type UserSessions struct {
    Sessions   []Session
    TotalCount int
}

type GrpcSessionClient struct{ /* ... */ }
func NewGrpcSessionClient(addr string) (*GrpcSessionClient, error)
```

**使用例**:

```go
client, err := NewGrpcSessionClient("session-server:8080")
if err != nil {
    log.Fatal(err)
}

resp, err := client.CreateSession(ctx, CreateSessionRequest{
    UserID:   "usr_01JABCDEF1234567890",
    DeviceID: "device_abc123",
})
if err != nil {
    return err
}
fmt.Printf("セッション作成: %s, 有効期限: %s\n", resp.SessionID, resp.ExpiresAt)
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/session-client/`

```
session-client/
├── package.json        # "@k1s0/session-client", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # SessionClient, GrpcSessionClient, Session, CreateSessionRequest, CreateSessionResponse, RefreshSessionResponse, UserSessions, SessionError
└── __tests__/
    └── session-client.test.ts
```

**主要 API**:

```typescript
export interface CreateSessionRequest {
  userId: string;
  deviceId: string;
  deviceName?: string;
  deviceType?: string;
  userAgent?: string;
  ipAddress?: string;
}

export interface Session {
  sessionId: string;
  userId: string;
  deviceId: string;
  deviceName?: string;
  deviceType?: string;
  ipAddress?: string;
  expiresAt: string;
  createdAt: string;
  lastAccessedAt: string;
}

export interface CreateSessionResponse {
  sessionId: string;
  userId: string;
  deviceId: string;
  expiresAt: string;
  createdAt: string;
}

export interface RefreshSessionResponse {
  sessionId: string;
  expiresAt: string;
}

export interface UserSessions {
  sessions: Session[];
  totalCount: number;
}

export interface SessionClient {
  createSession(req: CreateSessionRequest): Promise<CreateSessionResponse>;
  getSession(sessionId: string): Promise<Session>;
  refreshSession(sessionId: string): Promise<RefreshSessionResponse>;
  revokeSession(sessionId: string): Promise<void>;
  listUserSessions(userId: string): Promise<UserSessions>;
  revokeAllSessions(userId: string): Promise<number>;
}

export class GrpcSessionClient implements SessionClient {
  constructor(serverUrl: string);
  createSession(req: CreateSessionRequest): Promise<CreateSessionResponse>;
  getSession(sessionId: string): Promise<Session>;
  refreshSession(sessionId: string): Promise<RefreshSessionResponse>;
  revokeSession(sessionId: string): Promise<void>;
  listUserSessions(userId: string): Promise<UserSessions>;
  revokeAllSessions(userId: string): Promise<number>;
  close(): Promise<void>;
}

export class SessionError extends Error {
  constructor(
    message: string,
    public readonly code: 'NOT_FOUND' | 'EXPIRED' | 'ALREADY_REVOKED' | 'SERVER_ERROR' | 'TIMEOUT'
  );
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/session_client/`

```
session_client/
├── pubspec.yaml        # k1s0_session_client
├── analysis_options.yaml
├── lib/
│   ├── session_client.dart
│   └── src/
│       ├── client.dart       # SessionClient abstract, GrpcSessionClient
│       ├── model.dart        # Session, CreateSessionRequest, CreateSessionResponse, etc.
│       └── error.dart        # SessionError
└── test/
    └── session_client_test.dart
```

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  grpc: ^4.0.0
  protobuf: ^3.1.0
```

**使用例**:

```dart
import 'package:k1s0_session_client/session_client.dart';

final client = GrpcSessionClient('session-server:8080');

final response = await client.createSession(CreateSessionRequest(
  userId: 'usr_01JABCDEF1234567890',
  deviceId: 'device_abc123',
  deviceName: 'iPhone 15',
  deviceType: 'mobile',
));
print('セッション作成: ${response.sessionId}');

final session = await client.getSession(response.sessionId);
print('ユーザー: ${session.userId}, 有効期限: ${session.expiresAt}');
```

**カバレッジ目標**: 90%以上

## C# 実装

**配置先**: `regions/system/library/csharp/session-client/`

```
session-client/
├── src/
│   ├── SessionClient.csproj
│   ├── ISessionClient.cs          # セッション管理インターフェース
│   ├── GrpcSessionClient.cs       # gRPC 実装
│   ├── Session.cs                 # Session・CreateSessionRequest・CreateSessionResponse・RefreshSessionResponse・UserSessions
│   └── SessionException.cs        # 公開例外型
├── tests/
│   ├── SessionClient.Tests.csproj
│   ├── Unit/
│   │   └── SessionTests.cs
│   └── Integration/
│       └── GrpcSessionClientTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| Grpc.Net.Client 2.67 | gRPC クライアント |
| Google.Protobuf 3.29 | Protobuf シリアライゼーション |

**名前空間**: `K1s0.System.SessionClient`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `ISessionClient` | interface | セッション管理の抽象インターフェース |
| `GrpcSessionClient` | class | gRPC 経由の session-server 接続実装 |
| `CreateSessionRequest` | record | ユーザーID・デバイスID・デバイス情報・IPアドレス |
| `Session` | record | セッション情報全体 |
| `CreateSessionResponse` | record | セッションID・ユーザーID・有効期限・作成日時 |
| `RefreshSessionResponse` | record | セッションID・新しい有効期限 |
| `UserSessions` | record | セッション一覧・総件数 |
| `SessionException` | class | セッションエラーの公開例外型 |

**主要 API**:

```csharp
namespace K1s0.System.SessionClient;

public record CreateSessionRequest(
    string UserId,
    string DeviceId,
    string? DeviceName = null,
    string? DeviceType = null,
    string? UserAgent = null,
    string? IpAddress = null);

public record Session(
    string SessionId,
    string UserId,
    string DeviceId,
    string? DeviceName,
    string? DeviceType,
    string? IpAddress,
    DateTimeOffset ExpiresAt,
    DateTimeOffset CreatedAt,
    DateTimeOffset LastAccessedAt);

public record CreateSessionResponse(
    string SessionId,
    string UserId,
    string DeviceId,
    DateTimeOffset ExpiresAt,
    DateTimeOffset CreatedAt);

public record RefreshSessionResponse(string SessionId, DateTimeOffset ExpiresAt);

public record UserSessions(IReadOnlyList<Session> Sessions, int TotalCount);

public interface ISessionClient : IAsyncDisposable
{
    Task<CreateSessionResponse> CreateSessionAsync(CreateSessionRequest req, CancellationToken ct = default);
    Task<Session> GetSessionAsync(string sessionId, CancellationToken ct = default);
    Task<RefreshSessionResponse> RefreshSessionAsync(string sessionId, CancellationToken ct = default);
    Task RevokeSessionAsync(string sessionId, CancellationToken ct = default);
    Task<UserSessions> ListUserSessionsAsync(string userId, CancellationToken ct = default);
    Task<int> RevokeAllSessionsAsync(string userId, CancellationToken ct = default);
}

public sealed class GrpcSessionClient : ISessionClient
{
    public GrpcSessionClient(string serverUrl);
    // ... ISessionClient 実装
    public ValueTask DisposeAsync();
}
```

**カバレッジ目標**: 90%以上

---

## Swift

### パッケージ構成
- ターゲット: `K1s0SessionClient`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開 API

```swift
public struct CreateSessionRequest: Sendable {
    public let userId: String
    public let deviceId: String
    public let deviceName: String?
    public let deviceType: String?
    public let ipAddress: String?
    public init(userId: String, deviceId: String, deviceName: String? = nil, deviceType: String? = nil, ipAddress: String? = nil)
}

public struct Session: Sendable {
    public let sessionId: String
    public let userId: String
    public let deviceId: String
    public let deviceName: String?
    public let deviceType: String?
    public let ipAddress: String?
    public let expiresAt: Date
    public let createdAt: Date
    public let lastAccessedAt: Date
}

public struct CreateSessionResponse: Sendable {
    public let sessionId: String
    public let userId: String
    public let deviceId: String
    public let expiresAt: Date
    public let createdAt: Date
}

public protocol SessionClient: Sendable {
    func createSession(_ req: CreateSessionRequest) async throws -> CreateSessionResponse
    func getSession(_ sessionId: String) async throws -> Session
    func refreshSession(_ sessionId: String) async throws -> (sessionId: String, expiresAt: Date)
    func revokeSession(_ sessionId: String) async throws
    func listUserSessions(_ userId: String) async throws -> [Session]
    func revokeAllSessions(_ userId: String) async throws -> Int
}
```

### エラー型

```swift
public enum SessionError: Error, Sendable {
    case notFound(sessionId: String)
    case expired(sessionId: String)
    case alreadyRevoked(sessionId: String)
    case serverError(underlying: Error)
    case timeout
}
```

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上

---

## Python 実装

**配置先**: `regions/system/library/python/session_client/`

### パッケージ構造

```
session_client/
├── pyproject.toml
├── src/
│   └── k1s0_session_client/
│       ├── __init__.py       # 公開 API（再エクスポート）
│       ├── client.py         # SessionClient ABC・GrpcSessionClient
│       ├── model.py          # Session・CreateSessionRequest・CreateSessionResponse dataclass
│       ├── exceptions.py     # SessionError
│       └── py.typed
└── tests/
    ├── test_session_client.py
    └── test_session_lifecycle.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `SessionClient` | ABC | セッション CRUD 抽象基底クラス |
| `GrpcSessionClient` | class | gRPC 経由の session-server 接続実装 |
| `CreateSessionRequest` | dataclass | ユーザーID・デバイスID・デバイス情報 |
| `Session` | dataclass | セッション情報 |
| `CreateSessionResponse` | dataclass | セッションID・有効期限 |
| `SessionError` | Exception | 基底エラークラス |

### 使用例

```python
from k1s0_session_client import CreateSessionRequest, GrpcSessionClient, SessionError

client = GrpcSessionClient(server_url="http://session-server:8080")

# セッション作成
response = await client.create_session(CreateSessionRequest(
    user_id="usr_01JABCDEF1234567890",
    device_id="device_abc123",
    device_name="MacBook Pro",
    device_type="desktop",
    ip_address="192.168.1.1",
))
print(f"セッション作成: {response.session_id}")

# セッション取得
try:
    session = await client.get_session(response.session_id)
    print(f"有効期限: {session.expires_at}")
except SessionError as e:
    print(f"エラー: {e}")

# セッション失効
await client.revoke_session(response.session_id)
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| grpcio | >=1.70 | gRPC クライアント |
| grpcio-tools | >=1.70 | Protobuf コード生成 |
| pydantic | >=2.10 | リクエスト・レスポンスバリデーション |

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock
- カバレッジ目標: 90%以上
- 実行: `pytest` / `ruff check .`

## テスト戦略

### ユニットテスト（`#[cfg(test)]`）

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session_request_builder() {
        let req = CreateSessionRequest::new("user-1", "device-1")
            .device_name("MacBook Pro")
            .device_type("desktop");

        assert_eq!(req.user_id, "user-1");
        assert_eq!(req.device_name, Some("MacBook Pro".to_string()));
    }

    #[test]
    fn test_session_error_types() {
        let err = SessionError::NotFound("sess-1".to_string());
        assert!(matches!(err, SessionError::NotFound(_)));
    }
}
```

### 統合テスト

- `testcontainers` で session-server コンテナを起動して実際のセッションライフサイクル（作成→取得→更新→失効）を検証
- 有効期限切れセッションへのアクセスで `Expired` エラーが返ることを確認
- 失効済みセッションへの再失効操作で `AlreadyRevoked` エラーが返ることを確認
- 存在しないセッションIDで `NotFound` エラーが返ることを確認

### モックテスト

```rust
use mockall::mock;

mock! {
    pub TestSessionClient {}
    #[async_trait]
    impl SessionClient for TestSessionClient {
        async fn create_session(&self, req: CreateSessionRequest) -> Result<CreateSessionResponse, SessionError>;
        async fn get_session(&self, session_id: &str) -> Result<Session, SessionError>;
        async fn refresh_session(&self, session_id: &str) -> Result<RefreshSessionResponse, SessionError>;
        async fn revoke_session(&self, session_id: &str) -> Result<(), SessionError>;
        async fn list_user_sessions(&self, user_id: &str) -> Result<UserSessions, SessionError>;
        async fn revoke_all_sessions(&self, user_id: &str) -> Result<u32, SessionError>;
    }
}

#[tokio::test]
async fn test_logout_revokes_session() {
    let mut mock = MockTestSessionClient::new();
    mock.expect_revoke_session()
        .withf(|id| id == "sess-001")
        .once()
        .returning(|_| Ok(()));

    let auth_service = AuthService::new(Arc::new(mock));
    auth_service.logout("sess-001").await.unwrap();
}
```

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-session-server設計](system-session-server設計.md) — セッションサーバー設計
- [system-library-authlib設計](system-library-authlib設計.md) — JWT 認証ライブラリ（JWT 検証と組み合わせて利用）
- [system-library-cache設計](system-library-cache設計.md) — Redis キャッシュライブラリ（セッションストアの内部実装）
- [認証設計.md](認証設計.md) — 認証設計
- [JWT設計.md](JWT設計.md) — JWT 設計
