# GraphQL 設計

> **ガイド**: 設計背景・選定理由は [GraphQL設計.guide.md](./GraphQL設計.guide.md) を参照。

D-011 GraphQL 設計、D-124 実装技術選定を定義する。

元ドキュメント: [API設計.md](./API設計.md)

---

## D-011: GraphQL 設計

GraphQL は BFF（Backend for Frontend）として、複数サービスの集約が必要な場合にオプション採用する。

```
Client → Nginx Ingress Controller → Kong → (Istio Sidecar) → GraphQL BFF → gRPC (mTLS) → Backend Services
```

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

## 関連ドキュメント

- [API設計.md](./API設計.md) -- 基本方針・Tier 別 API 種別パターン
- [REST-API設計.md](REST-API設計.md) -- REST API エラーレスポンス・バージョニング・レート制限
- [gRPC設計.md](gRPC設計.md) -- gRPC サービス定義・バージョニング
- [ディレクトリ構成図.md](../../architecture/overview/ディレクトリ構成図.md) -- プロジェクトのディレクトリ構成
