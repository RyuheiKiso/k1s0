# k1s0-saga ライブラリ設計

## 概要

Saga パターンクライアントライブラリ。分散トランザクションの開始・状態取得・キャンセルを管理する REST クライアントを提供する。`SagaClient` 構造体、`SagaState`・`SagaStatus` 型、補償トランザクション制御をサポートする。

**配置先**: `regions/system/library/rust/saga/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `SagaClient` | 構造体 | Saga サーバーへの REST クライアント（HTTP、タイムアウト30秒） |
| `SagaStatus` | enum | Saga の実行ステータス（`Started`・`Running`・`Completed`・`Compensating`・`Failed`・`Cancelled`、シリアライズ時 SCREAMING_SNAKE_CASE） |
| `SagaState` | 構造体 | Saga の現在状態（saga_id・workflow_name・current_step・status・payload・correlation_id・initiated_by・error_message・created_at・updated_at） |
| `SagaStepLog` | 構造体 | 各ステップの実行ログ（id・saga_id・step_index・step_name・action・status・request_payload・response_payload・error_message・started_at・completed_at） |
| `StartSagaRequest` | 構造体 | Saga 開始リクエスト（workflow_name・payload・correlation_id・initiated_by） |
| `StartSagaResponse` | 構造体 | Saga 開始レスポンス（saga_id・status） |
| `SagaError` | enum | NetworkError・DeserializeError・ApiError（status_code + message） |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-saga"
version = "0.1.0"
edition = "2021"

[dependencies]
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
async-trait = "0.1"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
wiremock = "0.6"
```

**モジュール構成**:

```
saga/
├── src/
│   ├── lib.rs      # 公開 API（再エクスポート）
│   ├── client.rs   # SagaClient（HTTP REST クライアント）
│   ├── types.rs    # SagaStatus・SagaState・StartSagaRequest/Response・SagaStepLog
│   └── error.rs    # SagaError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_saga::{SagaClient, StartSagaRequest};

let client = SagaClient::new("http://saga-server:8080");

// Saga 開始
let request = StartSagaRequest {
    workflow_name: "order-fulfillment".to_string(),
    payload: serde_json::json!({ "order_id": "ord-123" }),
    correlation_id: Some("corr-001".to_string()),
    initiated_by: Some("order-service".to_string()),
};
let response = client.start_saga(&request).await?;

// 状態取得
let state = client.get_saga(&response.saga_id).await?;
println!("Status: {:?}", state.status);
println!("Current step: {}", state.current_step);

// キャンセル
client.cancel_saga(&response.saga_id).await?;
```

**API エンドポイント**:

| メソッド | パス | 説明 |
|---------|------|------|
| `POST` | `/api/v1/sagas` | Saga 開始 |
| `GET` | `/api/v1/sagas/{id}` | Saga 状態取得 |
| `POST` | `/api/v1/sagas/{id}/cancel` | Saga キャンセル |

## Go 実装

**配置先**: `regions/system/library/go/saga/`

```
saga/
├── client.go       # SagaClient 構造体・HTTP 実装
├── types.go        # SagaStatus・SagaState・Request/Response 型
├── error.go        # SagaError
├── saga_test.go    # ユニットテスト
├── go.mod
└── go.sum
```

**主要型**:

```go
type SagaStatus string

const (
    SagaStatusStarted      SagaStatus = "STARTED"
    SagaStatusRunning      SagaStatus = "RUNNING"
    SagaStatusCompleted    SagaStatus = "COMPLETED"
    SagaStatusCompensating SagaStatus = "COMPENSATING"
    SagaStatusFailed       SagaStatus = "FAILED"
    SagaStatusCancelled    SagaStatus = "CANCELLED"
)

type SagaState struct {
    SagaID       string        `json:"saga_id"`
    WorkflowName string        `json:"workflow_name"`
    Status       SagaStatus    `json:"status"`
    StepLogs     []SagaStepLog `json:"step_logs"`
    CreatedAt    time.Time     `json:"created_at"`
    UpdatedAt    time.Time     `json:"updated_at"`
}

type StartSagaRequest struct {
    WorkflowName string `json:"workflow_name"`
    Payload      any    `json:"payload"`
}

type SagaClient struct {
    endpoint   string
    httpClient *http.Client
}

func NewSagaClient(endpoint string) *SagaClient
func (c *SagaClient) StartSaga(ctx context.Context, req *StartSagaRequest) (*StartSagaResponse, error)
func (c *SagaClient) GetSaga(ctx context.Context, sagaID string) (*SagaState, error)
func (c *SagaClient) CancelSaga(ctx context.Context, sagaID string) error
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/saga/`

```
saga/
├── package.json        # "@k1s0/saga", "type":"module"
├── tsconfig.json       # ES2022, Node16, strict
├── vitest.config.ts    # globals:true
├── src/
│   └── index.ts        # SagaClient, SagaStatus, SagaState, SagaError
└── __tests__/
    └── saga.test.ts
```

**主要 API**:

```typescript
export type SagaStatus =
  | 'STARTED' | 'RUNNING' | 'COMPLETED'
  | 'COMPENSATING' | 'FAILED' | 'CANCELLED';

export interface SagaState {
  sagaId: string;
  workflowName: string;  // saga_type → workflow_name に統一
  status: SagaStatus;
  stepLogs: SagaStepLog[];
  createdAt: string;
  updatedAt: string;
}

export interface StartSagaRequest {
  workflowName: string;  // saga_type → workflow_name に統一
  payload: unknown;
}

export class SagaClient {
  constructor(endpoint: string);
  startSaga(request: StartSagaRequest): Promise<StartSagaResponse>;
  getSaga(sagaId: string): Promise<SagaState>;
  cancelSaga(sagaId: string): Promise<void>;
}
```

## Dart 実装

**配置先**: `regions/system/library/dart/saga/`

```
saga/
├── pubspec.yaml        # k1s0_saga, sdk >=3.4.0 <4.0.0
├── analysis_options.yaml
├── lib/
│   ├── saga.dart       # エクスポート
│   └── src/
│       ├── types.dart  # SagaStatus, SagaState, SagaStepLog
│       ├── client.dart # SagaClient
│       └── error.dart  # SagaError
└── test/
    └── saga_test.dart
```

## C# 実装

**配置先**: `regions/system/library/csharp/saga/`

```
saga/
├── src/
│   ├── Saga.csproj
│   ├── ISagaClient.cs             # Saga クライアントインターフェース
│   ├── HttpSagaClient.cs          # REST 実装
│   ├── GrpcSagaClient.cs          # gRPC 実装
│   ├── SagaState.cs               # Saga 状態
│   ├── SagaStatus.cs              # ステータス列挙型
│   ├── SagaStepLog.cs             # ステップ実行ログ
│   └── SagaException.cs           # 公開例外型
├── tests/
│   ├── Saga.Tests.csproj
│   ├── Unit/
│   │   └── SagaStatusTests.cs
│   └── Integration/
│       └── HttpSagaClientTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| Grpc.Net.Client | gRPC クライアント |
| Google.Protobuf | Protobuf シリアライズ |
| Grpc.Tools | proto コード生成 |

**名前空間**: `K1s0.System.Saga`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `ISagaClient` | interface | Saga の開始・状態取得・キャンセル |
| `HttpSagaClient` | class | REST ベースの Saga クライアント |
| `GrpcSagaClient` | class | gRPC ベースの Saga クライアント |
| `SagaState` | record | Saga の現在状態（SagaId・WorkflowName・Status・StepLogs 等） |
| `SagaStatus` | enum | `Started` / `Running` / `Completed` / `Compensating` / `Failed` / `Cancelled` |
| `SagaStepLog` | record | 各ステップの実行ログ |
| `SagaException` | class | saga ライブラリの公開例外型 |

**主要 API**:

```csharp
namespace K1s0.System.Saga;

public interface ISagaClient
{
    Task<StartSagaResponse> StartSagaAsync(
        StartSagaRequest request,
        CancellationToken cancellationToken = default);

    Task<SagaState> GetSagaAsync(
        string sagaId,
        CancellationToken cancellationToken = default);

    Task CancelSagaAsync(
        string sagaId,
        CancellationToken cancellationToken = default);
}

public enum SagaStatus
{
    Started,
    Running,
    Completed,
    Compensating,
    Failed,
    Cancelled,
}
```

**カバレッジ目標**: 85%以上

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-saga-server設計](system-saga-server設計.md) — saga-server REST API 設計
- [system-library-messaging設計](system-library-messaging設計.md) — k1s0-messaging ライブラリ

---

## Swift

### パッケージ構成
- ターゲット: `K1s0Saga`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開API
```swift
// Saga クライアント（actor で並行安全）
public actor SagaClient {
    public init(baseURL: URL, session: URLSession = .shared)

    /// Saga 開始
    public func startSaga(type: String, payload: Data, correlationId: String? = nil) async throws -> SagaState

    /// Saga 状態取得
    public func getSaga(id: UUID) async throws -> SagaState

    /// Saga キャンセル
    public func cancelSaga(id: UUID, reason: String? = nil) async throws -> SagaState
}

// Saga 状態
public struct SagaState: Codable, Sendable, Identifiable {
    public let id: UUID
    public let type: String
    public let status: SagaStatus
    public let steps: [SagaStep]
    public let createdAt: Date
    public let updatedAt: Date
    public let completedAt: Date?
}

// Saga ステータス
public enum SagaStatus: String, Codable, Sendable {
    case started
    case running
    case completed
    case compensating
    case cancelled
    case failed
}

public struct SagaStep: Codable, Sendable {
    public let name: String
    public let status: SagaStatus
    public let executedAt: Date?
}
```

### エラー型
```swift
public enum SagaError: Error, Sendable {
    case notFound(id: UUID)
    case invalidTransition(from: SagaStatus, to: SagaStatus)
    case httpError(statusCode: Int, body: String)
    case networkError(underlying: Error)
    case decodingFailed(underlying: Error)
}
```

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上
- [system-library-outbox設計](system-library-outbox設計.md) — k1s0-outbox ライブラリ
