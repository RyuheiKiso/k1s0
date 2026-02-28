# REST API 設計 ガイド

> **仕様**: エラースキーマ・ステータスコード・レート制限テーブルは [REST-API設計.md](./REST-API設計.md) を参照。

## D-007: エラーレスポンス実装例

### Go 実装

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

### Rust 実装

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

## D-008: バージョニング設計の背景

### URL パス方式の選定理由

REST API のバージョニングには主に 3 つの方式がある。

| 方式 | 例 | メリット | デメリット |
| --- | --- | --- | --- |
| URL パス | `/api/v1/orders` | 明確・キャッシュ容易・ルーティング単純 | URL が冗長 |
| ヘッダー | `Accept: application/vnd.k1s0.v1+json` | URL がクリーン | ブラウザテスト困難・キャッシュ考慮 |
| クエリパラメータ | `/orders?version=1` | 実装容易 | RESTful でない・省略時の挙動 |

k1s0 では **URL パス方式** を採用する。理由は以下の通り。

- **Kong との親和性**: Kong のルーティングはパスベースが基本であり、ヘッダーベースのルーティングは設定が複雑になる
- **開発者体験**: URL を見るだけでバージョンが分かるため、デバッグやログ分析が容易
- **キャッシュ戦略**: CDN やプロキシのキャッシュキーに自然に含まれる

### Kong ルーティング連携の設計意図

バージョン別のバックエンドサービスへのルーティングは Kong の Service / Route で制御する。`strip_path: false` を指定することで、バックエンドサービスにもバージョン付きパスがそのまま転送され、サービス側でもバージョン判定が可能になる。

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

---

## D-012: レート制限の設計背景

### Kong 一元管理の選定理由

レート制限の実装方式として、個別サービスでの実装と API Gateway での一元管理の 2 つのアプローチがある。

| 方式 | メリット | デメリット |
| --- | --- | --- |
| 個別サービス実装 | きめ細かい制御 | 実装重複・言語ごとの差異 |
| API Gateway 一元管理 | 一貫性・運用効率 | サービス固有の細かい制御が難しい |

k1s0 では **Kong API Gateway の Rate Limiting プラグイン** で一元管理する。Go / Rust の各サービスに個別のレート制限実装を持たせると、ライブラリのメンテナンスコストが増大し、挙動の一貫性が保証しにくい。

### Redis 共有状態の設計意図

Kong の複数インスタンス間でレートリミットカウンターを共有するため、Redis を使用する。`policy: local`（ローカルメモリ）ではインスタンスごとに独立したカウンターとなり、負荷分散環境では制限値が実質的に N 倍になってしまう問題がある。

`fault_tolerant: true` を設定し、Redis 障害時はレート制限を一時停止する。可用性を優先する設計判断であり、Redis 障害によるサービス全体のダウンを防止する。

### Tier 別制限値の根拠

| Tier | 制限値 | 根拠 |
| --- | --- | --- |
| system (3000/min) | 内部基盤サービスは他サービスから高頻度で呼び出されるため、高めに設定 |
| business (1000/min) | 領域共通サービスは中程度の呼び出し頻度を想定 |
| service (500/min) | 個別業務サービスはエンドユーザー由来のリクエストが主のため、低めに設定 |

### バースト制御の必要性

分あたりの制限だけでは、瞬間的なスパイク（例: 500リクエストが1秒間に集中）を防げない。秒あたりの制限を併設することで、バックエンドサービスの瞬間負荷を抑制する。

### 環境別倍率の設計意図

開発環境ではテスト容易性のため制限を緩和する。E2E テストや負荷テストでレート制限に引っかかることなく、テストを円滑に実行できるようにするための措置である。

---

## D-123: OpenAPI コード自動生成の設計背景

### ツール選定の理由

| 言語 / 用途 | ツール | 方式 | 選定理由 |
| --- | --- | --- | --- |
| Rust サーバー | utoipa | コード → OpenAPI | Rust のマクロシステムと自然に統合でき、コードと仕様の乖離を防げる |
| クライアント SDK | openapi-generator | OpenAPI → SDK | 多言語対応（TypeScript / Dart）で実績がある |

### Rust: utoipa によるコードファースト方式

Rust ではスキーマファースト（YAML → コード生成）ではなく、コードファースト（Rust コード → OpenAPI 生成）を採用する。理由は以下の通り。

- **型安全性**: Rust の型システムがそのまま OpenAPI スキーマに反映される
- **DRY**: バリデーション制約（`minimum`, `maximum` 等）をコード上のアノテーションで一箇所管理
- **ドキュメント自動同期**: `///` doc コメントが OpenAPI の `description` に自動変換される

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

### クライアント SDK 生成と配布

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

---

## 関連ドキュメント

- [REST-API設計.md](./REST-API設計.md) -- エラースキーマ・ステータスコード・レート制限テーブル（仕様）
- [API設計.md](./API設計.md) -- 基本方針・Tier 別 API 種別パターン
- [gRPC設計.md](gRPC設計.md) -- gRPC サービス定義・バージョニング
- [GraphQL設計.md](GraphQL設計.md) -- GraphQL 設計・実装技術選定
- [認証認可設計.md](../auth/認証認可設計.md) -- 認証・認可設計
- [APIゲートウェイ設計.md](./APIゲートウェイ設計.md) -- Kong 構成管理
