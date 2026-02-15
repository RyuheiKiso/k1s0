# API 設計

k1s0 における REST API / gRPC / GraphQL の設計方針と、バージョニング・レート制限・コード自動生成を定義する。
Tier アーキテクチャの詳細は [tier-architecture.md](tier-architecture.md) を参照。

## 基本方針

- サービス間通信は **gRPC** を標準とする
- 外部クライアント向けには **REST API** を Kong API Gateway 経由で公開する
- BFF が必要な場合は **GraphQL** をオプション採用する
- 全 API に統一的なエラーレスポンス・バージョニング・レート制限を適用する

---

## Tier 別 API 種別パターン

各 Tier のサービスが提供する API 種別のパターンを以下に定義する。具体的なエンドポイント一覧は各サービスの設計フェーズで定義する。

### system Tier

基盤サービスとして、認証・設定・共通マスタデータを提供する。

| API 種別 | プロトコル | 用途 | 例 |
| --- | --- | --- | --- |
| 認証 API | REST | OAuth 2.0 フローに基づくトークン発行・検証 | `/api/v1/auth/token`, `/api/v1/auth/refresh` |
| 認証 API | gRPC | サービス間のトークン検証（高速な内部呼び出し） | `AuthService.ValidateToken` |
| config API | REST | 設定値の取得・更新 | `/api/v1/config/{key}` |
| config API | gRPC | サービス間の設定値参照 | `ConfigService.GetConfig` |
| 共通マスタ API | REST | マスタデータの CRUD（外部向け） | `/api/v1/master/{resource}` |
| 共通マスタ API | gRPC | マスタデータ参照（サービス間） | `MasterService.GetMaster` |

### business Tier

ドメイン固有のビジネスロジックを提供する。

| API 種別 | プロトコル | 用途 | 例 |
| --- | --- | --- | --- |
| ドメイン CRUD API | REST | ドメインエンティティの作成・参照・更新・削除 | `/api/v1/ledger/entries`, `/api/v1/accounts` |
| ドメインイベント配信 | gRPC Stream | ドメインイベントのリアルタイム配信（サーバーストリーミング） | `LedgerService.StreamLedgerEvents` |
| ドメイン操作 API | gRPC | サービス間のドメイン操作呼び出し | `AccountingService.CloseLedger` |

### service Tier

フロントエンド向けの BFF やサードパーティ向けの外部連携を提供する。

| API 種別 | プロトコル | 用途 | 例 |
| --- | --- | --- | --- |
| BFF API | REST | フロントエンドからの標準的な API 呼び出し | `/api/v1/orders`, `/api/v1/dashboard` |
| BFF API | GraphQL | 複数サービスのデータ集約（導入基準を満たす場合） | `query { dashboard { ... } }` |
| 外部連携 API | REST | サードパーティシステムとのデータ連携 | `/api/v1/integrations/{provider}` |

> **注記**: 上記はパターンの定義であり、具体的なエンドポイント一覧・リクエスト/レスポンス仕様は各サービスの設計フェーズで定義する。

---

## D-007: REST API エラーレスポンス設計

### 統一 JSON スキーマ

すべての REST API エラーレスポンスは以下の JSON スキーマに従う。

```json
{
  "error": {
    "code": "SVC_ORDER_NOT_FOUND",
    "message": "指定された注文が見つかりません",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

| フィールド            | 型       | 必須 | 説明                                       |
| --------------------- | -------- | ---- | ------------------------------------------ |
| `error.code`          | string   | Yes  | Tier プレフィックス付きエラーコード        |
| `error.message`       | string   | Yes  | 人間が読めるエラーメッセージ               |
| `error.request_id`    | string   | Yes  | リクエスト追跡用の一意な ID               |
| `error.details`       | array    | No   | バリデーションエラー等の詳細情報           |

### Tier プレフィックス付きエラーコード

エラーコードは Tier アーキテクチャの階層に対応するプレフィックスを付与し、エラーの発生源を明確にする。

| プレフィックス | Tier     | 例                        |
| -------------- | -------- | ------------------------- |
| `SYS_`         | system   | `SYS_AUTH_TOKEN_EXPIRED`  |
| `BIZ_`         | business | `BIZ_ACCT_LEDGER_CLOSED`  |
| `SVC_`         | service  | `SVC_ORDER_NOT_FOUND`     |

エラーコードの命名規則: `{TIER}_{DOMAIN}_{ERROR_NAME}`

- `TIER`: `SYS` / `BIZ` / `SVC`
- `DOMAIN`: サービスまたはドメインの省略名（SCREAMING_SNAKE_CASE）
- `ERROR_NAME`: エラーの内容（SCREAMING_SNAKE_CASE）

### HTTP ステータスコードマッピング

| HTTP ステータス | 用途                               | エラーコード例                  |
| --------------- | ---------------------------------- | ------------------------------- |
| 400             | バリデーションエラー               | `SVC_ORDER_INVALID_QUANTITY`    |
| 401             | 認証エラー（トークン無効・期限切れ）| `SYS_AUTH_TOKEN_EXPIRED`        |
| 403             | 認可エラー（権限不足）             | `SYS_AUTH_FORBIDDEN`            |
| 404             | リソースが見つからない             | `SVC_ORDER_NOT_FOUND`           |
| 409             | 競合（楽観ロック等）               | `SVC_ORDER_VERSION_CONFLICT`    |
| 422             | ビジネスルール違反                 | `BIZ_ACCT_LEDGER_CLOSED`       |
| 429             | レート制限超過                     | `SYS_RATE_LIMIT_EXCEEDED`      |
| 500             | 内部サーバーエラー                 | `SYS_INTERNAL_ERROR`           |
| 503             | サービス利用不可                   | `SYS_SERVICE_UNAVAILABLE`      |

### バリデーションエラーの details 配列

バリデーションエラー（400）の場合、`details` 配列にフィールド単位のエラー情報を格納する。

```json
{
  "error": {
    "code": "SVC_ORDER_VALIDATION_FAILED",
    "message": "リクエストのバリデーションに失敗しました",
    "request_id": "req_abc123def456",
    "details": [
      {
        "field": "quantity",
        "reason": "must_be_positive",
        "message": "数量は1以上を指定してください"
      },
      {
        "field": "shipping_address.postal_code",
        "reason": "invalid_format",
        "message": "郵便番号の形式が不正です"
      }
    ]
  }
}
```

### Go 実装例

```go
// internal/adapter/handler/error.go

type APIError struct {
    Code      string          `json:"code"`
    Message   string          `json:"message"`
    RequestID string          `json:"request_id"`
    Details   []ErrorDetail   `json:"details,omitempty"`
}

type ErrorDetail struct {
    Field   string `json:"field"`
    Reason  string `json:"reason"`
    Message string `json:"message"`
}

type ErrorResponse struct {
    Error APIError `json:"error"`
}

func WriteError(w http.ResponseWriter, r *http.Request, status int, code, message string) {
    resp := ErrorResponse{
        Error: APIError{
            Code:      code,
            Message:   message,
            RequestID: middleware.GetRequestID(r.Context()),
        },
    }
    w.Header().Set("Content-Type", "application/json")
    w.WriteHeader(status)
    json.NewEncoder(w).Encode(resp)
}
```

### Rust 実装例

```rust
// src/adapter/handler/error.rs

use serde::Serialize;

#[derive(Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<ErrorDetail>,
}

#[derive(Serialize)]
pub struct ErrorDetail {
    pub field: String,
    pub reason: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: ApiError,
}

impl axum::response::IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        let status = match self.error.code.as_str() {
            c if c.ends_with("NOT_FOUND") => StatusCode::NOT_FOUND,
            c if c.ends_with("VALIDATION_FAILED") => StatusCode::BAD_REQUEST,
            c if c.ends_with("FORBIDDEN") => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, axum::Json(self)).into_response()
    }
}
```

---

## D-008: REST API バージョニング

### URL パス方式

REST API のバージョニングは **URL パス方式** を採用する。

```
https://api.k1s0.internal.example.com/api/v1/orders
https://api.k1s0.internal.example.com/api/v2/orders
```

### バージョニングルール

| 項目               | ルール                                               |
| ------------------ | ---------------------------------------------------- |
| 方式               | URL パスプレフィックス `/api/v{major}/`              |
| メジャーバージョン | 後方互換性を破壊する変更時にインクリメント           |
| 初期バージョン     | `v1`                                                 |
| 非推奨化ポリシー   | 新バージョンリリース後 **6 か月間** 旧バージョンを並行運用 |
| 非推奨ヘッダー     | `Deprecation: true` + `Sunset: <date>` をレスポンスに付与 |

### 後方互換とみなす変更（バージョンアップ不要）

- レスポンスへの新規フィールド追加（オプショナル）
- 新規エンドポイントの追加
- 新規クエリパラメータの追加（オプショナル）

### 後方互換を破壊する変更（メジャーバージョンアップ）

- 既存フィールドの削除・型変更
- 必須パラメータの追加
- エンドポイントの URL 変更
- レスポンス構造の変更

### Kong ルーティング連携

Kong API Gateway で URL パスに基づいてバージョン別のルーティングを行う。

```yaml
# Kong Service / Route 設定
services:
  - name: order-v1
    url: http://order-server.k1s0-service.svc.cluster.local:80
    routes:
      - name: order-v1-route
        paths:
          - /api/v1/orders
        strip_path: false

  - name: order-v2
    url: http://order-server-v2.k1s0-service.svc.cluster.local:80
    routes:
      - name: order-v2-route
        paths:
          - /api/v2/orders
        strip_path: false
```

### 非推奨レスポンスヘッダー

旧バージョンのエンドポイントには Kong プラグインで非推奨ヘッダーを付与する。

```
Deprecation: true
Sunset: Sat, 01 Mar 2026 00:00:00 GMT
Link: <https://api.k1s0.internal.example.com/api/v2/orders>; rel="successor-version"
```

---

## D-009: gRPC サービス定義パターン

### proto パッケージ命名

gRPC の proto ファイルは以下の命名規則に従う。

```
k1s0.{tier}.{domain}.v{major}
```

| 要素       | 説明                       | 例                  |
| ---------- | -------------------------- | ------------------- |
| `k1s0`     | プロジェクトプレフィックス | 固定                |
| `tier`     | Tier 名                    | `system`, `service` |
| `domain`   | ドメイン名                 | `auth`, `order`     |
| `v{major}` | メジャーバージョン         | `v1`, `v2`          |

### proto ファイル配置

proto ファイルは以下の2つのレベルで管理する。

#### プロジェクトルートの共有定義

プロジェクトルート `api/proto/` には、サービス間で共有されるプロトコル定義を配置する。

| パス | 用途 |
| --- | --- |
| `api/proto/k1s0/system/common/v1/` | 共有プロトコル定義（共通の message 型、共通の enum、Pagination 等） |
| `api/proto/k1s0/event/` | イベント定義（メッセージング設計.md 参照） |
| `api/proto/buf.yaml` | buf 設定（lint・breaking change 検出） |

各サービスは共有定義を import して参照する。

#### サービス内の固有定義

各サービス内の `api/proto/` には、そのサービス固有の gRPC サービス定義を配置する。サービス固有の Request / Response メッセージ型やサービス定義はここに格納し、他サービスからは直接参照しない。

#### 配置ルールまとめ

| 対象 | 配置先 | 例 |
| --- | --- | --- |
| 複数サービスで共有する message 型・enum | `api/proto/k1s0/system/common/v1/` | Pagination, Timestamp, Money |
| イベント定義（Kafka メッセージスキーマ） | `api/proto/k1s0/event/{tier}/{domain}/v1/` | OrderCreatedEvent, LoginEvent |
| サービス固有の gRPC サービス定義 | `{サービス}/api/proto/` | OrderService, LedgerService |
| サービス固有の Request/Response 型 | `{サービス}/api/proto/` | CreateOrderRequest |

#### ディレクトリ構成

```
# プロジェクトルート（共有定義）
api/proto/
├── k1s0/
│   ├── event/                     # イベント定義（メッセージング設計.md 参照）
│   │   ├── system/
│   │   ├── business/
│   │   └── service/
│   └── system/
│       └── common/
│           └── v1/
│               ├── types.proto            # Pagination, Timestamp 等の共通型
│               └── event_metadata.proto   # イベントメタデータ
└── buf.yaml                       # buf 設定

# サービス内（固有定義）
{サービス}/api/proto/
├── k1s0/
│   ├── system/
│   │   └── auth/
│   │       └── v1/
│   │           └── auth.proto
│   ├── business/
│   │   └── accounting/
│   │       └── v1/
│   │           └── ledger.proto
│   └── service/
│       └── order/
│           └── v1/
│               └── order.proto
└── buf.yaml
```

### 共通型定義

全 Tier で共有する型を `k1s0.system.common.v1` に定義する。

```protobuf
// k1s0/system/common/v1/types.proto
syntax = "proto3";
package k1s0.system.common.v1;

message Pagination {
  int32 page = 1;
  int32 page_size = 2;
}

message PaginationResult {
  int32 total_count = 1;
  int32 page = 2;
  int32 page_size = 3;
  bool has_next = 4;
}

message Timestamp {
  int64 seconds = 1;
  int32 nanos = 2;
}
```

### サービス定義例

```protobuf
// k1s0/service/order/v1/order.proto
syntax = "proto3";
package k1s0.service.order.v1;

import "k1s0/system/common/v1/types.proto";

service OrderService {
  rpc CreateOrder(CreateOrderRequest) returns (CreateOrderResponse);
  rpc GetOrder(GetOrderRequest) returns (GetOrderResponse);
  rpc ListOrders(ListOrdersRequest) returns (ListOrdersResponse);
}

message CreateOrderRequest {
  string product_id = 1;
  int32 quantity = 2;
}

message CreateOrderResponse {
  string order_id = 1;
}

message GetOrderRequest {
  string order_id = 1;
}

message GetOrderResponse {
  string order_id = 1;
  string product_id = 2;
  int32 quantity = 3;
  string status = 4;
}

message ListOrdersRequest {
  k1s0.system.common.v1.Pagination pagination = 1;
}

message ListOrdersResponse {
  repeated GetOrderResponse orders = 1;
  k1s0.system.common.v1.PaginationResult pagination = 2;
}
```

### gRPC ステータスコードマッピング

| gRPC ステータス       | 用途                               | HTTP 対応 |
| --------------------- | ---------------------------------- | --------- |
| `OK`                  | 成功                               | 200       |
| `INVALID_ARGUMENT`    | バリデーションエラー               | 400       |
| `UNAUTHENTICATED`     | 認証エラー                         | 401       |
| `PERMISSION_DENIED`   | 認可エラー                         | 403       |
| `NOT_FOUND`           | リソースが見つからない             | 404       |
| `ALREADY_EXISTS`      | リソースの重複                     | 409       |
| `FAILED_PRECONDITION` | ビジネスルール違反                 | 422       |
| `RESOURCE_EXHAUSTED`  | レート制限超過                     | 429       |
| `INTERNAL`            | 内部エラー                         | 500       |
| `UNAVAILABLE`         | サービス利用不可                   | 503       |

### Go Interceptor 実装例

```go
// internal/infra/grpc/interceptor.go

func UnaryErrorInterceptor() grpc.UnaryServerInterceptor {
    return func(
        ctx context.Context,
        req interface{},
        info *grpc.UnaryServerInfo,
        handler grpc.UnaryHandler,
    ) (interface{}, error) {
        resp, err := handler(ctx, req)
        if err != nil {
            // ドメインエラーを gRPC ステータスに変換
            if domainErr, ok := err.(*domain.Error); ok {
                st := status.New(mapToGRPCCode(domainErr.Code), domainErr.Message)
                st, _ = st.WithDetails(&errdetails.ErrorInfo{
                    Reason: domainErr.Code,
                    Domain: "k1s0",
                })
                return nil, st.Err()
            }
            return nil, status.Error(codes.Internal, "internal error")
        }
        return resp, nil
    }
}
```

### Rust Interceptor 実装例

```rust
// src/infra/grpc/interceptor.rs

use tonic::{Request, Status};

pub fn auth_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    let token = req.metadata().get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Status::unauthenticated("missing authorization token"))?;

    // JWT 検証ロジック
    validate_token(token)
        .map_err(|e| Status::unauthenticated(format!("invalid token: {}", e)))?;

    Ok(req)
}
```

### Buf による管理

proto ファイルの lint・破壊的変更検出には [Buf](https://buf.build/) を使用する。

```yaml
# buf.yaml
version: v2
modules:
  - path: api/proto
lint:
  use:
    - STANDARD
breaking:
  use:
    - FILE
```

```yaml
# buf.gen.yaml
version: v2
plugins:
  - remote: buf.build/protocolbuffers/go
    out: gen/go
    opt: paths=source_relative
  - remote: buf.build/grpc/go
    out: gen/go
    opt: paths=source_relative
  - remote: buf.build/protocolbuffers/rust
    out: gen/rust
```

---

## D-010: gRPC バージョニング

### パッケージレベルバージョニング

gRPC のバージョニングは **proto パッケージ名にメジャーバージョンを含める** 方式を採用する。

```
k1s0.service.order.v1  →  k1s0.service.order.v2
```

バージョンアップ時は新しいパッケージディレクトリを作成し、旧バージョンと並行運用する。

```
api/proto/k1s0/service/order/
├── v1/
│   └── order.proto     # 旧バージョン（非推奨期間中は維持）
└── v2/
    └── order.proto     # 新バージョン
```

### 後方互換性ルール

proto ファイルの変更時は以下のルールに従う。

#### 後方互換（バージョンアップ不要）

- 新規フィールドの追加（新しいフィールド番号を使用）
- 新規 RPC メソッドの追加
- 新規 enum 値の追加

#### 後方互換性を破壊する変更（メジャーバージョンアップ）

- フィールドの削除・番号変更
- フィールドの型変更
- RPC メソッドのシグネチャ変更
- メッセージ名の変更
- `required` / `optional` セマンティクスの変更

### buf breaking による CI 検出

CI パイプラインで `buf breaking` を実行し、意図しない破壊的変更を検出する。

```yaml
# .github/workflows/ci.yaml（proto 関連の抜粋）
jobs:
  proto-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: bufbuild/buf-setup-action@v1
      - name: Lint
        run: buf lint api/proto
      - name: Breaking change detection
        run: buf breaking api/proto --against '.git#branch=main'
```

破壊的変更が検出された場合は CI が失敗し、意図的な変更であれば新しいバージョンパッケージとして作成する。

---

## D-011: GraphQL 設計

### 採用方針

GraphQL は **BFF（Backend for Frontend）としてオプション採用** する。すべてのサービスに GraphQL を導入するのではなく、複数の REST / gRPC エンドポイントを集約して単一のクエリでクライアントに提供する必要がある場合に採用する。

```
Client → Nginx Ingress Controller → Kong → GraphQL BFF → gRPC → Backend Services
```

### GraphQL BFF 導入基準

#### 導入条件

GraphQL BFF は以下の条件を満たす場合に導入を検討する。

| 条件 | 説明 |
| --- | --- |
| サービス集約数 | **1つの画面で3つ以上のマイクロサービス**からデータを集約する必要がある場合 |
| フィールド差異 | クライアント種別（Web / Mobile）によって必要なフィールドが大きく異なる場合 |
| レスポンス最適化 | モバイル向けにレスポンスサイズの最小化が必要な場合 |

#### 導入フェーズ

- **初期フェーズでは GraphQL BFF を採用しない**。REST API で十分に対応可能な段階では REST を使用する
- フロントエンドの複雑性が増し、上記の導入条件を満たした段階で GraphQL BFF の導入を検討する
- 導入判断はフロントエンドチームとバックエンドチームの合意のもとで行う

#### 導入対象候補

以下のような集約表示が必要な画面が GraphQL BFF の導入対象候補となる。

| 画面 | 集約対象サービス例 | 理由 |
| --- | --- | --- |
| ダッシュボード画面 | 注文、在庫、会計、ユーザーなど | 複数ドメインの集約データを一覧表示する |
| レポート画面 | 売上、在庫推移、顧客分析など | 複数サービスの分析データを統合表示する |
| ユーザープロフィール画面 | ユーザー情報、注文履歴、通知など | ユーザーに紐づく複数サービスのデータを表示する |

### REST vs GraphQL 使い分け基準（D-089）

#### 原則: REST がデフォルト、GraphQL は条件を満たす場合のみ採用

| 条件                                                             | REST | GraphQL |
| ---------------------------------------------------------------- | ---- | ------- |
| 単一リソースの CRUD 操作                                        | o    |         |
| サービス間通信（バックエンド同士）                                | o    |         |
| 1画面で **3つ以上の異なるサービス** のデータを集約表示する       |      | o       |
| クライアントによって必要なフィールドが **大きく異なる**          |      | o       |
| モバイル向けに **レスポンスサイズの最小化** が必要               |      | o       |
| 公開 API（外部パートナー・サードパーティ向け）                   | o    |         |
| ファイルアップロード・ダウンロード                               | o    |         |
| WebSocket によるリアルタイム更新（Subscription）が必要           |      | o       |

#### 判断フロー

```
1. そのエンドポイントは単一サービスのデータだけで完結するか？
   → Yes: REST を使用
   → No: 次へ

2. クライアントが必要とするフィールドは固定的か？
   → Yes: REST で集約エンドポイントを作成
   → No: 次へ

3. 複数サービスのデータを1リクエストで取得する必要があるか？
   → Yes: GraphQL BFF を採用
   → No: REST を使用
```

#### GraphQL を採用してはならないケース

- **サービス間通信**: バックエンド間は gRPC を使用する。GraphQL はクライアント向け BFF 専用
- **単純な CRUD API**: REST で十分な場合に GraphQL を採用すると、不要な複雑性が増す
- **認証エンドポイント**: OAuth 2.0 の標準フローに従い REST で実装する

### クエリ制限

GraphQL の柔軟性に起因するパフォーマンスリスクを制御するため、以下の制限を設ける。

| 項目           | 制限値 | 説明                                   |
| -------------- | ------ | -------------------------------------- |
| クエリ深度上限 | 10     | ネストの最大深度                       |
| 複雑度上限     | 1000   | クエリの複雑度スコアの上限             |
| タイムアウト   | 30s    | クエリ実行のタイムアウト               |

### ページネーション

Relay スタイルの Cursor ベースページネーションを標準とする。

```graphql
type Query {
  orders(first: Int, after: String, last: Int, before: String): OrderConnection!
}

type OrderConnection {
  edges: [OrderEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type OrderEdge {
  node: Order!
  cursor: String!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}
```

### スキーマ進化によるバージョニング不要方針

GraphQL ではスキーマの進化的な変更（Evolutionary Schema Design）により、明示的なバージョニングを行わない。

| 変更種別           | 方法                                     |
| ------------------ | ---------------------------------------- |
| フィールド追加     | そのまま追加（既存クエリに影響なし）     |
| フィールド非推奨化 | `@deprecated(reason: "...")` を付与      |
| フィールド削除     | 非推奨化から 6 か月後に削除              |
| 型の追加           | そのまま追加                             |

```graphql
type Order {
  id: ID!
  status: OrderStatus!
  totalAmount: Float! @deprecated(reason: "Use totalPrice instead")
  totalPrice: Money!
}
```

### スキーマ設計例

```graphql
# schema.graphql

type Query {
  order(id: ID!): Order
  orders(first: Int, after: String): OrderConnection!
  me: User!
}

type Mutation {
  createOrder(input: CreateOrderInput!): CreateOrderPayload!
}

type Order {
  id: ID!
  product: Product!
  quantity: Int!
  status: OrderStatus!
  totalPrice: Money!
  createdAt: DateTime!
}

type Money {
  amount: Float!
  currency: String!
}

enum OrderStatus {
  PENDING
  CONFIRMED
  SHIPPED
  DELIVERED
  CANCELLED
}

input CreateOrderInput {
  productId: ID!
  quantity: Int!
}

type CreateOrderPayload {
  order: Order
  errors: [UserError!]!
}

type UserError {
  field: [String!]
  message: String!
}
```

---

## D-012: レート制限設計

### Kong 一元管理

レート制限は **Kong API Gateway の Rate Limiting プラグイン** で一元管理する。個別サービスでのレート制限実装は行わない。

### Tier 別デフォルト値

| Tier     | デフォルト制限 | 説明                               |
| -------- | -------------- | ---------------------------------- |
| system   | 3000 req/min   | 内部基盤サービス（高頻度呼び出し） |
| business | 1000 req/min   | 領域共通サービス                   |
| service  | 500 req/min    | 個別業務サービス                   |

### Kong プラグイン設定

```yaml
# Kong Rate Limiting プラグイン（グローバル設定）
plugins:
  - name: rate-limiting
    config:
      minute: 500                    # デフォルト（service Tier）
      policy: redis                  # Redis で共有状態を管理
      redis_host: redis.k1s0-system.svc.cluster.local
      redis_port: 6379
      redis_database: 1
      fault_tolerant: true           # Redis 障害時は制限なしで通過
      hide_client_headers: false     # X-RateLimit-* ヘッダーを返却
```

### エンドポイント別オーバーライド

特定のエンドポイントに対してデフォルト値を上書きする。

```yaml
# 例: 認証ログインエンドポイントは低めに設定（ブルートフォース防止）
# auth-v1 サービスの特定ルートにのみ適用（APIゲートウェイ設計.md のサービス名と統一）
services:
  - name: auth-v1
    routes:
      - name: auth-v1-login
        paths:
          - /api/v1/auth/login
    plugins:
      - name: rate-limiting
        config:
          minute: 30
          policy: redis

# 例: ヘルスチェックは高めに設定
  - name: health-check
    routes:
      - name: health-route
        paths:
          - /healthz
    plugins:
      - name: rate-limiting
        config:
          minute: 6000
          policy: redis
```

### Redis 共有状態

レート制限のカウンターは Redis で共有し、Kong の複数インスタンス間で一貫性を保つ。

| 設定項目         | 値                                            |
| ---------------- | --------------------------------------------- |
| Redis ホスト     | `redis.k1s0-system.svc.cluster.local`         |
| Redis DB         | 1（レート制限専用）                           |
| TTL              | Window サイズと同一（自動管理）               |
| フォールトトレラント | `true`（Redis 障害時は制限を一時停止）    |

### バースト制御

短時間のスパイクを許容するため、バースト制御を設定する。

```yaml
plugins:
  - name: rate-limiting
    config:
      minute: 500
      second: 20                    # 秒あたりの上限（バースト制御）
      policy: redis
```

| Tier     | 分あたり制限 | 秒あたり制限（バースト） |
| -------- | ------------ | ------------------------ |
| system   | 3000         | 100                      |
| business | 1000         | 50                       |
| service  | 500          | 20                       |

### 環境別倍率

開発環境ではテスト容易性のため制限を緩和する。

| 環境    | 倍率 | system     | business   | service    |
| ------- | ---- | ---------- | ---------- | ---------- |
| dev     | x10  | 30000/min  | 10000/min  | 5000/min   |
| staging | x2   | 6000/min   | 2000/min   | 1000/min   |
| prod    | x1   | 3000/min   | 1000/min   | 500/min    |

### レート制限レスポンスヘッダー

レート制限情報はレスポンスヘッダーで返却する。

```
X-RateLimit-Limit: 500
X-RateLimit-Remaining: 423
X-RateLimit-Reset: 1710000000
```

レート制限超過時は HTTP 429 を返却する。

```json
{
  "error": {
    "code": "SYS_RATE_LIMIT_EXCEEDED",
    "message": "レート制限を超過しました。しばらく待ってから再試行してください",
    "request_id": "req_xyz789",
    "details": [
      {
        "field": "rate_limit",
        "reason": "exceeded",
        "message": "500 requests per minute exceeded"
      }
    ]
  }
}
```

---

## D-123: OpenAPI コード自動生成

### 基本方針

OpenAPI 定義からサーバー・クライアントコードを自動生成し、API 定義と実装の一貫性を保証する。

### ツール選定

| 言語 / 用途          | ツール              | 方式                  |
| -------------------- | ------------------- | --------------------- |
| Go サーバー          | oapi-codegen        | OpenAPI → Go コード   |
| Rust サーバー        | utoipa              | Rust コード → OpenAPI |
| クライアント SDK     | openapi-generator   | OpenAPI → 各言語 SDK  |

### Go: oapi-codegen

OpenAPI 定義から Go のインターフェースとモデルを生成する。

```yaml
# oapi-codegen 設定ファイル
# api/openapi/gen.yaml
package: api
output: internal/adapter/handler/api_gen.go
generate:
  models: true
  chi-server: true
  strict-server: true
```

```bash
# 生成コマンド
oapi-codegen -config api/openapi/gen.yaml api/openapi/openapi.yaml
```

#### 生成先ディレクトリ

```
{サービス名}/
├── api/
│   └── openapi/
│       ├── openapi.yaml          # OpenAPI 定義（手動管理）
│       └── gen.yaml              # oapi-codegen 設定
├── internal/
│   └── adapter/
│       └── handler/
│           ├── api_gen.go        # 生成コード（git 管理）
│           └── handler.go        # 手動実装（インターフェース実装）
```

#### 生成コードの使用例

```go
// internal/adapter/handler/handler.go
package handler

// oapi-codegen が生成した StrictServerInterface を実装
type OrderHandler struct {
    usecase *usecase.OrderUsecase
}

// 生成インターフェースの実装
func (h *OrderHandler) CreateOrder(
    ctx context.Context,
    request api.CreateOrderRequestObject,
) (api.CreateOrderResponseObject, error) {
    order, err := h.usecase.Create(ctx, request.Body)
    if err != nil {
        return nil, err
    }
    return api.CreateOrder201JSONResponse(*order), nil
}
```

### Rust: utoipa

Rust ではコードファースト方式を採用し、utoipa マクロから OpenAPI ドキュメントを生成する。

```rust
// src/adapter/handler/order.rs
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateOrderInput {
    /// 商品 ID
    pub product_id: String,
    /// 注文数量
    #[schema(minimum = 1)]
    pub quantity: i32,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Order {
    pub id: String,
    pub product_id: String,
    pub quantity: i32,
    pub status: OrderStatus,
}

#[utoipa::path(
    post,
    path = "/api/v1/orders",
    request_body = CreateOrderInput,
    responses(
        (status = 201, description = "Order created", body = Order),
        (status = 400, description = "Validation error", body = ErrorResponse),
    ),
    tag = "orders"
)]
async fn create_order(
    State(state): State<AppState>,
    Json(input): Json<CreateOrderInput>,
) -> Result<Json<Order>, AppError> {
    // ...
}
```

```rust
// src/main.rs
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(handler::create_order, handler::get_order, handler::list_orders),
    components(schemas(CreateOrderInput, Order, OrderStatus, ErrorResponse))
)]
struct ApiDoc;

// /openapi.json エンドポイントで OpenAPI ドキュメントを提供
```

### クライアント SDK: openapi-generator

OpenAPI 定義から各言語のクライアント SDK を生成する。

```bash
# TypeScript クライアント生成
openapi-generator-cli generate \
  -i api/openapi/openapi.yaml \
  -g typescript-axios \
  -o gen/ts-client \
  --additional-properties=supportsES6=true

# Dart クライアント生成
openapi-generator-cli generate \
  -i api/openapi/openapi.yaml \
  -g dart \
  -o gen/dart-client
```

#### 生成先ディレクトリ

```
{サービス名}/
├── api/
│   └── openapi/
│       └── openapi.yaml
└── gen/
    ├── ts-client/           # TypeScript SDK（React 用）
    └── dart-client/         # Dart SDK（Flutter 用）
```

#### クライアント SDK 配布方式

| 言語 | 生成先 | 配布方式 | 詳細 |
| --- | --- | --- | --- |
| TypeScript | `gen/ts-client/` | npm private registry | Harbor 内の npm レジストリで配布。`@k1s0/` スコープで公開する |
| Dart | `gen/dart-client/` | Git submodule / パス参照 | Flutter プロジェクトから Git submodule または `pubspec.yaml` のパス参照で依存する |

##### TypeScript SDK の配布設定

```json
// gen/ts-client/package.json
{
  "name": "@k1s0/order-client",
  "version": "1.0.0",
  "publishConfig": {
    "registry": "https://harbor.internal.example.com/npm/"
  }
}
```

##### Dart SDK の依存設定

```yaml
# Flutter プロジェクトの pubspec.yaml
dependencies:
  order_client:
    path: ../../gen/dart-client    # パス参照の場合
    # または Git submodule 経由
    # git:
    #   url: https://git.internal.example.com/k1s0/order-client-dart.git
    #   ref: v1.0.0
```

#### SDK 自動再生成（CI/CD 連携）

OpenAPI 定義または proto 定義の変更時に、CI/CD パイプラインでクライアント SDK を自動再生成する。

```yaml
# .github/workflows/ci.yaml（SDK 再生成の抜粋）
jobs:
  sdk-generate:
    runs-on: ubuntu-latest
    if: contains(github.event.pull_request.changed_files, 'api/openapi/') || contains(github.event.pull_request.changed_files, 'api/proto/')
    steps:
      - uses: actions/checkout@v4
      - name: Generate TypeScript SDK
        run: |
          openapi-generator-cli generate \
            -i api/openapi/openapi.yaml \
            -g typescript-axios \
            -o gen/ts-client \
            --additional-properties=supportsES6=true
      - name: Generate Dart SDK
        run: |
          openapi-generator-cli generate \
            -i api/openapi/openapi.yaml \
            -g dart \
            -o gen/dart-client
      - name: Publish TypeScript SDK
        run: |
          cd gen/ts-client
          npm publish --registry https://harbor.internal.example.com/npm/
      - name: Verify no diff
        run: git diff --exit-code gen/
```

### CI 連携

```yaml
# .github/workflows/ci.yaml（OpenAPI 関連の抜粋）
jobs:
  openapi-validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Validate OpenAPI
        run: |
          npx @redocly/cli lint api/openapi/openapi.yaml

  openapi-codegen:
    needs: openapi-validate
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Generate Go server code
        run: |
          go install github.com/oapi-codegen/oapi-codegen/v2/cmd/oapi-codegen@latest
          oapi-codegen -config api/openapi/gen.yaml api/openapi/openapi.yaml
      - name: Verify no diff
        run: git diff --exit-code
```

---

## D-124: GraphQL 実装技術選定

### 技術選定

GraphQL BFF の実装には Go と Rust の両方に対応する。

| 言語 | ライブラリ      | 方式             | 特徴                         |
| ---- | --------------- | ---------------- | ---------------------------- |
| Go   | gqlgen          | コード生成ベース | スキーマファースト、型安全   |
| Rust | async-graphql   | マクロベース     | 高パフォーマンス、型安全     |

### Go: gqlgen（コード生成ベース）

スキーマファースト開発で、GraphQL スキーマから Go のリゾルバーインターフェースを生成する。

#### gqlgen 設定

```yaml
# gqlgen.yml
schema:
  - api/graphql/*.graphql
exec:
  filename: internal/adapter/graphql/generated.go
  package: graphql
model:
  filename: internal/adapter/graphql/models_gen.go
  package: graphql
resolver:
  layout: follow-schema
  dir: internal/adapter/graphql
  package: graphql
```

#### リゾルバー実装例

```go
// internal/adapter/graphql/resolver.go
package graphql

type Resolver struct {
    orderClient  pb.OrderServiceClient    // gRPC クライアント
    authClient   pb.AuthServiceClient
}

// internal/adapter/graphql/order.resolvers.go（生成テンプレートから手動実装）
func (r *queryResolver) Order(ctx context.Context, id string) (*model.Order, error) {
    resp, err := r.orderClient.GetOrder(ctx, &pb.GetOrderRequest{OrderId: id})
    if err != nil {
        return nil, err
    }
    return toGraphQLOrder(resp), nil
}

func (r *queryResolver) Orders(ctx context.Context, first *int, after *string) (*model.OrderConnection, error) {
    // Relay Cursor ベースページネーションの実装
    resp, err := r.orderClient.ListOrders(ctx, &pb.ListOrdersRequest{
        Pagination: &pb.Pagination{
            PageSize: int32(derefOr(first, 20)),
        },
    })
    if err != nil {
        return nil, err
    }
    return toOrderConnection(resp), nil
}
```

### Rust: async-graphql（マクロベース）

Rust マクロでスキーマとリゾルバーを同時に定義する。

```rust
// src/adapter/graphql/schema.rs
use async_graphql::*;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn order(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Order>> {
        let client = ctx.data::<OrderServiceClient>()?;
        let resp = client
            .get_order(GetOrderRequest {
                order_id: id.to_string(),
            })
            .await?;
        Ok(Some(resp.into()))
    }

    async fn orders(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<String>,
    ) -> Result<OrderConnection> {
        let client = ctx.data::<OrderServiceClient>()?;
        let resp = client
            .list_orders(ListOrdersRequest {
                pagination: Some(Pagination {
                    page_size: first.unwrap_or(20),
                    ..Default::default()
                }),
            })
            .await?;
        Ok(resp.into())
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_order(
        &self,
        ctx: &Context<'_>,
        input: CreateOrderInput,
    ) -> Result<CreateOrderPayload> {
        let client = ctx.data::<OrderServiceClient>()?;
        let resp = client
            .create_order(input.into())
            .await?;
        Ok(CreateOrderPayload {
            order: Some(resp.into()),
            errors: vec![],
        })
    }
}

#[derive(SimpleObject)]
pub struct Order {
    pub id: ID,
    pub product_id: String,
    pub quantity: i32,
    pub status: OrderStatus,
    pub total_price: Money,
}
```

### BFF ディレクトリ構成

GraphQL BFF サーバーは regions 内に配置する。

```
regions/service/{サービス名}/
└── server/
    ├── go/
    │   └── bff/                        # Go BFF
    │       ├── cmd/
    │       │   └── main.go
    │       ├── internal/
    │       │   ├── adapter/
    │       │   │   └── graphql/
    │       │   │       ├── generated.go     # gqlgen 生成コード
    │       │   │       ├── models_gen.go    # gqlgen 生成モデル
    │       │   │       ├── resolver.go      # リゾルバー（手動実装）
    │       │   │       └── order.resolvers.go
    │       │   └── infra/
    │       │       └── grpc/               # バックエンド gRPC クライアント
    │       ├── api/
    │       │   └── graphql/
    │       │       └── schema.graphql      # スキーマ定義
    │       ├── gqlgen.yml
    │       └── go.mod
    └── rust/
        └── bff/                        # Rust BFF
            ├── src/
            │   ├── main.rs
            │   ├── adapter/
            │   │   └── graphql/
            │   │       ├── schema.rs       # スキーマ + リゾルバー
            │   │       └── types.rs        # GraphQL 型定義
            │   └── infra/
            │       └── grpc/               # バックエンド gRPC クライアント
            ├── api/
            │   └── graphql/
            │       └── schema.graphql      # スキーマ定義（参照用）
            └── Cargo.toml
```

### スキーマファースト開発フロー

```
1. schema.graphql を定義・更新
     ↓
2. Go: gqlgen generate でリゾルバーインターフェース生成
   Rust: async-graphql マクロで型を定義
     ↓
3. リゾルバー実装（gRPC バックエンドを呼び出し）
     ↓
4. CI でスキーマバリデーション + テスト
     ↓
5. GraphQL Playground で動作確認（dev 環境のみ有効）
```

---

## 関連ドキュメント

- [tier-architecture.md](tier-architecture.md) — Tier アーキテクチャの詳細
- [config設計.md](config設計.md) — config.yaml スキーマと環境別管理
- [kubernetes設計.md](kubernetes設計.md) — Namespace・NetworkPolicy 設計
- [helm設計.md](helm設計.md) — Helm Chart と values 設計
- [認証認可設計.md](認証認可設計.md) — 認証・認可・Kong 認証フロー
- [インフラ設計.md](インフラ設計.md) — オンプレミスインフラ全体構成
- [APIゲートウェイ設計.md](APIゲートウェイ設計.md) — Kong 構成管理
- [メッセージング設計.md](メッセージング設計.md) — Kafka・イベント駆動設計
- [CI-CD設計.md](CI-CD設計.md) — CI/CD パイプライン設計
