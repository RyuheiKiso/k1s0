# GraphQL ガイド

> **仕様**: テーブル定義・APIスキーマは [GraphQL設計.md](./GraphQL設計.md) を参照。

## 採用方針

GraphQL は **BFF（Backend for Frontend）としてオプション採用** する。すべてのサービスに GraphQL を導入するのではなく、複数の REST / gRPC エンドポイントを集約して単一のクエリでクライアントに提供する必要がある場合に採用する。

### 導入フェーズ

- **初期フェーズでは GraphQL BFF を採用しない**。REST API で十分に対応可能な段階では REST を使用する
- フロントエンドの複雑性が増し、導入条件を満たした段階で GraphQL BFF の導入を検討する
- 導入判断はフロントエンドチームとバックエンドチームの合意のもとで行う

## REST vs GraphQL 判断フロー（D-089）

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

## スキーマ進化によるバージョニング不要方針

GraphQL ではスキーマの進化的な変更（Evolutionary Schema Design）により、明示的なバージョニングを行わない。

```graphql
type Order {
  id: ID!
  status: OrderStatus!
  totalAmount: Float! @deprecated(reason: "Use totalPrice instead")
  totalPrice: Money!
}
```

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
