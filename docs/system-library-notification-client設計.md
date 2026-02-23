# k1s0-notification-client ライブラリ設計

## 概要

system-notification-server への通知送信クライアントライブラリ。gRPC と Kafka 経由の非同期送信を抽象化した `NotificationClient` トレイトを提供する。チャネル種別（メール・SMS・プッシュ通知等）・テンプレート ID・受信者・変数マップを統一インターフェースで扱い、全 Tier のサービスから共通利用する。

**配置先**: `regions/system/library/rust/notification-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `NotificationClient` | トレイト | 通知送信インターフェース |
| `GrpcNotificationClient` | 構造体 | gRPC 経由の同期送信（notification-server へ直接）|
| `KafkaNotificationClient` | 構造体 | Kafka 経由の非同期送信（k1s0.system.notification.requested.v1）|
| `NotificationRequest` | 構造体 | チャネル種別・テンプレート ID・受信者・変数マップ |
| `NotificationResult` | 構造体 | 通知 ID・送信ステータス |
| `NotificationError` | enum | `SendFailed`・`TemplateNotFound`・`InvalidChannel` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-notification-client"
version = "0.1.0"
edition = "2021"

[features]
grpc = ["tonic"]
kafka = ["rdkafka"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
tonic = { version = "0.12", optional = true }
rdkafka = { version = "0.37", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
mockall = "0.13"
```

**Cargo.toml への追加行**:

```toml
k1s0-notification-client = { path = "../../system/library/rust/notification-client" }
# gRPC 経由送信を有効化する場合:
k1s0-notification-client = { path = "../../system/library/rust/notification-client", features = ["grpc"] }
# Kafka 経由送信を有効化する場合:
k1s0-notification-client = { path = "../../system/library/rust/notification-client", features = ["kafka"] }
```

**モジュール構成**:

```
notification-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── client.rs       # NotificationClient トレイト
│   ├── grpc.rs         # GrpcNotificationClient
│   ├── kafka.rs        # KafkaNotificationClient
│   ├── request.rs      # NotificationRequest・NotificationResult
│   └── error.rs        # NotificationError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_notification_client::{
    GrpcNotificationClient, KafkaNotificationClient,
    NotificationClient, NotificationRequest,
};
use std::collections::HashMap;

// gRPC 経由（即時送信）
let client = GrpcNotificationClient::new("http://notification-server:8080").await?;

let mut vars = HashMap::new();
vars.insert("user_name".to_string(), "Alice".to_string());
vars.insert("reset_url".to_string(), "https://example.com/reset/abc123".to_string());

let request = NotificationRequest::new()
    .channel("email")
    .template_id("password-reset")
    .recipient("alice@example.com")
    .variables(vars);

let result = client.send(request).await?;
tracing::info!(notification_id = %result.notification_id, "通知送信完了");

// Kafka 経由（非同期送信）
let kafka_client = KafkaNotificationClient::new("kafka:9092").await?;

let request = NotificationRequest::new()
    .channel("push")
    .template_id("order-shipped")
    .recipient("device-token-xyz")
    .variable("order_id", "ORD-001");

kafka_client.send(request).await?;
```

## Go 実装

**配置先**: `regions/system/library/go/notification-client/`

```
notification-client/
├── notification_client.go
├── grpc_client.go
├── kafka_client.go
├── request.go
├── notification_client_test.go
├── go.mod
└── go.sum
```

**依存関係**: `google.golang.org/grpc v1.70`, `github.com/segmentio/kafka-go v0.4`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type NotificationClient interface {
    Send(ctx context.Context, req NotificationRequest) (NotificationResult, error)
}

type NotificationRequest struct {
    Channel    string
    TemplateID string
    Recipient  string
    Variables  map[string]string
}

type NotificationResult struct {
    NotificationID string
    Status         string
}

type GrpcNotificationClient struct { /* ... */ }
func NewGrpcNotificationClient(addr string) (*GrpcNotificationClient, error)
func (c *GrpcNotificationClient) Send(ctx context.Context, req NotificationRequest) (NotificationResult, error)

type KafkaNotificationClient struct { /* ... */ }
func NewKafkaNotificationClient(brokers string) (*KafkaNotificationClient, error)
func (c *KafkaNotificationClient) Send(ctx context.Context, req NotificationRequest) (NotificationResult, error)
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/notification-client/`

```
notification-client/
├── package.json        # "@k1s0/notification-client", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # NotificationClient, GrpcNotificationClient, KafkaNotificationClient, NotificationRequest, NotificationResult, NotificationError
└── __tests__/
    ├── grpc-client.test.ts
    └── kafka-client.test.ts
```

**主要 API**:

```typescript
export interface NotificationClient {
  send(request: NotificationRequest): Promise<NotificationResult>;
}

export interface NotificationRequest {
  channel: string;
  templateId: string;
  recipient: string;
  variables?: Record<string, string>;
}

export interface NotificationResult {
  notificationId: string;
  status: 'sent' | 'queued' | 'failed';
}

export class GrpcNotificationClient implements NotificationClient {
  constructor(serverUrl: string);
  send(request: NotificationRequest): Promise<NotificationResult>;
  close(): Promise<void>;
}

export class KafkaNotificationClient implements NotificationClient {
  constructor(brokers: string);
  send(request: NotificationRequest): Promise<NotificationResult>;
  close(): Promise<void>;
}

export class NotificationError extends Error {
  constructor(
    message: string,
    public readonly code: 'SEND_FAILED' | 'TEMPLATE_NOT_FOUND' | 'INVALID_CHANNEL'
  );
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/notification-client/`

```
notification-client/
├── pubspec.yaml        # k1s0_notification_client
├── analysis_options.yaml
├── lib/
│   ├── notification_client.dart
│   └── src/
│       ├── client.dart         # NotificationClient abstract, GrpcNotificationClient, KafkaNotificationClient
│       ├── request.dart        # NotificationRequest, NotificationResult
│       └── error.dart          # NotificationError
└── test/
    ├── grpc_client_test.dart
    └── kafka_client_test.dart
```

**カバレッジ目標**: 90%以上

## C# 実装

**配置先**: `regions/system/library/csharp/notification-client/`

```
notification-client/
├── src/
│   ├── NotificationClient.csproj
│   ├── INotificationClient.cs          # 通知送信インターフェース
│   ├── GrpcNotificationClient.cs       # gRPC 実装
│   ├── KafkaNotificationClient.cs      # Kafka 実装
│   ├── NotificationRequest.cs          # リクエスト・レスポンス型
│   └── NotificationException.cs        # 公開例外型
├── tests/
│   ├── NotificationClient.Tests.csproj
│   ├── Unit/
│   │   └── NotificationRequestTests.cs
│   └── Integration/
│       ├── GrpcNotificationClientTests.cs
│       └── KafkaNotificationClientTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| Grpc.Net.Client 2.67 | gRPC クライアント |
| Confluent.Kafka 2.6 | Kafka プロデューサー |

**名前空間**: `K1s0.System.NotificationClient`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `INotificationClient` | interface | 通知送信の抽象インターフェース |
| `GrpcNotificationClient` | class | gRPC 経由の即時送信実装 |
| `KafkaNotificationClient` | class | Kafka 経由の非同期送信実装 |
| `NotificationRequest` | record | チャネル・テンプレート ID・受信者・変数マップ |
| `NotificationResult` | record | 通知 ID・送信ステータス |
| `NotificationException` | class | 通知送信エラーの公開例外型 |

**主要 API**:

```csharp
namespace K1s0.System.NotificationClient;

public interface INotificationClient
{
    Task<NotificationResult> SendAsync(
        NotificationRequest request,
        CancellationToken ct = default);
}

public record NotificationRequest(
    string Channel,
    string TemplateId,
    string Recipient,
    IReadOnlyDictionary<string, string>? Variables = null);

public record NotificationResult(
    string NotificationId,
    string Status);

public sealed class GrpcNotificationClient : INotificationClient, IAsyncDisposable
{
    public GrpcNotificationClient(string serverUrl);
    public Task<NotificationResult> SendAsync(NotificationRequest request, CancellationToken ct = default);
    public ValueTask DisposeAsync();
}

public sealed class KafkaNotificationClient : INotificationClient, IAsyncDisposable
{
    public KafkaNotificationClient(string brokers);
    public Task<NotificationResult> SendAsync(NotificationRequest request, CancellationToken ct = default);
    public ValueTask DisposeAsync();
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
public protocol NotificationClient: Sendable {
    func send(_ request: NotificationRequest) async throws -> NotificationResult
}

public struct NotificationRequest: Sendable {
    public let channel: String
    public let templateId: String
    public let recipient: String
    public let variables: [String: String]

    public init(
        channel: String,
        templateId: String,
        recipient: String,
        variables: [String: String] = [:]
    )
}

public struct NotificationResult: Sendable {
    public let notificationId: String
    public let status: NotificationStatus
}

public enum NotificationStatus: String, Sendable {
    case sent, queued, failed
}
```

### エラー型

```swift
public enum NotificationError: Error, Sendable {
    case sendFailed(underlying: Error)
    case templateNotFound(templateId: String)
    case invalidChannel(channel: String)
}
```

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上

---

## Python 実装

**配置先**: `regions/system/library/python/notification-client/`

### パッケージ構造

```
notification-client/
├── pyproject.toml
├── src/
│   └── k1s0_notification_client/
│       ├── __init__.py           # 公開 API（再エクスポート）
│       ├── client.py             # NotificationClient ABC・GrpcNotificationClient・KafkaNotificationClient
│       ├── request.py            # NotificationRequest dataclass・NotificationResult
│       ├── exceptions.py         # NotificationError
│       └── py.typed
└── tests/
    ├── test_grpc_client.py
    └── test_kafka_client.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `NotificationClient` | ABC | 通知送信抽象基底クラス（`send`） |
| `GrpcNotificationClient` | class | gRPC 経由の即時送信実装 |
| `KafkaNotificationClient` | class | Kafka 経由の非同期送信実装 |
| `NotificationRequest` | dataclass | チャネル・テンプレート ID・受信者・変数マップ |
| `NotificationResult` | dataclass | 通知 ID・送信ステータス |
| `NotificationError` | Exception | 通知送信エラー基底クラス |

### 使用例

```python
import asyncio
from k1s0_notification_client import (
    GrpcNotificationClient,
    KafkaNotificationClient,
    NotificationRequest,
)

# gRPC 経由（即時送信）
client = GrpcNotificationClient(server_url="http://notification-server:8080")

request = NotificationRequest(
    channel="email",
    template_id="password-reset",
    recipient="alice@example.com",
    variables={"user_name": "Alice", "reset_url": "https://example.com/reset/abc123"},
)
result = await client.send(request)
print(f"通知送信完了: {result.notification_id}")

# Kafka 経由（非同期送信）
kafka_client = KafkaNotificationClient(brokers="kafka:9092")

request = NotificationRequest(
    channel="push",
    template_id="order-shipped",
    recipient="device-token-xyz",
    variables={"order_id": "ORD-001"},
)
await kafka_client.send(request)
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| grpcio | >=1.70 | gRPC クライアント |
| kafka-python | >=2.0 | Kafka プロデューサー |
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
    fn test_notification_request_builder() {
        let req = NotificationRequest::new()
            .channel("email")
            .template_id("welcome")
            .recipient("user@example.com")
            .variable("name", "Bob");

        assert_eq!(req.channel, "email");
        assert_eq!(req.template_id, "welcome");
        assert_eq!(req.recipient, "user@example.com");
        assert_eq!(req.variables.get("name").unwrap(), "Bob");
    }

    #[test]
    fn test_invalid_channel_error() {
        let err = NotificationError::InvalidChannel("unknown".to_string());
        assert!(matches!(err, NotificationError::InvalidChannel(_)));
    }
}
```

### 統合テスト

- `testcontainers` で notification-server・Kafka コンテナを起動して実際の送信フローを検証
- gRPC 送信の成功・失敗（テンプレート不存在・チャネル無効）シナリオをカバー
- Kafka トピック（`k1s0.system.notification.requested.v1`）へのメッセージ発行を確認

### モックテスト

```rust
use mockall::mock;

mock! {
    pub TestNotificationClient {}
    #[async_trait]
    impl NotificationClient for TestNotificationClient {
        async fn send(&self, request: NotificationRequest) -> Result<NotificationResult, NotificationError>;
    }
}

#[tokio::test]
async fn test_service_sends_notification_on_order_created() {
    let mut mock_client = MockTestNotificationClient::new();
    mock_client
        .expect_send()
        .once()
        .returning(|_| Ok(NotificationResult {
            notification_id: "notif-001".to_string(),
            status: "sent".to_string(),
        }));

    let service = OrderService::new(Arc::new(mock_client));
    service.create_order(order_request).await.unwrap();
}
```

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-notification-server設計](system-notification-server設計.md) — 通知サーバー設計
- [system-library-kafka設計](system-library-kafka設計.md) — Kafka プロデューサー/コンシューマー設計
- [system-library-eventstore設計](system-library-eventstore設計.md) — イベント永続化ライブラリ
- [system-library-correlation設計](system-library-correlation設計.md) — トレース ID 伝播
