# k1s0-webhook-client ライブラリ設計

Webhook 配信クライアントライブラリ。HMAC-SHA256 署名付き HTTP POST 配信・指数バックオフリトライ・べき等性をサポートする。

## 概要

HMAC-SHA256 署名付きの HTTP POST 配信、指数バックオフリトライ、べき等性（Idempotency-Key ヘッダー）をサポートする Webhook 配信クライアントライブラリ。notification-server から利用する。

配信リクエストには `X-K1s0-Signature` ヘッダーに HMAC-SHA256 署名を付与し、受信側での改ざん検知を可能にする。リトライは指数バックオフ + ジッターにより過負荷を防止し、`MaxRetriesExceeded` に至るまでのすべての試行を tracing で記録する。`Idempotency-Key` ヘッダーにより重複配信を受信側でフィルタリングできる。

**配置先**: `regions/system/library/rust/webhook-client/`

## 公開 API

| 型・関数 | 種別 | 説明 |
|---------|------|------|
| `WebhookClient` | トレイト | Webhook 配信インターフェース |
| `HttpWebhookClient` | 構造体 | HTTP ベースの WebhookClient 実装（リトライ・べき等性対応）。全言語に存在 |
| `WebhookConfig` | 構造体 | リトライ設定（max_retries、initial_backoff_ms、max_backoff_ms） |
| `WebhookPayload` | 構造体 | イベント種別・タイムスタンプ・データ |
| `generate_signature` | 関数 | HMAC-SHA256 署名生成 |
| `verify_signature` | 関数 | HMAC-SHA256 署名検証 |
| `WebhookError` | enum | Webhook 送信エラー（5バリアント: RequestFailed, SerializationError, SignatureError, Internal, MaxRetriesExceeded） |
| `WebhookErrorCode` | 型 | エラーコード（TS: type union、Dart: enum）。`SEND_FAILED`、`MAX_RETRIES_EXCEEDED` |
| `MaxRetriesExceededError` | 構造体 | リトライ上限到達エラー（Go のみ。Rust は WebhookError::MaxRetriesExceeded バリアント） |
| `InMemoryWebhookClient` | クラス | テスト用スタブ（TS/Dart のみ） |
| `MockWebhookClient` | 構造体 | テスト用モック（Rust のみ、`feature = "mock"`） |

> **設計ノート: 署名パターンの言語間差異**
> - **Rust**: `WebhookClient` トレイトは `send` と `send_with_signature` の 2 メソッドを提供。secret は `send_with_signature` の呼び出し時に引数で渡す。
> - **Go/TS/Dart**: `WebhookClient` インターフェースは `send` メソッドのみ提供。secret はコンストラクタ（Go: `NewHTTPWebhookClient(secret)`、TS: `new HttpWebhookClient({secret})`、Dart: `HttpWebhookClient(secret: ...)`）で注入し、secret が設定されている場合は `send` 時に自動的に署名を付与する。

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
uuid = { version = "1", features = ["v4"] }
rand = "0.8"
tracing = "0.1"
reqwest = { version = "0.12", features = ["json"], default-features = false }
mockall = { version = "0.13", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
wiremock = "0.6"
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

**依存関係**: 標準ライブラリの `crypto/hmac`・`crypto/sha256`・`net/http` を使用

**主要インターフェース**:

```go
const SignatureHeader = "X-K1s0-Signature"
const IdempotencyKeyHeader = "Idempotency-Key"

type WebhookPayload struct {
    EventType string         `json:"event_type"`
    Timestamp string         `json:"timestamp"`
    Data      map[string]any `json:"data"`
}

// WebhookConfig はリトライ設定。
type WebhookConfig struct {
    MaxRetries       int  // デフォルト: 3
    InitialBackoffMs int  // デフォルト: 100
    MaxBackoffMs     int  // デフォルト: 10000
}

func DefaultWebhookConfig() WebhookConfig

// MaxRetriesExceededError はリトライ上限到達エラー。
type MaxRetriesExceededError struct {
    Attempts       int
    LastStatusCode int
}

func (e *MaxRetriesExceededError) Error() string

// WebhookClient インターフェース（send メソッドのみ。secret はコンストラクタで注入）
type WebhookClient interface {
    Send(ctx context.Context, url string, payload *WebhookPayload) (int, error)
}

// HTTPWebhookClient は HTTP ベースの実装（secret 設定時に自動署名）。
type HTTPWebhookClient struct {
    Secret     string
    Config     WebhookConfig
    HTTPClient *http.Client
}

func NewHTTPWebhookClient(secret string) *HTTPWebhookClient
func NewHTTPWebhookClientWithConfig(secret string, config WebhookConfig) *HTTPWebhookClient

func (c *HTTPWebhookClient) Send(ctx context.Context, url string, payload *WebhookPayload) (int, error)

// HMAC-SHA256 署名生成・検証
func GenerateSignature(secret string, body []byte) string
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

export interface WebhookConfig {
  maxRetries?: number;       // デフォルト: 3
  initialBackoffMs?: number; // デフォルト: 1000
  maxBackoffMs?: number;     // デフォルト: 30000
  secret?: string;           // 設定時に send で自動署名
}

export type WebhookErrorCode = 'SEND_FAILED' | 'MAX_RETRIES_EXCEEDED';

export class WebhookError extends Error {
  readonly code: WebhookErrorCode;
  constructor(message: string, code: WebhookErrorCode);
}

export interface WebhookClient {
  send(url: string, payload: WebhookPayload): Promise<number>;
}

// HTTP ベースの実装（secret 設定時に自動署名、リトライ・べき等性対応）
export class HttpWebhookClient implements WebhookClient {
  constructor(config?: WebhookConfig & { secret?: string }, fetchFn?: typeof fetch);
  send(url: string, payload: WebhookPayload): Promise<number>;
}

// テスト用スタブ
export class InMemoryWebhookClient implements WebhookClient {
  send(url: string, payload: WebhookPayload): Promise<number>;
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
  http: ^1.0.0
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

// WebhookConfig — リトライ設定
class WebhookConfig {
  final int maxRetries;       // デフォルト: 3
  final int initialBackoffMs; // デフォルト: 1000
  final int maxBackoffMs;     // デフォルト: 30000
  const WebhookConfig({...});
}

// WebhookErrorCode
enum WebhookErrorCode { sendFailed, maxRetriesExceeded }

// WebhookError
class WebhookError implements Exception {
  final String message;
  final WebhookErrorCode code;
  const WebhookError(this.message, this.code);
}

// WebhookClient — 送信インターフェース（send メソッドのみ）
abstract class WebhookClient {
  Future<int> send(String url, WebhookPayload payload);
}

// HttpWebhookClient — HTTP ベースの実装（secret 設定時に自動署名、リトライ・べき等性対応）
class HttpWebhookClient implements WebhookClient {
  HttpWebhookClient({
    String? secret,
    WebhookConfig config = const WebhookConfig(),
    http.Client? httpClient,
  });
  @override
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

// HttpWebhookClient（署名付き、リトライ対応）
final client = HttpWebhookClient(
  secret: 'my-hmac-secret',
  config: const WebhookConfig(maxRetries: 5),
);

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
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, header_exists};

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
    async fn test_send_with_signature_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header_exists("X-K1s0-Signature"))
            .and(header_exists("Idempotency-Key"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let client = HttpWebhookClient::new();
        let payload = WebhookPayload {
            event_type: "test.event".to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            data: serde_json::json!({}),
        };
        let status = client
            .send_with_signature(&server.uri(), &payload, "secret")
            .await
            .unwrap();
        assert_eq!(status, 200);
    }

    #[tokio::test]
    async fn test_max_retries_exceeded() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let config = WebhookConfig {
            max_retries: 2,
            initial_backoff_ms: 10,
            max_backoff_ms: 100,
        };
        let client = HttpWebhookClient::with_config(config);
        let payload = WebhookPayload {
            event_type: "test.event".to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            data: serde_json::json!({}),
        };
        let result = client.send(&server.uri(), &payload).await;
        assert!(matches!(result, Err(WebhookError::MaxRetriesExceeded { .. })));
    }
}
```

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-retry設計](../resilience/retry.md) — k1s0-retry ライブラリ（リトライロジック）
- [system-library-idempotency設計](../resilience/idempotency.md) — k1s0-idempotency ライブラリ
- [system-library-cache設計](../data/cache.md) — k1s0-cache ライブラリ
