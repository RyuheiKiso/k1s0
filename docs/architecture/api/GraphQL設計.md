# GraphQL 設計

D-011 GraphQL 設計、D-124 実装技術選定を定義する。

元ドキュメント: [API設計.md](./API設計.md)

---

## D-011: GraphQL 設計

GraphQL は BFF（Backend for Frontend）として、複数サービスの集約が必要な場合にオプション採用する。

```
Client → Nginx Ingress Controller → Kong → (Istio Sidecar) → GraphQL BFF → gRPC (mTLS) → Backend Services
```

### 採用方針

GraphQL は **BFF（Backend for Frontend）としてオプション採用** する。すべてのサービスに GraphQL を導入するのではなく、複数の REST / gRPC エンドポイントを集約して単一のクエリでクライアントに提供する必要がある場合に採用する。

#### 導入フェーズ

- **初期フェーズでは GraphQL BFF を採用しない**。REST API で十分に対応可能な段階では REST を使用する
- フロントエンドの複雑性が増し、導入条件を満たした段階で GraphQL BFF の導入を検討する
- 導入判断はフロントエンドチームとバックエンドチームの合意のもとで行う

### GraphQL BFF 導入基準

| 条件 | 説明 |
| --- | --- |
| サービス集約数 | **1つの画面で3つ以上のマイクロサービス**からデータを集約する必要がある場合 |
| フィールド差異 | クライアント種別（Web / Mobile）によって必要なフィールドが大きく異なる場合 |
| レスポンス最適化 | モバイル向けにレスポンスサイズの最小化が必要な場合 |

### 導入対象候補

| 画面 | 集約対象サービス例 | 理由 |
| --- | --- | --- |
| ダッシュボード画面 | 注文、在庫、会計、ユーザーなど | 複数ドメインの集約データを一覧表示する |
| レポート画面 | 売上、在庫推移、顧客分析など | 複数サービスの分析データを統合表示する |
| ユーザープロフィール画面 | ユーザー情報、注文履歴、通知など | ユーザーに紐づく複数サービスのデータを表示する |

### System Tier GraphQL ゲートウェイ

System Tier では、基盤サービス（テナント・設定・フィーチャーフラグ等）の集約レイヤーとして GraphQL ゲートウェイを配置する。Service Tier の BFF とは異なり、system tier の共通サービスを統一的に集約する役割を持つ。

| 項目 | 説明 |
| --- | --- |
| 配置パス | `regions/system/server/rust/graphql-gateway/` |
| 役割 | system tier の複数 gRPC バックエンドを単一 GraphQL スキーマに集約 |
| 対象サービス | テナント管理、フィーチャーフラグ、設定管理 |
| 実装言語 | Rust（async-graphql v7 + axum） |
| 詳細設計 | [graphql-gateway設計.md](../../servers/graphql-gateway/server.md) |

### REST vs GraphQL 使い分け基準（D-089）

原則: REST がデフォルト、GraphQL は条件を満たす場合のみ採用。

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

### REST vs GraphQL 判断フロー（D-089）

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

### GraphQL を採用してはならないケース

- **サービス間通信**: バックエンド間は gRPC を使用する。GraphQL はクライアント向け BFF 専用
- **単純な CRUD API**: REST で十分な場合に GraphQL を採用すると、不要な複雑性が増す
- **認証エンドポイント**: OAuth 2.0 の標準フローに従い REST で実装する

### クエリ制限

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

### スキーマ進化によるバージョニング

明示的なバージョニングを行わず、スキーマの進化的な変更で対応する。

| 変更種別           | 方法                                     |
| ------------------ | ---------------------------------------- |
| フィールド追加     | そのまま追加（既存クエリに影響なし）     |
| フィールド非推奨化 | `@deprecated(reason: "...")` を付与      |
| フィールド削除     | 非推奨化から 6 か月後に削除              |
| 型の追加           | そのまま追加                             |

GraphQL ではスキーマの進化的な変更（Evolutionary Schema Design）により、明示的なバージョニングを行わない。

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

## D-124: GraphQL 実装技術選定

### 技術選定

| 言語 | ライブラリ      | 方式             | 特徴                         |
| ---- | --------------- | ---------------- | ---------------------------- |
| Go   | gqlgen          | コード生成ベース | スキーマファースト、型安全   |
| Rust | async-graphql   | マクロベース     | 高パフォーマンス、型安全     |

### gqlgen 設定

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

### BFF ディレクトリ構成

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

---

## Go: gqlgen リゾルバー実装例

スキーマファースト開発で、GraphQL スキーマから Go のリゾルバーインターフェースを生成する。

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

## Rust: async-graphql リゾルバー実装例

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

## スキーマファースト開発フロー

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

- [API設計.md](./API設計.md) -- 基本方針・Tier 別 API 種別パターン
- [REST-API設計.md](REST-API設計.md) -- REST API エラーレスポンス・バージョニング・レート制限
- [gRPC設計.md](gRPC設計.md) -- gRPC サービス定義・バージョニング
- [ディレクトリ構成図.md](../../architecture/overview/ディレクトリ構成図.md) -- プロジェクトのディレクトリ構成
