# k1s0-webhook-client ライブラリ設計

Webhook 配信クライアントライブラリ。HMAC-SHA256 署名付き HTTP POST 配信・指数バックオフリトライ・べき等性をサポートする。

## 概要

HMAC-SHA256 署名付きの HTTP POST 配信、指数バックオフリトライ、べき等性（Idempotency-Key ヘッダー）をサポートする Webhook 配信クライアントライブラリ。notification-server から利用する。

配信リクエストには `X-K1s0-Signature` ヘッダーに HMAC-SHA256 署名を付与し、受信側での改ざん検知を可能にする。リトライは指数バックオフ + ジッターにより過負荷を防止し、`MaxRetriesExceeded` に至るまでのすべての試行を tracing で記録する。`Idempotency-Key` ヘッダーにより重複配信を受信側でフィルタリングできる。

**配置先**: `regions/system/library/rust/webhook-client/`

## 公開 API

| 型・関数 | 種別 | 説明 |
|---------|------|------|
| `WebhookClient` | トレイト | Webhook 配信インターフェース（`send`・`send_with_signature`） |
| `WebhookPayload` | 構造体 | イベント種別・タイムスタンプ・データ |
| `generate_signature` | 関数 | HMAC-SHA256 署名生成 |
| `verify_signature` | 関数 | HMAC-SHA256 署名検証 |
| `WebhookError` | enum | Webhook 送信エラー |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-webhook-client"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
thiserror = "2"
tokio = { version = "1", features = ["sync", "time", "macros"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
hmac = "0.12"
sha2 = "0.10"
hex = "0.4"
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

**依存追加**: `k1s0-webhook-client = { path = "../../system/library/rust/webhook-client" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
webhook-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── client.rs       # WebhookClient トレイト
│   ├── payload.rs      # WebhookPayload
│   ├── signature.rs    # generate_signature・verify_signature
│   └── error.rs        # WebhookError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_webhook_client::{WebhookClient, WebhookPayload, generate_signature, verify_signature};

// WebhookClient トレイト経由で送信
let status = client.send("https://example.com/webhooks", &payload).await?;

// 署名付き送信
let status = client.send_with_signature(
    "https://example.com/webhooks",
    &payload,
    "my-hmac-secret",
).await?;

// 署名生成・検証
let sig = generate_signature("my-hmac-secret", body_bytes);
let is_valid = verify_signature("my-hmac-secret", body_bytes, &sig);
```

## Go 実装

**配置先**: `regions/system/library/go/webhook-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: 標準ライブラリの `crypto/hmac`・`net/http` を使用

**主要インターフェース**:

```go
type WebhookPayload struct {
    EventType string         `json:"event_type"`
    Timestamp string         `json:"timestamp"`
    Data      map[string]any `json:"data"`
}

type WebhookClient interface {
    Send(ctx context.Context, url string, payload *WebhookPayload) (int, error)
}

type HTTPWebhookClient struct {
    Secret     string
    HTTPClient *http.Client
}

func NewHTTPWebhookClient(secret string) *HTTPWebhookClient

func (c *HTTPWebhookClient) Send(ctx context.Context, url string, payload *WebhookPayload) (int, error)

// HMAC-SHA256 署名生成
// 注: Go 実装は現在 `X-Webhook-Signature` を使用。`X-K1s0-Signature` に統一予定
func GenerateSignature(secret string, body []byte) string

// 署名検証
func VerifySignature(secret string, body []byte, sig string) bool
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/webhook-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export interface WebhookPayload {
  eventType: string;
  timestamp: string;
  data: Record<string, unknown>;
}

export interface WebhookClient {
  send(url: string, payload: WebhookPayload): Promise<number>;
}

export class InMemoryWebhookClient implements WebhookClient {
  async send(url: string, payload: WebhookPayload): Promise<number>;
  getSent(): Array<{ url: string; payload: WebhookPayload }>;
}

export function generateSignature(secret: string, body: string): string;
export function verifySignature(secret: string, body: string, signature: string): boolean;
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/webhook_client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

> **注**: Dart のパッケージ命名慣習によりディレクトリ名はアンダースコア `webhook_client/` を使用（他言語はハイフン `webhook-client/`）。

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  crypto: ^3.0.0
```

**主要 API**:

```dart
import 'package:k1s0_webhook_client/webhook_client.dart';

// WebhookPayload — イベント種別・タイムスタンプ・データ
class WebhookPayload {
  final String eventType;
  final String timestamp;
  final Map<String, dynamic> data;

  const WebhookPayload({
    required this.eventType,
    required this.timestamp,
    required this.data,
  });
}

// WebhookClient — 送信インターフェース（他言語と統一の send(url, payload) パターン）
abstract class WebhookClient {
  Future<int> send(String url, WebhookPayload payload);
}

// InMemoryWebhookClient — テスト用スタブ
class InMemoryWebhookClient implements WebhookClient {
  List<(String, WebhookPayload)> get sent;
  @override
  Future<int> send(String url, WebhookPayload payload);
}

// HMAC-SHA256 署名生成・検証
String generateSignature(String secret, String body);
bool verifySignature(String secret, String body, String signature);
```

**使用例**:

```dart
import 'package:k1s0_webhook_client/webhook_client.dart';

final payload = WebhookPayload(
  eventType: 'order.created',
  timestamp: DateTime.now().toIso8601String(),
  data: {'order_id': 'ord_123', 'amount': 4900},
);

final statusCode = await client.send('https://example.com/webhooks', payload);
print('status: $statusCode');

// 署名生成・検証
final sig = generateSignature('my-hmac-secret', '{"event_type":"order.created"}');
final isValid = verifySignature('my-hmac-secret', '{"event_type":"order.created"}', sig);
```

**カバレッジ目標**: 90%以上

## テスト戦略

**ユニットテスト** (`#[cfg(test)]`):
- `generate_signature` の出力を既知の HMAC-SHA256 ハッシュと照合
- `verify_signature` で正しいシークレット・不正なシークレット両方を検証
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
        let sig = generate_signature(secret, body);
        assert!(verify_signature(secret, body, &sig));
    }

    #[test]
    fn test_signature_invalid_secret_fails() {
        let sig = generate_signature("correct-secret", b"body");
        let result = verify_signature("wrong-secret", b"body", &sig);
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

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-retry設計](../resilience/retry.md) — k1s0-retry ライブラリ（リトライロジック）
- [system-library-idempotency設計](../resilience/idempotency.md) — k1s0-idempotency ライブラリ
- [system-library-cache設計](../data/cache.md) — k1s0-cache ライブラリ
