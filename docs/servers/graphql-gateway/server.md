# system-graphql-gateway 設計

> **ガイド**: 設計背景・実装例は [server.guide.md](./server.guide.md) を参照。

system tier の GraphQL BFF ゲートウェイ。複数 gRPC バックエンドを単一 GraphQL スキーマに集約する。Rust（async-graphql）実装。

## 概要

| 機能 | 説明 |
| --- | --- |
| GraphQL スキーマ集約 | 認証・設定・テナント等の system サービスを単一スキーマに統合 |
| DataLoader によるバッチ処理 | N+1 問題を DataLoader で解決し、バックエンドへの呼び出しを最小化 |
| サブスクリプション | WebSocket 経由のリアルタイム更新（イベント配信） |
| イントロスペクション | 開発環境のみ GraphQL スキーマイントロスペクションを有効化 |
| 認証ミドルウェア | JWT 検証により認証済みリクエストのみを許可 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| GraphQL | async-graphql v7 |

### 配置パス

配置: `regions/system/server/rust/graphql-gateway/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## API 定義

### REST / GraphQL エンドポイント

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/graphql` | GraphQL クエリ / ミューテーション | JWT 必須 |
| GET | `/graphql` | GraphQL Playground（development のみ） | 不要 |
| POST | `/graphql/ws` | WebSocket サブスクリプション | JWT 必須 |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### POST /graphql

GraphQL クエリおよびミューテーションを受け付ける。リクエストボディは `application/json` 形式。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `query` | string | Yes | GraphQL クエリ文字列 |
| `variables` | object | No | クエリ変数 |
| `operationName` | string | No | 実行するオペレーション名 |

**エラーレスポンス**: GraphQL 仕様に従い HTTP 200 で `errors` フィールドを返却。JWT 欠落・無効時のみ HTTP 401。

#### POST /graphql/ws（WebSocket サブスクリプション）

`graphql-ws` プロトコルを使用。接続時に `connection_init` メッセージで JWT を送信する。

### GraphQL スキーマ（主要型）

```graphql
# api/graphql/schema.graphql

type Query {
  tenant(id: ID!): Tenant
  tenants(page: Int, pageSize: Int): TenantConnection!
  featureFlag(key: String!): FeatureFlag
  featureFlags(environment: String): [FeatureFlag!]!
  config(key: String!): ConfigEntry
}

type Mutation {
  createTenant(input: CreateTenantInput!): Tenant!
  updateTenant(id: ID!, input: UpdateTenantInput!): Tenant!
  setFeatureFlag(key: String!, input: SetFeatureFlagInput!): FeatureFlag!
}

type Subscription {
  tenantUpdated(tenantId: ID!): Tenant!
  featureFlagChanged(key: String!): FeatureFlag!
  configChanged(namespaces: [String!]): ConfigEntry!
}

type Tenant {
  id: ID!
  name: String!
  status: TenantStatus!
  createdAt: String!
  updatedAt: String!
}

enum TenantStatus {
  ACTIVE
  SUSPENDED
  DELETED
}

type FeatureFlag {
  key: String!
  name: String!
  enabled: Boolean!
  rolloutPercentage: Int!
  targetEnvironments: [String!]!
}

type ConfigEntry {
  key: String!
  value: String!
  updatedAt: String!
}

type TenantConnection {
  nodes: [Tenant!]!
  totalCount: Int!
  hasNext: Boolean!
}

input CreateTenantInput {
  name: String!
}

input UpdateTenantInput {
  name: String
  status: TenantStatus
}

input SetFeatureFlagInput {
  enabled: Boolean!
  rolloutPercentage: Int
  targetEnvironments: [String!]
}
```

### エラーコード（GraphQL extensions.code）

| コード | 説明 |
| --- | --- |
| `NOT_FOUND` | 要求したリソースが見つからない |
| `UNAUTHORIZED` | JWT 認証エラー |
| `FORBIDDEN` | 権限不足 |
| `VALIDATION_ERROR` | 入力バリデーションエラー |
| `BACKEND_ERROR` | バックエンド gRPC 呼び出しエラー |
| `INTERNAL_ERROR` | 内部エラー |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/model | GraphQL 出力型（`Tenant`, `FeatureFlag`, `ConfigEntry` 等） | GraphQL スキーマ型定義 |
| domain/loader | DataLoader インターフェース | バッチ取得トレイト |
| usecase | `TenantQueryResolver`, `FeatureFlagQueryResolver`, `ConfigQueryResolver`, `TenantMutationResolver`, `SubscriptionResolver` | クエリ・ミューテーション・サブスクリプション解決 |
| adapter/graphql | async-graphql の Query / Mutation / Subscription 実装 | GraphQL レイヤー |
| adapter/middleware | JWT 検証ミドルウェア（axum layer） | 認証処理 |
| infra/config | Config ローダー | config.yaml の読み込み |
| infra/grpc | `TenantGrpcClient`, `FeatureFlagGrpcClient`, `ConfigGrpcClient` | tonic gRPC クライアント |
| infra/auth | JWT 検証実装 | JWKS 取得・署名検証 |

### ドメインモデル

#### GraphQL Context

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `user_id` | String | JWT から取得したユーザー ID |
| `roles` | Vec\<String\> | JWT から取得したロールリスト |
| `request_id` | String | リクエスト追跡 ID |
| `tenant_loader` | DataLoader | テナントバッチローダー |
| `flag_loader` | DataLoader | フィーチャーフラグバッチローダー |

#### DataLoader 設計

| DataLoader | バッチキー | 呼び出し先 gRPC |
| --- | --- | --- |
| `TenantLoader` | テナント ID リスト | TenantService.BatchGetTenants |
| `FeatureFlagLoader` | フラグキーリスト | FeatureFlagService.ListFlags |
| `ConfigLoader` | 設定キーリスト | ConfigService.BatchGetConfigs |

---

## 設定フィールド

### graphql

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `introspection` | bool | `false` | スキーマイントロスペクション有効化（development のみ推奨） |
| `playground` | bool | `false` | GraphQL Playground 有効化（development のみ推奨） |
| `max_depth` | int | `10` | クエリネスト深度の上限 |
| `max_complexity` | int | `1000` | クエリ複雑度の上限 |

### auth

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `jwks_url` | string | JWKS エンドポイント URL |

### backends

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `tenant.address` | string | テナントサービス gRPC エンドポイント |
| `tenant.timeout_ms` | int | リクエストタイムアウト（ミリ秒） |
| `featureflag.address` | string | フィーチャーフラグサービス gRPC エンドポイント |
| `featureflag.timeout_ms` | int | リクエストタイムアウト（ミリ秒） |
| `config.address` | string | 設定サービス gRPC エンドポイント |
| `config.timeout_ms` | int | リクエストタイムアウト（ミリ秒） |

---

## デプロイ

ポート構成:

| プロトコル | ポート | 説明 |
| --- | --- | --- |
| REST / GraphQL (HTTP) | 8080 | GraphQL API + Playground |

---

## 詳細設計ドキュメント

- [system-graphql-gateway-implementation.md](implementation.md) -- 実装設計の詳細
- [system-graphql-gateway-deploy.md](deploy.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [RBAC設計.md](../../architecture/auth/RBAC設計.md) -- RBAC ロールモデル
- [GraphQL設計.md](../../architecture/api/GraphQL設計.md) -- GraphQL 設計ガイドライン
- [テンプレート仕様-BFF.md](../../templates/client/BFF.md) -- BFF テンプレート仕様
- [system-server.md](../auth/server.md) -- system tier サーバー一覧
