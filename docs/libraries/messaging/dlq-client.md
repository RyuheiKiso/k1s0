# k1s0-dlq-client ライブラリ設計

## 概要

Kafka のデッドレタートピック（`*.dlq`）に送られた処理失敗メッセージを管理する REST クライアント SDK。
DLQ メッセージの一覧取得・詳細取得・再処理・削除・一括再処理を提供する。

**配置先**:
- `regions/system/library/go/dlq-client/`
- `regions/system/library/rust/dlq-client/`
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
| `DlqError`（Dart: `DlqException`） | 構造体/クラス | DLQ クライアントエラー型 |

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
  updated_at: datetime (nullable)
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

**配置先**: `regions/system/library/go/dlq-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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

**配置先**: `regions/system/library/rust/dlq-client/`

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

**配置先**: `regions/system/library/typescript/dlq-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

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
  updatedAt: string | null;
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

**配置先**: `regions/system/library/dart/dlq_client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```dart
enum DlqStatus { pending, retrying, resolved, dead }

class DlqMessage {
  final String id;
  final String originalTopic;
  final String errorMessage;
  final int retryCount;
  final int maxRetries;
  final dynamic payload;
  final DlqStatus status;
  final String createdAt;
  final String? updatedAt;
  final String? lastRetryAt;

  factory DlqMessage.fromJson(Map<String, dynamic> json);
}

class ListDlqMessagesResponse {
  final List<DlqMessage> messages;
  final int total;
  final int page;

  factory ListDlqMessagesResponse.fromJson(Map<String, dynamic> json);
}

class RetryDlqMessageResponse {
  final String messageId;
  final DlqStatus status;

  factory RetryDlqMessageResponse.fromJson(Map<String, dynamic> json);
}

class DlqException implements Exception {
  final String message;
  final int? statusCode;
}

class DlqClient {
  DlqClient(String endpoint, {http.Client? httpClient});
  Future<ListDlqMessagesResponse> listMessages(String topic, int page, int pageSize);
  Future<DlqMessage> getMessage(String messageId);
  Future<RetryDlqMessageResponse> retryMessage(String messageId);
  Future<void> deleteMessage(String messageId);
  Future<void> retryAll(String topic);
}
```

## Proto との整合性ノート

Proto 定義 (`api/proto/k1s0/system/dlq/v1/dlq.proto`) との差異:
- Proto の `payload_json` (string) は REST 実装では `payload` (JSON object) として扱う
- Proto の `RetryMessageResponse` は `DlqMessage` 全体を返すが、REST クライアントは `message_id` + `status` のみ
- Proto の `RetryAllResponse` に `retried_count` フィールドがあるが、REST クライアントでは使用しない
- Proto の `ListMessagesResponse` には `page` フィールドがないが、REST レスポンスには `page` を含める（ページネーション慣例）
- Proto の `DeleteMessageResponse` は削除された `id` を返すが、REST クライアントは void を返す

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-messaging設計](messaging.md) — k1s0-messaging ライブラリ

---
