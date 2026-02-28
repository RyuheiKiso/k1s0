# gRPC 設計

D-009 サービス定義パターン、D-010 バージョニングを定義する。

元ドキュメント: [API設計.md](./API設計.md)

---

## D-009: gRPC サービス定義パターン

### proto パッケージ命名

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

全 proto ファイルは `api/proto/` に集約して配置する（詳細は [proto設計.md](proto設計.md) を参照）。

#### 配置ルールまとめ

| 対象 | 配置先 | 例 |
| --- | --- | --- |
| 複数サービスで共有する message 型・enum | `api/proto/k1s0/system/common/v1/` | Pagination, Timestamp, Money |
| イベント定義（Kafka メッセージスキーマ） | `api/proto/k1s0/event/{tier}/{domain}/v1/` | OrderCreatedEvent, LoginEvent |
| サービス固有の gRPC サービス定義 | `api/proto/k1s0/system/{domain}/v1/` | AuthService, SagaService |
| サービス固有の Request/Response 型 | `api/proto/k1s0/system/{domain}/v1/` | CreateOrderRequest |

> **注記**: gRPC サービス定義は Tier に関わらず `api/proto/k1s0/system/` に集約して配置する。詳細なディレクトリ構成は [proto設計.md](proto設計.md) を参照。

#### ディレクトリ構成

```
api/proto/
├── k1s0/
│   ├── event/                     # イベント定義（メッセージング設計.md 参照）
│   │   ├── system/
│   │   ├── business/
│   │   └── service/
│   └── system/                    # 全サービスの proto を集約（proto設計.md 参照）
│       ├── common/
│       │   └── v1/
│       │       ├── types.proto            # Pagination, Timestamp 等の共通型
│       │       └── event_metadata.proto   # イベントメタデータ
│       ├── auth/
│       │   └── v1/
│       │       └── auth.proto
│       └── {service}/
│           └── v1/
│               └── {service}.proto
├── buf.yaml                       # buf 設定
├── buf.gen.yaml                   # コード生成設定
└── buf.lock                       # 依存ロック
```

### 共通型定義

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

### Buf による管理

proto ファイルの lint・破壊的変更検出には [Buf](https://buf.build/) を使用する。

```yaml
# buf.yaml
version: v2
modules:
  - path: .
lint:
  use:
    - STANDARD
  except:
    - PACKAGE_VERSION_SUFFIX   # v1 パッケージを許容
breaking:
  use:
    - FILE
deps:
  - buf.build/googleapis/googleapis   # google.protobuf.Timestamp 等の標準型
```

```yaml
# buf.gen.yaml
version: v2
plugins:
  # --- Go ---
  - remote: buf.build/protocolbuffers/go
    out: gen/go
    opt:
      - paths=source_relative

  - remote: buf.build/grpc/go
    out: gen/go
    opt:
      - paths=source_relative

  # --- TypeScript (ts-proto) ---
  - remote: buf.build/community/timostamm-protobuf-ts
    out: gen/ts
    opt:
      - long_type_string

  # --- Rust (prost + tonic) ---
  - remote: buf.build/community/neoeinstein-prost
    out: gen/rust
    opt:
      - compile_well_known_types

  - remote: buf.build/community/neoeinstein-tonic
    out: gen/rust
    opt:
      - compile_well_known_types
```

---

## D-010: gRPC バージョニング

### パッケージレベルバージョニング

```
k1s0.service.order.v1  →  k1s0.service.order.v2
```

### パッケージレベルバージョニングの背景

gRPC のバージョニングは **proto パッケージ名にメジャーバージョンを含める** 方式を採用する。

バージョンアップ時は新しいパッケージディレクトリを作成し、旧バージョンと並行運用する。

```
api/proto/k1s0/service/order/
├── v1/
│   └── order.proto     # 旧バージョン（非推奨期間中は維持）
└── v2/
    └── order.proto     # 新バージョン
```

### 後方互換性ルール

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

---

## Go Interceptor 実装例

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

## Rust Interceptor 実装例

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

---

## 関連ドキュメント

- [API設計.md](./API設計.md) -- 基本方針・Tier 別 API 種別パターン
- [REST-API設計.md](REST-API設計.md) -- REST API エラーレスポンス・バージョニング・レート制限
- [GraphQL設計.md](GraphQL設計.md) -- GraphQL 設計・実装技術選定
- [proto設計.md](proto設計.md) -- gRPC サービス定義・Protobuf スキーマ・buf 設定
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka・イベント駆動設計
