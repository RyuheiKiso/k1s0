# k1s0-audit-client ライブラリ設計

## 概要

system-auth-server の監査ログ API（`POST /api/v1/audit/logs`）への送信クライアントライブラリ。全 Tier のサービスから共通利用し、認証・認可イベントの監査記録を統一する。非同期バッファリング送信とフォールバック（ローカルログ）をサポートし、監査ログの欠落を防ぎながら本来の業務処理への影響を最小化する。

**配置先**: `regions/system/library/rust/audit-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `AuditClient` | トレイト | 監査ログ送信インターフェース |
| `HttpAuditClient` | 構造体 | auth-server の REST API 経由送信 |
| `BufferedAuditClient` | 構造体 | 非同期バッファリング + バッチ送信（高スループット用）|
| `AuditEvent` | 構造体 | event_type・user_id・ip_address・resource・action・result・detail・trace_id |
| `AuditEventType` | enum | `LoginSuccess`・`LoginFailure`・`TokenValidate`・`PermissionDenied`・`ResourceAccess` 等 |
| `AuditClientConfig` | 構造体 | auth-server URL・バッファサイズ・フラッシュ間隔設定 |
| `AuditClientError` | enum | `SendFailed`・`BufferFull`・`SerializationError` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-audit-client"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["sync", "time"] }
thiserror = "2"
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
mockall = "0.13"
wiremock = "0.6"
```

**Cargo.toml への追加行**:

```toml
k1s0-audit-client = { path = "../../system/library/rust/audit-client" }
```

**モジュール構成**:

```
audit-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── client.rs       # AuditClient トレイト
│   ├── http.rs         # HttpAuditClient
│   ├── buffered.rs     # BufferedAuditClient
│   ├── event.rs        # AuditEvent・AuditEventType
│   ├── config.rs       # AuditClientConfig
│   └── error.rs        # AuditClientError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_audit_client::{
    AuditClient, AuditClientConfig, AuditEvent, AuditEventType,
    BufferedAuditClient, HttpAuditClient,
};
use std::time::Duration;

// 即時送信（低スループットサービス向け）
let config = AuditClientConfig::new("http://auth-server:8080");
let client = HttpAuditClient::new(config).await?;

let event = AuditEvent::builder()
    .event_type(AuditEventType::LoginSuccess)
    .user_id("USR-123")
    .ip_address("192.168.1.10")
    .resource("auth")
    .action("login")
    .result("success")
    .trace_id("trace-abc-456")
    .build();

client.record(event).await?;

// バッファリング送信（高スループットサービス向け）
let config = AuditClientConfig::new("http://auth-server:8080")
    .buffer_size(1000)
    .flush_interval(Duration::from_secs(5))
    .with_fallback_logging(true); // auth-server 障害時はローカルログへフォールバック

let buffered = BufferedAuditClient::new(config).await?;

// イベント記録（バッファに積まれ非同期にバッチ送信される）
buffered.record(AuditEvent::builder()
    .event_type(AuditEventType::PermissionDenied)
    .user_id("USR-456")
    .resource("orders")
    .action("delete")
    .result("denied")
    .build()
).await?;
```

## Go 実装

**配置先**: `regions/system/library/go/audit-client/`

```
audit-client/
├── audit_client.go
├── http_client.go
├── buffered_client.go
├── event.go
├── config.go
├── audit_client_test.go
├── go.mod
└── go.sum
```

**依存関係**: `github.com/go-resty/resty/v2 v2.16`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type AuditClient interface {
    Record(ctx context.Context, event AuditEvent) error
    Close() error
}

type AuditEventType string

const (
    LoginSuccess    AuditEventType = "login.success"
    LoginFailure    AuditEventType = "login.failure"
    TokenValidate   AuditEventType = "token.validate"
    PermissionDenied AuditEventType = "permission.denied"
    ResourceAccess  AuditEventType = "resource.access"
)

type AuditEvent struct {
    EventType  AuditEventType `json:"event_type"`
    UserID     string         `json:"user_id"`
    IPAddress  string         `json:"ip_address,omitempty"`
    Resource   string         `json:"resource"`
    Action     string         `json:"action"`
    Result     string         `json:"result"`
    Detail     string         `json:"detail,omitempty"`
    TraceID    string         `json:"trace_id,omitempty"`
    OccurredAt time.Time      `json:"occurred_at"`
}

type AuditClientConfig struct {
    AuthServerURL   string
    BufferSize      int
    FlushInterval   time.Duration
    FallbackLogging bool
}

func NewHttpAuditClient(config AuditClientConfig) (AuditClient, error)
func NewBufferedAuditClient(config AuditClientConfig) (AuditClient, error)
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/audit-client/`

```
audit-client/
├── package.json        # "@k1s0/audit-client", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # AuditClient, HttpAuditClient, BufferedAuditClient, AuditEvent, AuditEventType, AuditClientConfig, AuditClientError
└── __tests__/
    ├── http-client.test.ts
    └── buffered-client.test.ts
```

**主要 API**:

```typescript
export type AuditEventType =
  | 'login.success'
  | 'login.failure'
  | 'token.validate'
  | 'permission.denied'
  | 'resource.access';

export interface AuditEvent {
  eventType: AuditEventType;
  userId: string;
  ipAddress?: string;
  resource: string;
  action: string;
  result: string;
  detail?: string;
  traceId?: string;
  occurredAt?: Date;
}

export interface AuditClientConfig {
  authServerUrl: string;
  bufferSize?: number;
  flushIntervalMs?: number;
  fallbackLogging?: boolean;
}

export interface AuditClient {
  record(event: AuditEvent): Promise<void>;
  close(): Promise<void>;
}

export class HttpAuditClient implements AuditClient {
  constructor(config: AuditClientConfig);
  record(event: AuditEvent): Promise<void>;
  close(): Promise<void>;
}

export class BufferedAuditClient implements AuditClient {
  constructor(config: AuditClientConfig);
  record(event: AuditEvent): Promise<void>;
  flush(): Promise<void>;
  close(): Promise<void>;
}

export class AuditClientError extends Error {
  constructor(
    message: string,
    public readonly code: 'SEND_FAILED' | 'BUFFER_FULL' | 'SERIALIZATION_ERROR'
  );
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/audit-client/`

```
audit-client/
├── pubspec.yaml        # k1s0_audit_client
├── analysis_options.yaml
├── lib/
│   ├── audit_client.dart
│   └── src/
│       ├── client.dart         # AuditClient abstract, HttpAuditClient, BufferedAuditClient
│       ├── event.dart          # AuditEvent, AuditEventType enum
│       ├── config.dart         # AuditClientConfig
│       └── error.dart          # AuditClientError
└── test/
    ├── http_client_test.dart
    └── buffered_client_test.dart
```

**カバレッジ目標**: 90%以上

## C# 実装

**配置先**: `regions/system/library/csharp/audit-client/`

```
audit-client/
├── src/
│   ├── AuditClient.csproj
│   ├── IAuditClient.cs             # 監査ログ送信インターフェース
│   ├── HttpAuditClient.cs          # HttpClient ベースの即時送信実装
│   ├── BufferedAuditClient.cs      # Channel<T> ベースのバッファリング送信実装
│   ├── AuditEvent.cs               # 監査イベント型・AuditEventType enum
│   ├── AuditClientConfig.cs        # auth-server URL・バッファ設定
│   └── AuditClientException.cs     # 公開例外型
├── tests/
│   ├── AuditClient.Tests.csproj
│   ├── Unit/
│   │   ├── AuditEventTests.cs
│   │   └── BufferedAuditClientTests.cs
│   └── Integration/
│       └── HttpAuditClientTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| Microsoft.Extensions.Http 9.0 | HttpClient ファクトリー |

**名前空間**: `K1s0.System.AuditClient`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `IAuditClient` | interface | 監査ログ送信の抽象インターフェース |
| `HttpAuditClient` | class | REST API 経由の即時送信実装 |
| `BufferedAuditClient` | class | Channel<T> ベースの非同期バッファリング送信実装 |
| `AuditEvent` | record | 監査イベントデータ型 |
| `AuditEventType` | enum | LoginSuccess / LoginFailure / TokenValidate / PermissionDenied / ResourceAccess |
| `AuditClientConfig` | record | auth-server URL・バッファサイズ・フラッシュ間隔 |
| `AuditClientException` | class | 監査クライアントエラーの公開例外型 |

**主要 API**:

```csharp
namespace K1s0.System.AuditClient;

public enum AuditEventType
{
    LoginSuccess,
    LoginFailure,
    TokenValidate,
    PermissionDenied,
    ResourceAccess,
}

public record AuditEvent(
    AuditEventType EventType,
    string UserId,
    string Resource,
    string Action,
    string Result,
    string? IpAddress = null,
    string? Detail = null,
    string? TraceId = null,
    DateTimeOffset? OccurredAt = null);

public interface IAuditClient : IAsyncDisposable
{
    Task RecordAsync(AuditEvent auditEvent, CancellationToken ct = default);
}

public record AuditClientConfig(
    string AuthServerUrl,
    int BufferSize = 1000,
    TimeSpan? FlushInterval = null,
    bool FallbackLogging = true);

public sealed class HttpAuditClient : IAuditClient
{
    public HttpAuditClient(AuditClientConfig config, HttpClient? httpClient = null);
    public Task RecordAsync(AuditEvent auditEvent, CancellationToken ct = default);
    public ValueTask DisposeAsync();
}

public sealed class BufferedAuditClient : IAuditClient
{
    public BufferedAuditClient(AuditClientConfig config, HttpClient? httpClient = null);
    public Task RecordAsync(AuditEvent auditEvent, CancellationToken ct = default);
    public Task FlushAsync(CancellationToken ct = default);
    public ValueTask DisposeAsync();
}
```

**カバレッジ目標**: 90%以上

---

## Swift

### パッケージ構成
- ターゲット: `K1s0AuditClient`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開API

```swift
public enum AuditEventType: String, Sendable {
    case loginSuccess = "login.success"
    case loginFailure = "login.failure"
    case tokenValidate = "token.validate"
    case permissionDenied = "permission.denied"
    case resourceAccess = "resource.access"
}

public struct AuditEvent: Sendable {
    public let eventType: AuditEventType
    public let userId: String
    public let resource: String
    public let action: String
    public let result: String
    public let ipAddress: String?
    public let detail: String?
    public let traceId: String?
    public let occurredAt: Date
}

public protocol AuditClient: Sendable {
    func record(_ event: AuditEvent) async throws
}

public struct AuditClientConfig: Sendable {
    public let authServerUrl: URL
    public let bufferSize: Int
    public let flushInterval: Duration
    public let fallbackLogging: Bool
    public init(authServerUrl: URL, bufferSize: Int = 1000,
                flushInterval: Duration = .seconds(5), fallbackLogging: Bool = true)
}
```

### エラー型

```swift
public enum AuditClientError: Error, Sendable {
    case sendFailed(underlying: Error)
    case bufferFull(droppedCount: Int)
    case serializationError(underlying: Error)
}
```

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上

---

## Python 実装

**配置先**: `regions/system/library/python/audit-client/`

### パッケージ構造

```
audit-client/
├── pyproject.toml
├── src/
│   └── k1s0_audit_client/
│       ├── __init__.py       # 公開 API（再エクスポート）
│       ├── client.py         # AuditClient ABC・HttpAuditClient・BufferedAuditClient
│       ├── event.py          # AuditEvent dataclass・AuditEventType enum
│       ├── config.py         # AuditClientConfig
│       ├── exceptions.py     # AuditClientError
│       └── py.typed
└── tests/
    ├── test_http_client.py
    └── test_buffered_client.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `AuditClient` | ABC | 監査ログ送信抽象基底クラス（`record`）|
| `HttpAuditClient` | class | aiohttp ベースの即時送信実装 |
| `BufferedAuditClient` | class | asyncio.Queue ベースのバッファリング送信実装 |
| `AuditEvent` | dataclass | 監査イベントデータ型 |
| `AuditEventType` | Enum | LOGIN_SUCCESS / LOGIN_FAILURE / TOKEN_VALIDATE / PERMISSION_DENIED / RESOURCE_ACCESS |
| `AuditClientConfig` | dataclass | auth-server URL・バッファサイズ・フラッシュ間隔 |
| `AuditClientError` | Exception | 監査クライアントエラー基底クラス |

### 使用例

```python
import asyncio
from k1s0_audit_client import (
    AuditClientConfig,
    AuditEvent,
    AuditEventType,
    BufferedAuditClient,
    HttpAuditClient,
)

# 即時送信
config = AuditClientConfig(auth_server_url="http://auth-server:8080")
client = HttpAuditClient(config)

event = AuditEvent(
    event_type=AuditEventType.LOGIN_SUCCESS,
    user_id="USR-123",
    ip_address="192.168.1.10",
    resource="auth",
    action="login",
    result="success",
    trace_id="trace-abc-456",
)
await client.record(event)

# バッファリング送信
buffered = BufferedAuditClient(
    AuditClientConfig(
        auth_server_url="http://auth-server:8080",
        buffer_size=1000,
        flush_interval=5.0,
        fallback_logging=True,
    )
)
await buffered.start()

await buffered.record(AuditEvent(
    event_type=AuditEventType.PERMISSION_DENIED,
    user_id="USR-456",
    resource="orders",
    action="delete",
    result="denied",
))

await buffered.close()
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| aiohttp | >=3.11 | 非同期 HTTP クライアント |
| pydantic | >=2.10 | イベントデータバリデーション |

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock / aioresponses
- カバレッジ目標: 90%以上
- 実行: `pytest` / `ruff check .`

## テスト戦略

### ユニットテスト（`#[cfg(test)]`）

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_builder() {
        let event = AuditEvent::builder()
            .event_type(AuditEventType::LoginSuccess)
            .user_id("USR-123")
            .resource("auth")
            .action("login")
            .result("success")
            .build();

        assert_eq!(event.user_id, "USR-123");
        assert!(matches!(event.event_type, AuditEventType::LoginSuccess));
    }

    #[test]
    fn test_audit_client_error_variants() {
        let err = AuditClientError::BufferFull { capacity: 1000 };
        assert!(matches!(err, AuditClientError::BufferFull { .. }));
    }

    #[tokio::test]
    async fn test_buffered_client_drops_on_buffer_full() {
        let config = AuditClientConfig::new("http://localhost:9999")
            .buffer_size(2);
        let client = BufferedAuditClient::new(config).await.unwrap();

        for i in 0..5 {
            let event = AuditEvent::builder()
                .event_type(AuditEventType::ResourceAccess)
                .user_id(format!("USR-{i}"))
                .resource("test")
                .action("read")
                .result("success")
                .build();
            let _ = client.record(event).await; // バッファフル時はエラーを返す
        }
    }
}
```

### 統合テスト

- `wiremock` で auth-server のスタブを立て、`POST /api/v1/audit/logs` への実際の HTTP 送信を検証
- バッファリングクライアントのフラッシュ間隔・バッチ送信サイズの動作を確認
- auth-server 障害時にローカルログへフォールバックすることを確認

### モックテスト

```rust
use mockall::mock;

mock! {
    pub TestAuditClient {}
    #[async_trait]
    impl AuditClient for TestAuditClient {
        async fn record(&self, event: AuditEvent) -> Result<(), AuditClientError>;
    }
}

#[tokio::test]
async fn test_auth_middleware_records_login_success() {
    let mut mock_audit = MockTestAuditClient::new();
    mock_audit
        .expect_record()
        .withf(|e| matches!(e.event_type, AuditEventType::LoginSuccess))
        .once()
        .returning(|_| Ok(()));

    let middleware = AuthMiddleware::new(Arc::new(mock_audit));
    middleware.authenticate(valid_credentials).await.unwrap();
}
```

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-serviceauth設計](system-library-serviceauth設計.md) — サービス間認証ライブラリ
- [system-library-authlib設計](system-library-authlib設計.md) — 認証ライブラリ
- [system-library-correlation設計](system-library-correlation設計.md) — トレース ID 伝播（trace_id 連携）
- [認証設計](認証設計.md) — 認証・認可全体設計
- [可観測性設計](可観測性設計.md) — ログ・メトリクス設計
