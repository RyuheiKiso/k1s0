# REST API 設計

D-007 エラーレスポンス、D-008 バージョニング、D-012 レート制限、D-123 OpenAPI コード自動生成の仕様。

元ドキュメント: [API設計.md](./API設計.md)

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
| 429             | レート制限超過                     | `SYS_RATE_EXCEEDED`      |
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

### エラーレスポンス実装例

#### Go 実装

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

#### Rust 実装

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

```
/api/v{major}/...
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

### Kong ルーティング連携

Kong の Service / Route でバージョン別ルーティング。バージョン別のバックエンドサービスへのルーティングは Kong の Service / Route で制御する。`strip_path: false` を指定することで、バックエンドサービスにもバージョン付きパスがそのまま転送され、サービス側でもバージョン判定が可能になる。

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

旧バージョンのエンドポイントには非推奨ヘッダーを付与する。

```
Deprecation: true
Sunset: Sat, 01 Mar 2026 00:00:00 GMT
Link: <https://api.k1s0.internal.example.com/api/v2/orders>; rel="successor-version"
```

---

## D-012: レート制限設計

### Kong 一元管理

Kong Rate Limiting プラグインで一元管理。個別サービス実装は不要。

#### Kong 一元管理の選定理由

レート制限の実装方式として、個別サービスでの実装と API Gateway での一元管理の 2 つのアプローチがある。

| 方式 | メリット | デメリット |
| --- | --- | --- |
| 個別サービス実装 | きめ細かい制御 | 実装重複・言語ごとの差異 |
| API Gateway 一元管理 | 一貫性・運用効率 | サービス固有の細かい制御が難しい |

k1s0 では **Kong API Gateway の Rate Limiting プラグイン** で一元管理する。Go / Rust の各サービスに個別のレート制限実装を持たせると、ライブラリのメンテナンスコストが増大し、挙動の一貫性が保証しにくい。

### Tier 別デフォルト値

| Tier     | デフォルト制限 | 説明                               |
| -------- | -------------- | ---------------------------------- |
| system   | 3000 req/min   | 内部基盤サービス（高頻度呼び出し） |
| business | 1000 req/min   | 領域共通サービス                   |
| service  | 500 req/min    | 個別業務サービス                   |

#### Tier 別制限値の根拠

| Tier | 制限値 | 根拠 |
| --- | --- | --- |
| system (3000/min) | 内部基盤サービスは他サービスから高頻度で呼び出されるため、高めに設定 |
| business (1000/min) | 領域共通サービスは中程度の呼び出し頻度を想定 |
| service (500/min) | 個別業務サービスはエンドユーザー由来のリクエストが主のため、低めに設定 |

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

| 設定項目         | 値                                            |
| ---------------- | --------------------------------------------- |
| Redis ホスト     | `redis.k1s0-system.svc.cluster.local`         |
| Redis DB         | 1（レート制限専用）                           |
| TTL              | Window サイズと同一（自動管理）               |
| フォールトトレラント | `true`（Redis 障害時は制限を一時停止）    |

#### Redis 共有状態の設計意図

Kong の複数インスタンス間でレートリミットカウンターを共有するため、Redis を使用する。`policy: local`（ローカルメモリ）ではインスタンスごとに独立したカウンターとなり、負荷分散環境では制限値が実質的に N 倍になってしまう問題がある。

`fault_tolerant: true` を設定し、Redis 障害時はレート制限を一時停止する。可用性を優先する設計判断であり、Redis 障害によるサービス全体のダウンを防止する。

### バースト制御

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
| business | 1000         | 40                       |
| service  | 500          | 20                       |

#### バースト制御の必要性

分あたりの制限だけでは、瞬間的なスパイク（例: 500リクエストが1秒間に集中）を防げない。秒あたりの制限を併設することで、バックエンドサービスの瞬間負荷を抑制する。

### 環境別倍率

| 環境    | 倍率 | system     | business   | service    |
| ------- | ---- | ---------- | ---------- | ---------- |
| dev     | x10  | 30000/min  | 10000/min  | 5000/min   |
| staging | x2   | 6000/min   | 2000/min   | 1000/min   |
| prod    | x1   | 3000/min   | 1000/min   | 500/min    |

#### 環境別倍率の設計意図

開発環境ではテスト容易性のため制限を緩和する。E2E テストや負荷テストでレート制限に引っかかることなく、テストを円滑に実行できるようにするための措置である。

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
    "code": "SYS_RATE_EXCEEDED",
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

### ツール選定

| 言語 / 用途          | ツール              | 方式                  |
| -------------------- | ------------------- | --------------------- |
| Rust サーバー        | utoipa              | Rust コード → OpenAPI |
| クライアント SDK     | openapi-generator   | OpenAPI → 各言語 SDK  |

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
```

---

## 関連ドキュメント

- [API設計.md](./API設計.md) -- 基本方針・Tier 別 API 種別パターン
- [gRPC設計.md](gRPC設計.md) -- gRPC サービス定義・バージョニング
- [GraphQL設計.md](GraphQL設計.md) -- GraphQL 設計・実装技術選定
- [認証認可設計.md](../auth/認証認可設計.md) -- 認証・認可設計
- [APIゲートウェイ設計.md](./APIゲートウェイ設計.md) -- Kong 構成管理
