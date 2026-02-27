# gRPC 設計

D-009 サービス定義パターン、D-010 バージョニングを定義する。

元ドキュメント: [API設計.md](../gateway/API設計.md)

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

## 関連ドキュメント

- [API設計.md](../gateway/API設計.md) -- 基本方針・Tier 別 API 種別パターン
- [REST-API設計.md](REST-API設計.md) -- REST API エラーレスポンス・バージョニング・レート制限
- [GraphQL設計.md](GraphQL設計.md) -- GraphQL 設計・実装技術選定
- [proto設計.md](proto設計.md) -- gRPC サービス定義・Protobuf スキーマ・buf 設定
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka・イベント駆動設計
