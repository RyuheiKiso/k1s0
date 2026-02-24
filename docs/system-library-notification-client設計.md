# k1s0-notification-client ライブラリ設計

## 概要

汎用通知送信クライアントライブラリ。チャネル種別（Email/SMS/Push/Webhook）・受信者・件名・本文を統一インターフェースで扱い、`NotificationRequest` / `NotificationResponse` を通じて通知を送信する。全 Tier のサービスから共通利用する。

**配置先**: `regions/system/library/rust/notification-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `NotificationClient` | トレイト | 通知送信インターフェース（send・send_batch） |
| `NotificationRequest` | 構造体 | id・channel・recipient・subject（任意）・body・metadata（任意） |
| `NotificationResponse` | 構造体 | id・status・message_id（任意） |
| `NotificationChannel` | enum | `Email`・`Sms`・`Push`・`Webhook` |
| `NotificationClientError` | enum | `SendError`・`BatchError`・`InvalidChannel`・`Internal` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-notification-client"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
uuid = { version = "1", features = ["v4"] }
mockall = { version = "0.13", optional = true }
```

**Cargo.toml への追加行**:

```toml
k1s0-notification-client = { path = "../../system/library/rust/notification-client" }
```

**モジュール構成**:

```
notification-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── client.rs       # NotificationClient トレイト
│   ├── request.rs      # NotificationChannel・NotificationRequest・NotificationResponse
│   └── error.rs        # NotificationClientError
└── Cargo.toml
```

**データモデル**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationChannel {
    Email,
    Sms,
    Push,
    Webhook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRequest {
    pub id: Uuid,
    pub channel: NotificationChannel,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: String,
    pub metadata: Option<serde_json::Value>,
}

impl NotificationRequest {
    pub fn new(
        channel: NotificationChannel,
        recipient: impl Into<String>,
        body: impl Into<String>,
    ) -> Self;

    pub fn with_subject(mut self, subject: impl Into<String>) -> Self;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub status: String,
    pub message_id: Option<String>,
}
```

**トレイト**:

```rust
#[async_trait]
pub trait NotificationClient: Send + Sync {
    async fn send(
        &self,
        request: NotificationRequest,
    ) -> Result<NotificationResponse, NotificationClientError>;
    async fn send_batch(
        &self,
        requests: Vec<NotificationRequest>,
    ) -> Result<Vec<NotificationResponse>, NotificationClientError>;
}
```

**エラー型**:

```rust
pub enum NotificationClientError {
    SendError(String),
    BatchError(String),
    InvalidChannel(String),
    Internal(String),
}
```

**使用例**:

```rust
use k1s0_notification_client::{
    NotificationClient, NotificationChannel, NotificationRequest,
};

let request = NotificationRequest::new(
    NotificationChannel::Email,
    "user@example.com",
    "Hello! Your order has been shipped.",
)
.with_subject("Order Shipped");

let response = client.send(request).await?;
tracing::info!(id = %response.id, status = %response.status, "通知送信完了");
```

## Go 実装

**配置先**: `regions/system/library/go/notification-client/`

```
notification-client/
├── notificationclient.go
├── notificationclient_test.go
└── go.mod
```

**主要インターフェース**:

```go
type Channel string

const (
    ChannelEmail Channel = "email"
    ChannelSMS   Channel = "sms"
    ChannelPush  Channel = "push"
)

type NotificationRequest struct {
    ID        string  `json:"id"`
    Channel   Channel `json:"channel"`
    Recipient string  `json:"recipient"`
    Subject   string  `json:"subject,omitempty"`
    Body      string  `json:"body"`
}

type NotificationResponse struct {
    ID        string `json:"id"`
    Status    string `json:"status"`
    MessageID string `json:"message_id,omitempty"`
}

type NotificationClient interface {
    Send(ctx context.Context, req NotificationRequest) (NotificationResponse, error)
}

type InMemoryClient struct{ /* ... */ }
func NewInMemoryClient() *InMemoryClient
func (c *InMemoryClient) SentRequests() []NotificationRequest
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/notification-client/`

```
notification-client/
├── package.json        # "@k1s0/notification-client", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # NotificationClient, InMemoryNotificationClient, NotificationRequest, NotificationResponse, NotificationChannel
└── __tests__/
    └── notification-client.test.ts
```

**主要 API**:

```typescript
export type NotificationChannel = 'email' | 'sms' | 'push' | 'webhook';

export interface NotificationRequest {
  id: string;
  channel: NotificationChannel;
  recipient: string;
  subject?: string;
  body: string;
}

export interface NotificationResponse {
  id: string;
  status: string;
  messageId?: string;
}

export interface NotificationClient {
  send(request: NotificationRequest): Promise<NotificationResponse>;
}

export class InMemoryNotificationClient implements NotificationClient {
  async send(request: NotificationRequest): Promise<NotificationResponse>;
  getSent(): NotificationRequest[];
}
```

**カバレッジ目標**: 90%以上

## C# 実装

**配置先**: `regions/system/library/csharp/notification-client/`

```
notification-client/
├── K1s0.System.NotificationClient.csproj
├── INotificationClient.cs
├── InMemoryNotificationClient.cs
├── NotificationTypes.cs
├── tests/
│   ├── K1s0.System.NotificationClient.Tests.csproj
│   └── InMemoryNotificationClientTests.cs
```

**名前空間**: `K1s0.System.NotificationClient`

**主要 API**:

```csharp
namespace K1s0.System.NotificationClient;

public enum NotificationChannel
{
    Email,
    Sms,
    Push,
    Webhook,
}

public record NotificationRequest(
    string Id,
    NotificationChannel Channel,
    string Recipient,
    string? Subject,
    string Body);

public record NotificationResponse(
    string Id,
    string Status,
    string? MessageId);

public interface INotificationClient
{
    Task<NotificationResponse> SendAsync(NotificationRequest request, CancellationToken ct = default);
}

public class InMemoryNotificationClient : INotificationClient
{
    public IReadOnlyList<NotificationRequest> Sent { get; }
    public Task<NotificationResponse> SendAsync(NotificationRequest request, CancellationToken ct = default);
}
```

**カバレッジ目標**: 90%以上

---

## Swift

### パッケージ構成
- ターゲット: `K1s0NotificationClient`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開API

```swift
public enum NotificationChannel: String, Sendable {
    case email
    case sms
    case push
    case webhook
}

public struct NotificationRequest: Sendable {
    public let id: String
    public let channel: NotificationChannel
    public let recipient: String
    public let subject: String?
    public let body: String

    public init(channel: NotificationChannel, recipient: String, body: String, subject: String? = nil)
}

public struct NotificationResponse: Sendable {
    public let id: String
    public let status: String

    public init(id: String, status: String)
}

public protocol NotificationClient: Sendable {
    func send(_ request: NotificationRequest) async throws -> NotificationResponse
}

public actor InMemoryNotificationClient: NotificationClient {
    public init()
    public func sent() -> [NotificationRequest]
    public func send(_ request: NotificationRequest) async throws -> NotificationResponse
}
```

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上

---

## テスト戦略

### ユニットテスト（Rust）

```rust
#[tokio::test]
async fn test_mock_send() {
    let mut mock = MockNotificationClient::new();
    mock.expect_send()
        .times(1)
        .returning(|_req| {
            Box::pin(async move {
                Ok(NotificationResponse {
                    id: Uuid::new_v4(),
                    status: "sent".to_string(),
                    message_id: Some("msg-123".to_string()),
                })
            })
        });

    let request = NotificationRequest::new(
        NotificationChannel::Email,
        "user@example.com",
        "Hello!",
    )
    .with_subject("Test Subject");

    let result = mock.send(request).await.unwrap();
    assert_eq!(result.status, "sent");
}
```

### モックテスト

```rust
// feature = "mock" 有効時に MockNotificationClient が自動生成される
use k1s0_notification_client::MockNotificationClient;
```

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-eventstore設計](system-library-eventstore設計.md) — イベント永続化ライブラリ
- [system-library-correlation設計](system-library-correlation設計.md) — トレース ID 伝播
