# k1s0-saga ライブラリ設計

## 概要

Saga パターンクライアントライブラリ。分散トランザクションの開始・状態取得・キャンセルを管理する REST クライアントを提供する。`SagaClient` 構造体、`SagaState`・`SagaStatus` 型、補償トランザクション制御をサポートする。

**配置先**: `regions/system/library/rust/saga/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `SagaClient` | 構造体 | Saga サーバーへの REST クライアント |
| `SagaStatus` | enum | Saga の実行ステータス（`STARTED`・`RUNNING`・`COMPLETED`・`COMPENSATING`・`FAILED`・`CANCELLED`） |
| `SagaState` | 構造体 | Saga の現在状態（ID・ステータス・ステップログ） |
| `SagaStepLog` | 構造体 | 各ステップの実行ログ |
| `StartSagaRequest` | 構造体 | Saga 開始リクエスト（Saga タイプ・ペイロード） |
| `StartSagaResponse` | 構造体 | Saga 開始レスポンス（Saga ID） |
| `SagaError` | enum | ネットワーク・デシリアライズ・API エラー型 |

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
tokio = { version = "1", features = ["rt-multi-thread"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
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
};
let response = client.start_saga(&request).await?;

// 状態取得
let state = client.get_saga(&response.saga_id).await?;
println!("Status: {:?}", state.status);

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

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-saga-server設計](system-saga-server設計.md) — saga-server REST API 設計
- [system-library-messaging設計](system-library-messaging設計.md) — k1s0-messaging ライブラリ
- [system-library-outbox設計](system-library-outbox設計.md) — k1s0-outbox ライブラリ
