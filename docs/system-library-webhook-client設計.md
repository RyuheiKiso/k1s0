# k1s0-webhook-client ライブラリ設計

Webhook 配信クライアントライブラリ。HMAC-SHA256 署名付き HTTP POST 配信・指数バックオフリトライ・べき等性をサポートする。

## 概要

HMAC-SHA256 署名付きの HTTP POST 配信、指数バックオフリトライ、べき等性（Idempotency-Key ヘッダー）をサポートする Webhook 配信クライアントライブラリ。notification-server から利用する。

配信リクエストには `X-K1s0-Signature` ヘッダーに HMAC-SHA256 署名を付与し、受信側での改ざん検知を可能にする。リトライは指数バックオフ + ジッターにより過負荷を防止し、`MaxRetriesExceeded` に至るまでのすべての試行を tracing で記録する。`Idempotency-Key` ヘッダーにより重複配信を受信側でフィルタリングできる。

**配置先**: `regions/system/library/rust/webhook-client/`

## 公開 API

| 型・関数 | 種別 | 説明 |
|---------|------|------|
| `WebhookClient` | 構造体 | Webhook 配信クライアント（reqwest ベース）|
| `WebhookConfig` | 構造体 | エンドポイント URL・シークレット・タイムアウト・リトライ設定 |
| `WebhookPayload` | 構造体 | イベント種別・データ・タイムスタンプ・Idempotency-Key |
| `WebhookResponse` | 構造体 | HTTP ステータスコード・レスポンスボディ |
| `WebhookSignature` | 構造体 | HMAC-SHA256 署名生成・検証 |
| `WebhookError` | enum | `DeliveryFailed`・`SignatureError`・`Timeout`・`MaxRetriesExceeded` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-webhook-client"
version = "0.1.0"
edition = "2021"

[dependencies]
reqwest = { version = "0.12", features = ["json"] }
hmac = "0.12"
sha2 = "0.10"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["time"] }
thiserror = "2"
tracing = "0.1"
uuid = { version = "1", features = ["v4"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
wiremock = "0.6"
```

**Cargo.toml への追加行**:

```toml
k1s0-webhook-client = { path = "../../system/library/rust/webhook-client" }
```

**モジュール構成**:

```
webhook-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── client.rs       # WebhookClient
│   ├── config.rs       # WebhookConfig
│   ├── payload.rs      # WebhookPayload・WebhookResponse
│   ├── signature.rs    # WebhookSignature（HMAC-SHA256）
│   └── error.rs        # WebhookError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_webhook_client::{WebhookClient, WebhookConfig, WebhookPayload};
use std::time::Duration;
use serde_json::json;

// クライアントの構築
let config = WebhookConfig::new("https://example.com/webhooks")
    .with_secret("my-hmac-secret")
    .with_timeout(Duration::from_secs(10))
    .with_max_retries(3);

let client = WebhookClient::new(config);

// Webhook 配信
let payload = WebhookPayload::new("order.created", json!({
    "order_id": "ord_123",
    "amount": 4900,
}));

// 署名・Idempotency-Key を自動付与して送信（失敗時はリトライ）
let response = client.deliver(payload).await?;
println!("delivered: status={}", response.status_code);

// 受信側での署名検証
use k1s0_webhook_client::WebhookSignature;
let is_valid = WebhookSignature::verify(
    "my-hmac-secret",
    request_body_bytes,
    &request_headers["X-K1s0-Signature"],
)?;
```

## Go 実装

**配置先**: `regions/system/library/go/webhook-client/`

```
webhook-client/
├── webhook.go
├── client.go
├── config.go
├── payload.go
├── signature.go
├── webhook_test.go
├── go.mod
└── go.sum
```

**依存関係**: `github.com/stretchr/testify v1.10.0`（標準ライブラリの `crypto/hmac`・`net/http` を使用）

**主要インターフェース**:

```go
type WebhookClient struct{}

func NewWebhookClient(config WebhookConfig) *WebhookClient

func (c *WebhookClient) Deliver(ctx context.Context, payload WebhookPayload) (*WebhookResponse, error)

type WebhookConfig struct {
    EndpointURL string
    Secret      string
    Timeout     time.Duration
    MaxRetries  int
}

type WebhookPayload struct {
    EventType      string          `json:"event_type"`
    Data           json.RawMessage `json:"data"`
    Timestamp      time.Time       `json:"timestamp"`
    IdempotencyKey string          `json:"idempotency_key"`
}

// HMAC-SHA256 署名生成
func Sign(secret string, body []byte) string

// 署名検証
func Verify(secret string, body []byte, signature string) bool
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/webhook-client/`

```
webhook-client/
├── package.json        # "@k1s0/webhook-client", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # WebhookClient, WebhookConfig, WebhookPayload, WebhookSignature, WebhookError
└── __tests__/
    └── webhook.test.ts
```

**主要 API**:

```typescript
export interface WebhookConfig {
  endpointUrl: string;
  secret: string;
  timeoutMs?: number;
  maxRetries?: number;
}

export interface WebhookPayload {
  eventType: string;
  data: unknown;
  timestamp?: Date;
  idempotencyKey?: string;
}

export interface WebhookResponse {
  statusCode: number;
  body: string;
}

export class WebhookClient {
  constructor(config: WebhookConfig);
  deliver(payload: WebhookPayload): Promise<WebhookResponse>;
}

export class WebhookSignature {
  static sign(secret: string, body: string): string;
  static verify(secret: string, body: string, signature: string): boolean;
}

export class WebhookError extends Error {
  readonly kind: 'DeliveryFailed' | 'SignatureError' | 'Timeout' | 'MaxRetriesExceeded';
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/webhook-client/`

```
webhook-client/
├── pubspec.yaml        # k1s0_webhook_client
├── analysis_options.yaml
├── lib/
│   ├── webhook_client.dart
│   └── src/
│       ├── client.dart       # WebhookClient
│       ├── config.dart       # WebhookConfig
│       ├── payload.dart      # WebhookPayload・WebhookResponse
│       ├── signature.dart    # WebhookSignature（HMAC-SHA256）
│       └── error.dart        # WebhookError
└── test/
    └── webhook_client_test.dart
```

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  http: ^1.3.0
  crypto: ^3.0.6
  uuid: ^4.5.1
```

**使用例**:

```dart
import 'package:k1s0_webhook_client/webhook_client.dart';

final config = WebhookConfig(
  endpointUrl: 'https://example.com/webhooks',
  secret: 'my-hmac-secret',
  maxRetries: 3,
);

final client = WebhookClient(config);

final payload = WebhookPayload(
  eventType: 'order.created',
  data: {'order_id': 'ord_123', 'amount': 4900},
);

final response = await client.deliver(payload);
print('status: ${response.statusCode}');
```

**カバレッジ目標**: 90%以上

## C# 実装

**配置先**: `regions/system/library/csharp/webhook-client/`

```
webhook-client/
├── src/
│   ├── WebhookClient.csproj
│   ├── IWebhookClient.cs       # Webhook 配信インターフェース
│   ├── WebhookClient.cs        # reqwest 相当（HttpClient ベース）
│   ├── WebhookConfig.cs        # エンドポイント URL・シークレット・リトライ設定
│   ├── WebhookPayload.cs       # イベント種別・データ・Idempotency-Key
│   ├── WebhookResponse.cs      # HTTP ステータスコード・レスポンスボディ
│   ├── WebhookSignature.cs     # HMAC-SHA256 署名生成・検証
│   └── WebhookException.cs     # 公開例外型
├── tests/
│   ├── WebhookClient.Tests.csproj
│   ├── Unit/
│   │   ├── WebhookSignatureTests.cs
│   │   └── WebhookPayloadTests.cs
│   └── Integration/
│       └── WebhookClientTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| Polly 8.4 | 指数バックオフリトライポリシー |

**名前空間**: `K1s0.System.WebhookClient`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `IWebhookClient` | interface | Webhook 配信インターフェース |
| `WebhookClient` | class | HttpClient ベースの配信実装 |
| `WebhookConfig` | record | エンドポイント URL・シークレット・タイムアウト・リトライ設定 |
| `WebhookPayload` | record | イベント種別・データ・タイムスタンプ・Idempotency-Key |
| `WebhookResponse` | record | HTTP ステータスコード・レスポンスボディ |
| `WebhookSignature` | class | HMAC-SHA256 署名生成・検証（静的メソッド）|
| `WebhookException` | class | 配信エラーの公開例外型 |

**主要 API**:

```csharp
namespace K1s0.System.WebhookClient;

public interface IWebhookClient
{
    Task<WebhookResponse> DeliverAsync(WebhookPayload payload, CancellationToken ct = default);
}

public record WebhookConfig(
    string EndpointUrl,
    string Secret,
    TimeSpan? Timeout = null,
    int MaxRetries = 3);

public record WebhookPayload(
    string EventType,
    object Data,
    DateTimeOffset? Timestamp = null,
    string? IdempotencyKey = null);

public static class WebhookSignature
{
    public static string Sign(string secret, ReadOnlySpan<byte> body);
    public static bool Verify(string secret, ReadOnlySpan<byte> body, string signature);
}
```

**カバレッジ目標**: 90%以上

---

## Swift

### パッケージ構成
- ターゲット: `K1s0WebhookClient`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開API
```swift
// Webhook 配信プロトコル
public protocol WebhookClientProtocol: Sendable {
    func deliver(_ payload: WebhookPayload) async throws -> WebhookResponse
}

// Webhook 設定
public struct WebhookConfig: Sendable {
    public let endpointUrl: URL
    public let secret: String
    public let timeout: Duration
    public let maxRetries: Int
    public init(endpointUrl: URL, secret: String, timeout: Duration = .seconds(10), maxRetries: Int = 3)
}

// Webhook ペイロード
public struct WebhookPayload: Codable, Sendable {
    public let eventType: String
    public let data: AnyJSON
    public let timestamp: Date
    public let idempotencyKey: String
    public init(eventType: String, data: AnyJSON)
}

// HMAC-SHA256 署名
public enum WebhookSignature {
    public static func sign(secret: String, body: Data) -> String
    public static func verify(secret: String, body: Data, signature: String) -> Bool
}
```

### エラー型
```swift
public enum WebhookError: Error, Sendable {
    case deliveryFailed(statusCode: Int, body: String)
    case signatureError(underlying: Error)
    case timeout
    case maxRetriesExceeded(attempts: Int)
}
```

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上

---

## Python 実装

**配置先**: `regions/system/library/python/webhook-client/`

### パッケージ構造

```
webhook-client/
├── pyproject.toml
├── src/
│   └── k1s0_webhook_client/
│       ├── __init__.py       # 公開 API（再エクスポート）
│       ├── client.py         # WebhookClient
│       ├── config.py         # WebhookConfig
│       ├── payload.py        # WebhookPayload・WebhookResponse
│       ├── signature.py      # WebhookSignature（HMAC-SHA256）
│       ├── exceptions.py     # WebhookError
│       └── py.typed
└── tests/
    ├── test_client.py
    └── test_signature.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `WebhookClient` | class | Webhook 配信クライアント（httpx ベース） |
| `WebhookConfig` | dataclass | エンドポイント URL・シークレット・タイムアウト・リトライ設定 |
| `WebhookPayload` | dataclass | イベント種別・データ・タイムスタンプ・Idempotency-Key |
| `WebhookResponse` | dataclass | HTTP ステータスコード・レスポンスボディ |
| `WebhookSignature` | class | HMAC-SHA256 署名生成・検証（静的メソッド） |
| `WebhookError` | Exception | 配信エラー基底クラス |

### 使用例

```python
from k1s0_webhook_client import WebhookClient, WebhookConfig, WebhookPayload

config = WebhookConfig(
    endpoint_url="https://example.com/webhooks",
    secret="my-hmac-secret",
    max_retries=3,
)
client = WebhookClient(config)

payload = WebhookPayload(
    event_type="order.created",
    data={"order_id": "ord_123", "amount": 4900},
)

response = await client.deliver(payload)
print(f"status: {response.status_code}")

# 署名検証
from k1s0_webhook_client import WebhookSignature
is_valid = WebhookSignature.verify("my-hmac-secret", body_bytes, signature_header)
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| httpx | >=0.28 | 非同期 HTTP クライアント |

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- HTTP モック: respx（httpx 専用モック）
- カバレッジ目標: 90%以上
- 実行: `pytest` / `ruff check .`

## テスト戦略

**ユニットテスト** (`#[cfg(test)]`):
- `WebhookSignature::sign` の出力を既知の HMAC-SHA256 ハッシュと照合
- `WebhookSignature::verify` で正しいシークレット・不正なシークレット両方を検証
- `WebhookPayload` に `idempotency_key` が自動付与されることを確認
- リトライ回数が `max_retries` を超えた場合に `MaxRetriesExceeded` エラーが返ることを確認

**統合テスト** (wiremock):
- wiremock でターゲットサーバーをモックし、200・429・500 各応答パターンで配信フローを検証
- 429（Too Many Requests）や 5xx 応答でリトライが発生し指数バックオフが機能することを確認
- `Idempotency-Key` ヘッダーが全リクエストに付与されていることをリクエストキャプチャで検証
- タイムアウト設定がレスポンス遅延に対して正しく機能することを確認

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_sign_verify() {
        let secret = "test-secret";
        let body = b"payload body";
        let sig = WebhookSignature::sign(secret, body);
        assert!(WebhookSignature::verify(secret, body, &sig).unwrap());
    }

    #[test]
    fn test_signature_invalid_secret_fails() {
        let sig = WebhookSignature::sign("correct-secret", b"body");
        let result = WebhookSignature::verify("wrong-secret", b"body", &sig).unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_deliver_success() {
        use wiremock::{MockServer, Mock, ResponseTemplate};
        use wiremock::matchers::{method, path, header_exists};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/webhooks"))
            .and(header_exists("X-K1s0-Signature"))
            .and(header_exists("Idempotency-Key"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let config = WebhookConfig::new(format!("{}/webhooks", server.uri()))
            .with_secret("secret");
        let client = WebhookClient::new(config);
        let payload = WebhookPayload::new("test.event", serde_json::json!({}));
        let resp = client.deliver(payload).await.unwrap();
        assert_eq!(resp.status_code, 200);
    }
}
```

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-retry設計](system-library-retry設計.md) — k1s0-retry ライブラリ（リトライロジック）
- [system-library-idempotency設計](system-library-idempotency設計.md) — k1s0-idempotency ライブラリ
- [system-library-cache設計](system-library-cache設計.md) — k1s0-cache ライブラリ
