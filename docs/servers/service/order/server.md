# service-order-server 設計

service tier の注文管理サーバー設計を定義する。顧客の注文作成・照会・ステータス管理を REST API で提供し、注文イベントを Kafka に非同期配信する。
Rust で実装する。

## 概要

### RBAC対応表

| ロール名 | リソース/アクション |
|---------|-----------------|
| order:read | 注文一覧取得・単体取得 |
| order:write | 注文作成・ステータス更新 |

Tier: `Tier::Service`。JWKS ベースの JWT 認証と、`require_permission(Tier::Service, "order", action)` による権限チェックを行う。

service tier の注文管理サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| 注文作成 API | 顧客 ID・明細を指定して注文を作成し、合計金額を自動計算する |
| 注文一覧取得 API | 顧客 ID・ステータスによるフィルタリング付きの注文一覧を取得する |
| 注文詳細取得 API | 注文 ID を指定して注文と明細を取得する |
| ステータス更新 API | 注文ステータスのステートマシンに従った遷移を行う |
| 注文イベント配信 | Kafka トピックへの注文作成・更新・キャンセルイベントの非同期配信 |
| **Saga Consumer** | `payment.completed/failed` を購読して注文ステータスを自動更新する（C-001） |

### Saga 補償フロー（C-001）

Choreography-based Saga パターンを採用。Order Consumer は payment イベントを購読して注文ステータスを更新する。

| 購読トピック | 処理 | 結果 |
|-----------|------|------|
| `payment.completed` | `UpdateOrderStatus(Confirmed)` | Saga 正常完了 |
| `payment.failed` | `UpdateOrderStatus(Cancelled)` | Saga 補償開始（order.cancelled → 在庫解放） |

- Consumer Group: `k1s0-order-consumer`（`kafka.consumer_group_id` で設定可能）
- 冪等性: `InvalidStatusTransition` エラーを "処理済み" としてスキップ
- 詳細: [イベント駆動設計.md](../../../libraries/_common/イベント駆動設計.md#saga-補償トランザクションパターンc-001)

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| Kafka クライアント | rdkafka v0.37 |
| バリデーション | validator v0.18 |

### 配置パス

配置: `regions/service/order/server/rust/order/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SVC_ORDER_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/orders` | 注文作成 | `order:write` |
| GET | `/api/v1/orders` | 注文一覧取得（フィルター付き） | `order:read` |
| GET | `/api/v1/orders/{order_id}` | 注文詳細取得（明細含む） | `order:read` |
| PUT | `/api/v1/orders/{order_id}/status` | 注文ステータス更新 | `order:write` |
| GET | `/healthz` | ヘルスチェック | 不要（公開） |
| GET | `/readyz` | レディネスチェック | 不要（公開） |
| GET | `/metrics` | Prometheus メトリクス | 不要（公開） |

#### POST /api/v1/orders

注文と明細を作成する。初期ステータスは `pending`。合計金額は明細の `quantity * unit_price` の合算で自動計算される。

**リクエスト**

```json
{
  "customer_id": "CUST-001",
  "currency": "JPY",
  "notes": "rush delivery",
  "items": [
    {
      "product_id": "PROD-001",
      "product_name": "Widget",
      "quantity": 3,
      "unit_price": 1000
    }
  ]
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `customer_id` | string | Yes | 顧客 ID |
| `currency` | string | Yes | 通貨コード（例: `JPY`, `USD`） |
| `notes` | string | No | 備考 |
| `items` | array | Yes | 注文明細（1件以上必須） |
| `items[].product_id` | string | Yes | 商品 ID |
| `items[].product_name` | string | Yes | 商品名 |
| `items[].quantity` | int | Yes | 数量（1 以上） |
| `items[].unit_price` | int64 | Yes | 単価（0 以上、最小通貨単位） |

**レスポンス（201 Created）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "customer_id": "CUST-001",
  "status": "pending",
  "total_amount": 3000,
  "currency": "JPY",
  "notes": "rush delivery",
  "items": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440111",
      "product_id": "PROD-001",
      "product_name": "Widget",
      "quantity": 3,
      "unit_price": 1000,
      "subtotal": 3000
    }
  ],
  "created_by": "admin@example.com",
  "version": 1,
  "created_at": "2026-03-01T00:00:00+00:00",
  "updated_at": "2026-03-01T00:00:00+00:00"
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SVC_ORDER_VALIDATION_FAILED",
    "message": "order must contain at least one item",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/orders

注文一覧をフィルタ条件付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `customer_id` | string | No | - | 顧客 ID でフィルタ |
| `status` | string | No | - | ステータスでフィルタ（`pending`, `confirmed`, 等） |
| `limit` | int | No | 50 | 取得件数上限 |
| `offset` | int | No | 0 | オフセット |

**レスポンス（200 OK）**

```json
{
  "orders": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "customer_id": "CUST-001",
      "status": "pending",
      "total_amount": 3000,
      "currency": "JPY",
      "created_at": "2026-03-01T00:00:00+00:00",
      "updated_at": "2026-03-01T00:00:00+00:00"
    }
  ],
  "total": 42
}
```

#### GET /api/v1/orders/{order_id}

注文の詳細（明細含む）を取得する。

**レスポンス（200 OK）**

POST /api/v1/orders の 201 レスポンスと同一形式。

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SVC_ORDER_NOT_FOUND",
    "message": "Order '550e8400-e29b-41d4-a716-446655440000' not found",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### PUT /api/v1/orders/{order_id}/status

注文ステータスを更新する。ステートマシンの遷移ルールに従い、不正な遷移は拒否される。

**リクエスト**

```json
{
  "status": "confirmed"
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `status` | string | Yes | 遷移先ステータス |

**レスポンス（200 OK）**

注文詳細レスポンスと同一形式（更新後のステータスが反映される）。

**レスポンス（400 Bad Request — 不正なステータス遷移）**

```json
{
  "error": {
    "code": "SVC_ORDER_INVALID_STATUS_TRANSITION",
    "message": "invalid status transition: 'delivered' -> 'pending'",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /healthz

**レスポンス（200 OK）**

```json
{
  "status": "ok"
}
```

#### GET /readyz

PostgreSQL への接続を確認する。

**レスポンス（200 OK）**

```json
{
  "status": "ready",
  "checks": {
    "database": "ok"
  }
}
```

**レスポンス（503 Service Unavailable）**

```json
{
  "status": "not_ready",
  "checks": {
    "database": "error: connection timeout"
  }
}
```

---

## 注文ステータス ステートマシン

```
                 ┌──────────────────────────────────────────────┐
                 │                                              │
                 ▼                                              │
┌─────────┐   ┌───────────┐   ┌────────────┐   ┌─────────┐   │
│ pending  │──>│ confirmed │──>│ processing │──>│ shipped │──>│ delivered │
└────┬─────┘   └─────┬─────┘   └──────┬─────┘   └─────────┘   └───────────┘
     │               │                │
     │               │                │
     ▼               ▼                ▼
┌───────────────────────────────────────┐
│             cancelled                  │
└───────────────────────────────────────┘
```

| 遷移元 | 遷移先 |
| --- | --- |
| pending | confirmed, cancelled |
| confirmed | processing, cancelled |
| processing | shipped, cancelled |
| shipped | delivered |

`delivered` と `cancelled` は終端ステータスであり、他のステータスへ遷移できない。

---

## エラーコード

| エラーコード | HTTP Status | 説明 |
| --- | --- | --- |
| `SVC_ORDER_NOT_FOUND` | 404 | 指定された注文が見つからない |
| `SVC_ORDER_VALIDATION_FAILED` | 400 | リクエストのバリデーションエラー |
| `SVC_ORDER_INVALID_STATUS_TRANSITION` | 400 | 不正なステータス遷移 |
| `SVC_ORDER_VERSION_CONFLICT` | 409 | 楽観的ロックによるバージョン競合 |
| `SVC_ORDER_INTERNAL_ERROR` | 500 | 内部サーバーエラー |

---

## Kafka イベント

注文のライフサイクルイベントを Kafka トピックに非同期配信する。Kafka 接続が利用できない場合は `NoopOrderEventPublisher` にフォールバックし、サーバーの起動を妨げない。

| トピック | イベント | トリガー |
| --- | --- | --- |
| `k1s0.service.order.created.v1` | order.created | 注文作成時 |
| `k1s0.service.order.updated.v1` | order.updated | ステータス更新時（cancelled 以外） |
| `k1s0.service.order.cancelled.v1` | order.cancelled | ステータスが cancelled に遷移時 |

### イベントペイロード例

メッセージング共通仕様に従い、metadata wrapper 構造 + epoch millis タイムスタンプを使用する。

**order.created**

```json
{
  "metadata": {
    "event_id": "550e8400-e29b-41d4-a716-446655440000",
    "event_type": "order.created",
    "source": "order-server",
    "timestamp": 1740787200000,
    "trace_id": "",
    "correlation_id": "660e8400-e29b-41d4-a716-446655440111",
    "schema_version": 1
  },
  "order_id": "660e8400-e29b-41d4-a716-446655440111",
  "customer_id": "CUST-001",
  "items": [
    {
      "product_id": "PROD-001",
      "quantity": 3,
      "unit_price": 1000
    }
  ],
  "total_amount": 3000,
  "currency": "JPY"
}
```

**order.updated**

```json
{
  "metadata": {
    "event_id": "550e8400-e29b-41d4-a716-446655440001",
    "event_type": "order.updated",
    "source": "order-server",
    "timestamp": 1740787200000,
    "trace_id": "",
    "correlation_id": "660e8400-e29b-41d4-a716-446655440111",
    "schema_version": 1
  },
  "order_id": "660e8400-e29b-41d4-a716-446655440111",
  "user_id": "admin@example.com",
  "status": "confirmed",
  "total_amount": 3000
}
```

**order.cancelled**

```json
{
  "metadata": {
    "event_id": "550e8400-e29b-41d4-a716-446655440002",
    "event_type": "order.cancelled",
    "source": "order-server",
    "timestamp": 1740787200000,
    "trace_id": "",
    "correlation_id": "660e8400-e29b-41d4-a716-446655440111",
    "schema_version": 1
  },
  "order_id": "660e8400-e29b-41d4-a716-446655440111",
  "user_id": "admin@example.com",
  "status": "cancelled",
  "total_amount": 3000,
  "reason": "status changed to cancelled"
}
```

---

## 設定フィールド

### server

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `host` | string | `0.0.0.0` | バインドアドレス |
| `port` | int | `8310` | REST API ポート |

### database

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `host` | string | - | PostgreSQL ホスト |
| `port` | int | `5432` | PostgreSQL ポート |
| `name` | string | - | データベース名 |
| `schema` | string | `order_service` | スキーマ名 |
| `user` | string | - | 接続ユーザー |
| `password` | string | `""` | パスワード |
| `ssl_mode` | string | `disable` | SSL モード |
| `max_connections` | int | `25` | 最大接続数 |
| `max_idle_conns` | int | `5` | 最小アイドル接続数 |
| `conn_max_lifetime` | int | `300` | 接続の最大ライフタイム（秒） |

### kafka

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `brokers` | string[] | Kafka ブローカーアドレス一覧 |
| `order_created_topic` | string | 注文作成イベントのトピック名 |
| `order_updated_topic` | string | 注文更新イベントのトピック名 |
| `order_cancelled_topic` | string | 注文キャンセルイベントのトピック名 |

### auth

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `jwks_url` | string | - | JWKS エンドポイント URL |
| `issuer` | string | - | JWT issuer |
| `audience` | string | - | JWT audience |
| `jwks_cache_ttl_secs` | int | `300` | JWKS キャッシュ TTL（秒） |

> **認証必須**: `auth` セクションは本番環境では必須。未設定の場合は起動時に `require_auth_state` によりエラーとなる。dev/test 環境では `ALLOW_INSECURE_NO_AUTH=true` を設定することで認証なし起動が可能（リリースビルドでは不可）。
> gRPC ハンドラー（GetOrder・ListOrders）はミドルウェアとは独立して Claims チェックを実施する（defense-in-depth）。

### config.yaml（本番）

```yaml
app:
  name: "k1s0-order-server"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8310

database:
  host: "postgres.k1s0-service.svc.cluster.local"
  port: 5432
  name: "k1s0_order"
  schema: "order_service"
  user: "app"
  password: ""
  ssl_mode: "require"
  max_connections: 25
  max_idle_conns: 5
  conn_max_lifetime: 300

kafka:
  brokers:
    - "kafka.k1s0-infra.svc.cluster.local:9092"
  order_created_topic: "k1s0.service.order.created.v1"
  order_updated_topic: "k1s0.service.order.updated.v1"
  order_cancelled_topic: "k1s0.service.order.cancelled.v1"

auth:
  jwks_url: "https://auth.k1s0.internal.example.com/realms/k1s0/protocol/openid-connect/certs"
  issuer: "https://auth.k1s0.internal.example.com/realms/k1s0"
  audience: "k1s0-api"
  jwks_cache_ttl_secs: 300

observability:
  log:
    level: "info"
    format: "json"
  trace:
    enabled: true
    endpoint: "http://otel-collector.observability:4317"
    sample_rate: 1.0
  metrics:
    enabled: true
    path: "/metrics"
```

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の4レイヤー構成に従う。

| レイヤー | パッケージ / モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `Order`, `OrderItem`, `OrderStatus`, `CreateOrder`, `CreateOrderItem`, `OrderFilter` | エンティティ・値オブジェクト定義 |
| domain/error | `OrderError` | ドメイン固有エラー型（`thiserror` ベース） |
| domain/repository | `OrderRepository` | リポジトリトレイト |
| domain/service | `OrderDomainService` | ドメインサービス（バリデーション・ステータス遷移検証・合計金額計算） |
| usecase | `CreateOrderUseCase`, `GetOrderUseCase`, `UpdateOrderStatusUseCase`, `ListOrdersUseCase` | ユースケース |
| usecase | `OrderEventPublisher` | イベント発行トレイト |
| adapter/handler | REST ハンドラー + ルーティング | プロトコル変換 |
| adapter/presenter | `OrderDetailResponse`, `OrderListResponse`, `OrderSummaryResponse` | ドメインモデル → API レスポンス変換 |
| adapter/middleware | `auth_middleware`, `require_permission` | JWT 認証・RBAC ミドルウェア |
| infrastructure/database | `OrderPostgresRepository` | PostgreSQL リポジトリ実装 |
| infrastructure/config | Config ローダー | config.yaml の読み込みとバリデーション |
| infrastructure/kafka | `OrderKafkaProducer` | Kafka プロデューサー（注文イベント配信） |

### ドメインモデル

#### Order

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | 注文の一意識別子 |
| `customer_id` | string | 顧客 ID |
| `status` | OrderStatus | 注文ステータス（`pending`, `confirmed`, `processing`, `shipped`, `delivered`, `cancelled`） |
| `total_amount` | i64 | 合計金額（最小通貨単位） |
| `currency` | string | 通貨コード |
| `notes` | string? | 備考 |
| `created_by` | string | 作成者 |
| `updated_by` | string? | 最終更新者 |
| `version` | i32 | バージョン番号（楽観的排他制御用） |
| `created_at` | timestamp | 作成日時 |
| `updated_at` | timestamp | 更新日時 |

#### OrderItem

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | 明細の一意識別子 |
| `order_id` | UUID | 親注文の ID（FK） |
| `product_id` | string | 商品 ID |
| `product_name` | string | 商品名 |
| `quantity` | i32 | 数量 |
| `unit_price` | i64 | 単価（最小通貨単位） |
| `subtotal` | i64 | 小計（`quantity * unit_price`） |
| `created_at` | timestamp | 作成日時 |

### 依存関係図

```
                    ┌────────────────────────────────────────────────────────┐
                    │                    adapter 層                          │
                    │  ┌──────────────┐  ┌────────────┐  ┌──────────────┐  │
                    │  │ REST Handler │  │ Middleware  │  │   Presenter  │  │
                    │  │ (order)      │  │ (auth/rbac) │  │              │  │
                    │  └──────┬───────┘  └──────┬──────┘  └──────┬───────┘  │
                    └─────────┼─────────────────┼────────────────┼──────────┘
                              │                 │                │
                    ┌─────────▼─────────────────▼────────────────▼──────────┐
                    │                   usecase 層                          │
                    │  CreateOrder / GetOrder / UpdateOrderStatus /         │
                    │  ListOrders / OrderEventPublisher                     │
                    └─────────┬────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────────────┐
              │               │                       │
    ┌─────────▼──────┐  ┌────▼───────────┐  ┌───────▼─────────────┐
    │  domain/entity  │  │ domain/service │  │ domain/repository   │
    │  Order,         │  │ OrderDomain    │  │ OrderRepository     │
    │  OrderItem,     │  │ Service        │  │ (trait)             │
    │  OrderStatus    │  │                │  │                     │
    └────────────────┘  └────────────────┘  └──────────┬──────────┘
                                                       │
                    ┌──────────────────────────────────┼──────────────┐
                    │             infrastructure 層    │              │
                    │  ┌──────────────┐  ┌─────────────▼──────────┐  │
                    │  │ Config       │  │ OrderPostgres          │  │
                    │  │ Loader       │  │ Repository (impl)      │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    │  ┌──────────────────────────────────────────┐  │
                    │  │ OrderKafkaProducer / NoopEventPublisher  │  │
                    │  └──────────────────────────────────────────┘  │
                    └────────────────────────────────────────────────┘
```

### 依存ライブラリ

| ライブラリ | 説明 |
| --- | --- |
| `k1s0-auth` | JWKS ベース JWT 検証 |
| `k1s0-telemetry` | OpenTelemetry トレーシング・メトリクス |
| `k1s0-server-common` | 統一エラー型（`ServiceError`）、認証ミドルウェア、RBAC、graceful shutdown |

---

## 詳細設計ドキュメント

実装・データベースの詳細は以下の分割ドキュメントを参照。

- [service-order-server-implementation.md](implementation.md) -- Rust 実装詳細（Cargo.toml・ドメイン・リポジトリ・ユースケース・ハンドラー）
- [service-order-database.md](database.md) -- データベーススキーマ・マイグレーション・ER 図

---

## クライアント実装

| プラットフォーム | 配置パス | 技術スタック |
|----------------|---------|-------------|
| React | `regions/service/order/client/react/order/` | TanStack Query + Router, Zod, Axios |
| Flutter | `regions/service/order/client/flutter/order/` | Riverpod, go_router, Dio |

両クライアントとも BFF 経由で本サーバーの REST API を呼び出す。

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [system-server.md](../auth/server.md) -- auth-server 設計（認証基盤）
- [config-server.md](../config/server.md) -- config-server 設計（設定管理）
- [APIゲートウェイ設計.md](../../architecture/api/APIゲートウェイ設計.md) -- Kong 構成管理
- [サービスメッシュ設計.md](../../infrastructure/service-mesh/サービスメッシュ設計.md) -- Istio 設計・mTLS

## ObservabilityConfig（log/trace/metrics）

本サーバーの observability 設定は共通仕様を採用する。log / trace / metrics の構造と推奨値は [共通実装](../_common/implementation.md) の「ObservabilityConfig（log/trace/metrics）」を参照。
