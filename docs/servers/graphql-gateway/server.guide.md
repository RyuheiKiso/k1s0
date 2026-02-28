# system-graphql-gateway 設計ガイド

> **仕様**: テーブル定義・APIスキーマは [server.md](./server.md) を参照。

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) および [GraphQL設計.md](../../architecture/api/GraphQL設計.md) に基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust（async-graphql v7 + axum） |
| 役割 | GraphQL はクライアント向け集約レイヤー（BFF）としてのみ使用。バックエンドは REST/gRPC を維持 |
| スキーマ管理 | GraphQL スキーマは `api/graphql/schema.graphql` に定義。コードファーストではなくスキーマファースト |
| バックエンド通信 | tonic gRPC クライアントで各バックエンドサービスを呼び出す |
| N+1 対策 | async-graphql の DataLoader を使用してバッチ化 |
| イントロスペクション | `environment: development` 時のみ有効。本番・ステージングでは無効化 |
| サブスクリプション | axum の WebSocket サポートを使用。`/graphql/ws` エンドポイント |
| 認証 | JWT 検証ミドルウェアを axum レイヤーに組み込み。`Authorization: Bearer` ヘッダー必須 |
| ポート | ホスト側 8095（内部 8080） |

---

## API リクエスト・レスポンス例

### POST /graphql

**リクエスト**

```json
{
  "query": "query GetTenant($id: ID!) { tenant(id: $id) { id name status createdAt } }",
  "variables": {
    "id": "tenant-abc"
  }
}
```

**レスポンス（200 OK）**

```json
{
  "data": {
    "tenant": {
      "id": "tenant-abc",
      "name": "株式会社サンプル",
      "status": "ACTIVE",
      "createdAt": "2026-02-20T10:00:00.000+00:00"
    }
  }
}
```

**レスポンス（200 OK -- エラー）**

GraphQL 仕様に従い、エラー時も HTTP 200 を返し `errors` フィールドにエラー情報を含める。

```json
{
  "data": null,
  "errors": [
    {
      "message": "tenant not found: tenant-abc",
      "locations": [{"line": 1, "column": 9}],
      "path": ["tenant"],
      "extensions": {
        "code": "NOT_FOUND",
        "request_id": "req_abc123def456"
      }
    }
  ]
}
```

**レスポンス（401 Unauthorized）**

JWT が無効または欠落している場合は HTTP 401 を返す（GraphQL レスポンスではなく HTTP エラー）。

```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "missing or invalid JWT token",
    "request_id": "req_abc123def456"
  }
}
```

### POST /graphql/ws（WebSocket サブスクリプション）

`graphql-ws` プロトコルを使用した WebSocket サブスクリプション。クライアントは接続確立後に `connection_init` メッセージで JWT を送信する。

**接続メッセージ（クライアント送信）**

```json
{
  "type": "connection_init",
  "payload": {
    "Authorization": "Bearer eyJhbGciOiJSUzI1NiJ9..."
  }
}
```

**サブスクリプション例**

```json
{
  "type": "subscribe",
  "id": "sub-001",
  "payload": {
    "query": "subscription OnTenantUpdated($tenantId: ID!) { tenantUpdated(tenantId: $tenantId) { id name status } }"
  }
}
```

---

## 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ GraphQL Handler (graphql_handler.rs)     │   │
                    │  │  POST /graphql (Query / Mutation)        │   │
                    │  │  GET /graphql (Playground)               │   │
                    │  │  POST /graphql/ws (Subscription)         │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ JWT Middleware (auth_middleware.rs)       │   │
                    │  │  Authorization ヘッダー検証              │   │
                    │  │  JWKS 署名検証                           │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  TenantQueryResolver / FeatureFlagQueryResolver │
                    │  ConfigQueryResolver / TenantMutationResolver   │
                    │  SubscriptionResolver                           │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/model   │              │ domain/loader              │   │
    │  Tenant,        │              │ TenantLoader               │   │
    │  FeatureFlag,   │              │ FeatureFlagLoader          │   │
    │  ConfigEntry,   │              │ ConfigLoader               │   │
    │  GraphqlContext │              │ (DataLoader trait)         │   │
    └────────────────┘              └──────────┬─────────────────┘   │
                    ┌──────────────────────────┼─────────────────────┘
                    │                  infra 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ JWT 検証     │  │ TenantGrpcClient       │  │
                    │  │ (JWKS)       │  ├────────────────────────┤  │
                    │  └──────────────┘  │ FeatureFlagGrpcClient  │  │
                    │  ┌──────────────┐  ├────────────────────────┤  │
                    │  │ Config       │  │ ConfigGrpcClient       │  │
                    │  │ Loader       │  └────────────────────────┘  │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

---

## 設定ファイル例

### config.yaml（本番）

```yaml
app:
  name: "graphql-gateway"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080

graphql:
  introspection: false
  playground: false
  max_depth: 10
  max_complexity: 1000

auth:
  jwks_url: "http://auth-server.k1s0-system.svc.cluster.local/jwks"

backends:
  tenant:
    address: "http://tenant-server.k1s0-system.svc.cluster.local:9090"
    timeout_ms: 3000
  featureflag:
    address: "http://featureflag-server.k1s0-system.svc.cluster.local:9090"
    timeout_ms: 3000
  config:
    address: "http://config-server.k1s0-system.svc.cluster.local:9090"
    timeout_ms: 3000
```

### Helm values

```yaml
# values-graphql-gateway.yaml（infra/helm/services/system/graphql-gateway/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/graphql-gateway
  tag: ""

replicaCount: 2

container:
  port: 8080

service:
  type: ClusterIP
  port: 80

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70

vault:
  enabled: false
```
