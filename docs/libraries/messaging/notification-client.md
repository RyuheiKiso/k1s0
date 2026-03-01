# k1s0-notification-client ライブラリ設計

## 概要

汎用通知送信クライアントライブラリ。チャネル種別（Email/SMS/Push/Slack/Webhook）・受信者・件名・本文を統一インターフェースで扱い、`NotificationRequest` / `NotificationResponse` を通じて通知を送信する。全 Tier のサービスから共通利用する。

**配置先**: `regions/system/library/rust/notification-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `NotificationClient` | トレイト | 通知送信インターフェース（send・send_batch） |
| `NotificationRequest` | 構造体 | id・channel・recipient・subject（任意）・body・metadata（任意） |
| `NotificationResponse` | 構造体 | id・status・message_id（任意） |
| `NotificationChannel` | enum | `Email`・`Sms`・`Push`・`Slack`・`Webhook` |
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

**依存追加**: `k1s0-notification-client = { path = "../../system/library/rust/notification-client" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

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
    Slack,
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

**配置先**: `regions/system/library/go/notification-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要インターフェース**:

```go
type Channel string

const (
    ChannelEmail   Channel = "email"
    ChannelSMS     Channel = "sms"
    ChannelPush    Channel = "push"
    ChannelSlack   Channel = "slack"
    ChannelWebhook Channel = "webhook"
)

type NotificationRequest struct {
    ID        string                 `json:"id"`
    Channel   Channel                `json:"channel"`
    Recipient string                 `json:"recipient"`
    Subject   string                 `json:"subject,omitempty"`
    Body      string                 `json:"body"`
    Metadata  map[string]interface{} `json:"metadata,omitempty"`
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

> **注**: Go 実装には `SendBatch` メソッドがない（Rust / Dart のみ `send_batch` を提供）。TypeScript も同様に `send` のみ。一括送信が必要な場合はループで `Send` を呼び出す。

## TypeScript 実装

**配置先**: `regions/system/library/typescript/notification-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export type NotificationChannel = 'email' | 'sms' | 'push' | 'slack' | 'webhook';

export interface NotificationRequest {
  id: string;
  channel: NotificationChannel;
  recipient: string;
  subject?: string;
  body: string;
  metadata?: Record<string, unknown>;
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

## Dart 実装

**配置先**: `regions/system/library/dart/notification_client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```dart
enum NotificationChannel { email, sms, push, slack, webhook }

class NotificationRequest {
  final String id;
  final NotificationChannel channel;
  final String recipient;
  final String? subject;
  final String body;
  final Map<String, dynamic>? metadata;

  NotificationRequest({
    required this.channel,
    required this.recipient,
    required this.body,
    this.subject,
    this.metadata,
  });
}

class NotificationResponse {
  final String id;
  final String status;
  final String? messageId;
}

abstract class NotificationClient {
  Future<NotificationResponse> send(NotificationRequest request);
  Future<List<NotificationResponse>> sendBatch(List<NotificationRequest> requests);
}

class InMemoryNotificationClient implements NotificationClient {
  final List<NotificationRequest> sent = [];
  @override
  Future<NotificationResponse> send(NotificationRequest request);
  @override
  Future<List<NotificationResponse>> sendBatch(List<NotificationRequest> requests);
}
```

**カバレッジ目標**: 90%以上

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

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-eventstore設計](../data/eventstore.md) — イベント永続化ライブラリ
- [system-library-correlation設計](../observability/correlation.md) — トレース ID 伝播
