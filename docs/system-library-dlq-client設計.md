# k1s0-dlq-client ライブラリ設計

## 概要

Kafka のデッドレタートピック（`*.dlq.v1`）に送られた処理失敗メッセージを管理する REST クライアント SDK。
DLQ メッセージの一覧取得・詳細取得・再処理・削除・一括再処理を提供する。

**配置先**:
- `regions/system/library/go/dlq/`
- `regions/system/library/rust/dlq/`
- `regions/system/library/typescript/dlq-client/`
- `regions/system/library/dart/dlq_client/`

## 公開 API

| 型・インターフェース | 種別 | 説明 |
|---------------------|------|------|
| `DlqClient` | 構造体/クラス | DLQ 管理サーバーへの REST クライアント |
| `DlqStatus` | enum | DLQ メッセージステータス（`PENDING`・`RETRYING`・`RESOLVED`・`DEAD`） |
| `DlqMessage` | 構造体/インターフェース | DLQ メッセージの詳細情報 |
| `ListDlqMessagesRequest` | 構造体/インターフェース | 一覧取得リクエスト（topic・page・page_size） |
| `ListDlqMessagesResponse` | 構造体/インターフェース | 一覧取得レスポンス（messages・total・page） |
| `RetryDlqMessageResponse` | 構造体/インターフェース | 再処理レスポンス（message_id・status） |
| `DlqError` | 構造体/クラス | DLQ クライアントエラー型 |

## クライアントメソッド

| メソッド | HTTP | パス | 説明 |
|---------|------|------|------|
| `list_messages(topic, page, page_size)` | GET | `/api/v1/dlq/:topic` | トピック別 DLQ メッセージ一覧 |
| `get_message(message_id)` | GET | `/api/v1/dlq/messages/:id` | メッセージ詳細取得 |
| `retry_message(message_id)` | POST | `/api/v1/dlq/messages/:id/retry` | メッセージ再処理 |
| `delete_message(message_id)` | DELETE | `/api/v1/dlq/messages/:id` | メッセージ削除 |
| `retry_all(topic)` | POST | `/api/v1/dlq/:topic/retry-all` | トピック内全メッセージ一括再処理 |

## 型定義

```
DlqMessage {
  id: string (UUID)
  original_topic: string
  error_message: string
  retry_count: int
  max_retries: int
  payload: JSON
  status: DlqStatus
  created_at: datetime
  last_retry_at: datetime (nullable)
}

DlqStatus: PENDING | RETRYING | RESOLVED | DEAD

ListDlqMessagesRequest {
  topic: string
  page: int
  page_size: int
}

ListDlqMessagesResponse {
  messages: []DlqMessage
  total: int
  page: int
}

RetryDlqMessageResponse {
  message_id: string
  status: DlqStatus
}
```

## Go 実装

**配置先**: `regions/system/library/go/dlq/`

```
dlq/
├── client.go     # DlqClient 構造体・HTTP 実装
├── types.go      # DlqStatus・DlqMessage・Request/Response 型
├── error.go      # DlqError
├── dlq_test.go   # ユニットテスト（httptest）
├── go.mod
└── go.sum
```

**主要型**:

```go
type DlqStatus string

const (
    DlqStatusPending   DlqStatus = "PENDING"
    DlqStatusRetrying  DlqStatus = "RETRYING"
    DlqStatusResolved  DlqStatus = "RESOLVED"
    DlqStatusDead      DlqStatus = "DEAD"
)

type DlqClient struct {
    endpoint   string
    httpClient *http.Client
}

func NewDlqClient(endpoint string) *DlqClient
func (c *DlqClient) ListMessages(ctx context.Context, req *ListDlqMessagesRequest) (*ListDlqMessagesResponse, error)
func (c *DlqClient) GetMessage(ctx context.Context, messageID string) (*DlqMessage, error)
func (c *DlqClient) RetryMessage(ctx context.Context, messageID string) (*RetryDlqMessageResponse, error)
func (c *DlqClient) DeleteMessage(ctx context.Context, messageID string) error
func (c *DlqClient) RetryAll(ctx context.Context, topic string) error
```

## Rust 実装

**配置先**: `regions/system/library/rust/dlq/`

```
dlq/
├── src/
│   ├── lib.rs      # 公開 API（再エクスポート）
│   ├── client.rs   # DlqClient（HTTP REST クライアント）
│   ├── types.rs    # DlqStatus・DlqMessage・Request/Response 型
│   └── error.rs    # DlqError
└── Cargo.toml
```

**主要 API**:

```rust
pub struct DlqClient { ... }

impl DlqClient {
    pub fn new(endpoint: &str) -> Self
    pub async fn list_messages(&self, topic: &str, page: u32, page_size: u32) -> Result<ListDlqMessagesResponse>
    pub async fn get_message(&self, message_id: &str) -> Result<DlqMessage>
    pub async fn retry_message(&self, message_id: &str) -> Result<RetryDlqMessageResponse>
    pub async fn delete_message(&self, message_id: &str) -> Result<()>
    pub async fn retry_all(&self, topic: &str) -> Result<()>
}
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/dlq-client/`

```
dlq-client/
├── package.json        # "@k1s0/dlq-client", "type":"module"
├── tsconfig.json       # ES2022, Node16, strict
├── vitest.config.ts    # globals:true
├── src/
│   ├── types.ts        # DlqStatus, DlqMessage, Request/Response 型
│   ├── error.ts        # DlqError クラス
│   ├── client.ts       # DlqClient クラス
│   └── index.ts        # re-export
└── __tests__/
    └── client.test.ts
```

**主要 API**:

```typescript
export type DlqStatus = 'PENDING' | 'RETRYING' | 'RESOLVED' | 'DEAD';

export interface DlqMessage {
  id: string;
  originalTopic: string;
  errorMessage: string;
  retryCount: number;
  maxRetries: number;
  payload: unknown;
  status: DlqStatus;
  createdAt: string;
  lastRetryAt: string | null;
}

export class DlqClient {
  constructor(endpoint: string);
  listMessages(topic: string, page: number, pageSize: number): Promise<ListDlqMessagesResponse>;
  getMessage(messageId: string): Promise<DlqMessage>;
  retryMessage(messageId: string): Promise<RetryDlqMessageResponse>;
  deleteMessage(messageId: string): Promise<void>;
  retryAll(topic: string): Promise<void>;
}
```

## Dart 実装

**配置先**: `regions/system/library/dart/dlq_client/`

```
dlq_client/
├── pubspec.yaml        # k1s0_dlq_client, sdk >=3.4.0 <4.0.0
├── analysis_options.yaml
├── lib/
│   ├── dlq_client.dart     # エクスポート
│   └── src/
│       ├── types.dart      # DlqStatus enum, DlqMessage, Request/Response
│       ├── error.dart      # DlqException
│       └── client.dart     # DlqClient クラス
└── test/
    └── dlq_client_test.dart
```

## C# 実装

**配置先**: `regions/system/library/csharp/dlq-client/`

```
dlq-client/
├── src/
│   ├── DlqClient.csproj
│   ├── IDlqClient.cs              # DLQ 管理インターフェース
│   ├── HttpDlqClient.cs           # REST 実装
│   ├── DlqMessage.cs              # DLQ メッセージ
│   ├── DlqStatus.cs               # ステータス列挙型（Pending/Retrying/Resolved/Dead）
│   ├── ListDlqMessagesResponse.cs # ページネーション型
│   └── DlqException.cs            # 公開例外型
├── tests/
│   ├── DlqClient.Tests.csproj
│   ├── Unit/
│   │   └── DlqMessageTests.cs
│   └── Integration/
│       └── HttpDlqClientTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**: `System.Net.Http.Json`（組み込み）

**名前空間**: `K1s0.System.DlqClient`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `IDlqClient` | interface | DLQ メッセージの一覧・取得・再処理・削除 |
| `HttpDlqClient` | class | REST ベースの DLQ クライアント |
| `DlqMessage` | record | DLQ メッセージ詳細情報 |
| `DlqStatus` | enum | `Pending` / `Retrying` / `Resolved` / `Dead` |
| `ListDlqMessagesResponse` | record | ページネーション型（Messages・Total・Page） |
| `DlqException` | class | dlq-client ライブラリの公開例外型 |

**主要 API**:

```csharp
namespace K1s0.System.DlqClient;

public interface IDlqClient
{
    Task<ListDlqMessagesResponse> ListMessagesAsync(
        string topic,
        int page = 1,
        int pageSize = 20,
        CancellationToken cancellationToken = default);

    Task<DlqMessage> GetMessageAsync(
        string messageId,
        CancellationToken cancellationToken = default);

    Task<RetryDlqMessageResponse> RetryMessageAsync(
        string messageId,
        CancellationToken cancellationToken = default);

    Task DeleteMessageAsync(
        string messageId,
        CancellationToken cancellationToken = default);

    Task RetryAllAsync(
        string topic,
        CancellationToken cancellationToken = default);
}

public enum DlqStatus
{
    Pending,
    Retrying,
    Resolved,
    Dead,
}
```

**カバレッジ目標**: 85%以上

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-messaging設計](system-library-messaging設計.md) — k1s0-messaging ライブラリ

---

## Swift

### パッケージ構成
- ターゲット: `K1s0DlqClient`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開API
```swift
// DLQ クライアント（actor で並行安全）
public actor DlqClient {
    public init(baseURL: URL, session: URLSession = .shared)

    /// DLQ メッセージ一覧取得
    public func listMessages(topic: String, limit: Int = 100, offset: Int = 0) async throws -> [DlqMessage]

    /// DLQ メッセージ詳細取得
    public func getMessage(topic: String, id: String) async throws -> DlqMessage

    /// メッセージ再処理
    public func retryMessage(topic: String, id: String) async throws

    /// メッセージ削除
    public func deleteMessage(topic: String, id: String) async throws

    /// 全メッセージ一括再処理
    public func retryAll(topic: String) async throws -> RetryAllResult
}

public struct DlqMessage: Codable, Sendable {
    public let id: String
    public let topic: String
    public let payload: Data
    public let errorReason: String
    public let failedAt: Date
    public let retryCount: Int
}

public struct RetryAllResult: Sendable {
    public let queued: Int
    public let failed: Int
}
```

### エラー型
```swift
public enum DlqError: Error, Sendable {
    case notFound(id: String)
    case httpError(statusCode: Int, body: String)
    case networkError(underlying: Error)
    case decodingFailed(underlying: Error)
}
```

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上
- [system-library-kafka設計](system-library-kafka設計.md) — k1s0-kafka ライブラリ
