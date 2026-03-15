# service-inventory-server 設計

service tier の在庫管理サーバー設計を定義する。商品の在庫予約・解放・照会・更新を REST API + gRPC で提供し、在庫イベントを Kafka に非同期配信する。
Rust で実装する。

## 概要

### RBAC対応表

| ロール名 | リソース/アクション |
|---------|-----------------|
| inventory:read | 在庫一覧取得・単体取得 |
| inventory:write | 在庫予約・解放・更新 |

Tier: `Tier::Service`。JWKS ベースの JWT 認証と、`require_permission(Tier::Service, "inventory", action)` による権限チェックを行う。

service tier の在庫管理サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| 在庫予約 API | 注文に紐づく在庫予約（qty_available から qty_reserved へ移動） |
| 在庫解放 API | 予約済み在庫の解放（キャンセル・返品時） |
| 在庫取得 API | 在庫 ID を指定して在庫情報を取得する |
| 在庫一覧取得 API | 商品 ID・倉庫 ID によるフィルタリング付きの在庫一覧を取得する |
| 在庫更新 API | 在庫数量を直接更新する（楽観的ロック付き） |
| 在庫イベント配信 | Kafka トピックへの在庫予約・解放イベントの非同期配信 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| Kafka クライアント | rdkafka v0.36 |
| gRPC | tonic v0.12 |

### 配置パス

配置: `regions/service/inventory/server/rust/inventory/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SVC_INVENTORY_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/inventory/reserve` | 在庫予約 | `inventory:write` |
| POST | `/api/v1/inventory/release` | 在庫解放 | `inventory:write` |
| GET | `/api/v1/inventory/:id` | 在庫取得 | `inventory:read` |
| GET | `/api/v1/inventory` | 在庫一覧取得（フィルター付き） | `inventory:read` |
| PUT | `/api/v1/inventory/:id` | 在庫数量更新（楽観ロック付き） | `inventory:write` |
| GET | `/healthz` | ヘルスチェック | 不要（公開） |
| GET | `/readyz` | レディネスチェック | 不要（公開） |
| GET | `/metrics` | Prometheus メトリクス | 不要（公開） |

#### POST /api/v1/inventory/reserve

注文に紐づく在庫を予約する。`qty_available` から `quantity` 分を `qty_reserved` に移動する。

**リクエスト**

```json
{
  "order_id": "660e8400-e29b-41d4-a716-446655440111",
  "product_id": "PROD-001",
  "warehouse_id": "WH-TOKYO-01",
  "quantity": 5
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `order_id` | string | Yes | 注文 ID |
| `product_id` | string | Yes | 商品 ID |
| `warehouse_id` | string | Yes | 倉庫 ID |
| `quantity` | int | Yes | 予約数量（1 以上） |

**レスポンス（200 OK）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "product_id": "PROD-001",
  "warehouse_id": "WH-TOKYO-01",
  "qty_available": 95,
  "qty_reserved": 5,
  "version": 2,
  "created_at": "2026-03-01T00:00:00+00:00",
  "updated_at": "2026-03-14T00:00:00+00:00"
}
```

**レスポンス（400 Bad Request — 在庫不足）**

```json
{
  "error": {
    "code": "SVC_INVENTORY_INSUFFICIENT_STOCK",
    "message": "insufficient stock: available=95, requested=100",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/inventory/release

予約済み在庫を解放する。`qty_reserved` から `quantity` 分を `qty_available` に戻す。

**リクエスト**

```json
{
  "order_id": "660e8400-e29b-41d4-a716-446655440111",
  "product_id": "PROD-001",
  "warehouse_id": "WH-TOKYO-01",
  "quantity": 5,
  "reason": "order cancelled"
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `order_id` | string | Yes | 注文 ID |
| `product_id` | string | Yes | 商品 ID |
| `warehouse_id` | string | Yes | 倉庫 ID |
| `quantity` | int | Yes | 解放数量（1 以上） |
| `reason` | string | No | 解放理由 |

**レスポンス（200 OK）**

在庫予約レスポンスと同一形式。

#### GET /api/v1/inventory/:id

在庫 ID を指定して在庫情報を取得する。

**レスポンス（200 OK）**

在庫予約レスポンスと同一形式。

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SVC_INVENTORY_NOT_FOUND",
    "message": "Inventory item '550e8400-...' not found",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/inventory

在庫一覧をフィルタ条件付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `product_id` | string | No | - | 商品 ID でフィルタ |
| `warehouse_id` | string | No | - | 倉庫 ID でフィルタ |
| `limit` | int | No | 50 | 取得件数上限 |
| `offset` | int | No | 0 | オフセット |

**レスポンス（200 OK）**

```json
{
  "items": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "product_id": "PROD-001",
      "warehouse_id": "WH-TOKYO-01",
      "qty_available": 95,
      "qty_reserved": 5,
      "version": 2,
      "created_at": "2026-03-01T00:00:00+00:00",
      "updated_at": "2026-03-14T00:00:00+00:00"
    }
  ],
  "total": 42
}
```

#### PUT /api/v1/inventory/:id

在庫数量を直接更新する。楽観的ロック（version）付き。

**リクエスト**

```json
{
  "qty_available": 200,
  "expected_version": 2
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `qty_available` | int | Yes | 新しい利用可能数量 |
| `expected_version` | int | Yes | 楽観的ロック用バージョン番号 |

**レスポンス（200 OK）**

在庫アイテムレスポンスと同一形式（更新後の値）。

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SVC_INVENTORY_VERSION_CONFLICT",
    "message": "version conflict for inventory item '550e8400-...'",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

### gRPC サービス定義

カノニカル定義ファイル: `api/proto/k1s0/service/inventory/v1/inventory.proto`

```protobuf
service InventoryService {
  rpc ReserveStock(ReserveStockRequest) returns (ReserveStockResponse);
  rpc ReleaseStock(ReleaseStockRequest) returns (ReleaseStockResponse);
  rpc GetInventory(GetInventoryRequest) returns (GetInventoryResponse);
  rpc ListInventory(ListInventoryRequest) returns (ListInventoryResponse);
  rpc UpdateStock(UpdateStockRequest) returns (UpdateStockResponse);
}
```

---

## エラーコード

| エラーコード | HTTP Status | 説明 |
| --- | --- | --- |
| `SVC_INVENTORY_NOT_FOUND` | 404 | 指定された在庫アイテムが見つからない |
| `SVC_INVENTORY_INSUFFICIENT_STOCK` | 400 | 在庫不足（予約数量 > 利用可能数量） |
| `SVC_INVENTORY_VERSION_CONFLICT` | 409 | 楽観的ロックによるバージョン競合 |
| `SVC_INVENTORY_VALIDATION_FAILED` | 400 | リクエストのバリデーションエラー |
| `SVC_INVENTORY_INTERNAL_ERROR` | 500 | 内部サーバーエラー |

---

## Kafka イベント

在庫の予約・解放イベントを Kafka トピックに非同期配信する。Kafka 接続が利用できない場合は Outbox Pattern で at-least-once delivery を保証する。

| トピック | イベント | トリガー |
| --- | --- | --- |
| `k1s0.service.inventory.reserved.v1` | inventory.reserved | 在庫予約時 |
| `k1s0.service.inventory.released.v1` | inventory.released | 在庫解放時 |

### イベントペイロード例

**inventory.reserved**

```json
{
  "metadata": {
    "event_id": "550e8400-e29b-41d4-a716-446655440000",
    "event_type": "inventory.reserved",
    "source": "inventory-server",
    "timestamp": 1740787200000,
    "schema_version": 1
  },
  "order_id": "660e8400-e29b-41d4-a716-446655440111",
  "product_id": "PROD-001",
  "warehouse_id": "WH-TOKYO-01",
  "quantity": 5
}
```

**inventory.released**

```json
{
  "metadata": {
    "event_id": "550e8400-e29b-41d4-a716-446655440001",
    "event_type": "inventory.released",
    "source": "inventory-server",
    "timestamp": 1740787200000,
    "schema_version": 1
  },
  "order_id": "660e8400-e29b-41d4-a716-446655440111",
  "product_id": "PROD-001",
  "warehouse_id": "WH-TOKYO-01",
  "quantity": 5,
  "reason": "order cancelled"
}
```

---

## 設定フィールド

### server

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `host` | string | `0.0.0.0` | バインドアドレス |
| `port` | int | `8311` | REST API ポート |
| `grpc_port` | int | `50072` | gRPC ポート |

### database

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `host` | string | - | PostgreSQL ホスト |
| `port` | int | `5432` | PostgreSQL ポート |
| `name` | string | - | データベース名 |
| `schema` | string | `inventory_service` | スキーマ名 |
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
| `inventory_reserved_topic` | string | 在庫予約イベントのトピック名 |
| `inventory_released_topic` | string | 在庫解放イベントのトピック名 |
| `security_protocol` | string | Kafka セキュリティプロトコル |

### auth

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `jwks_url` | string | - | JWKS エンドポイント URL |
| `issuer` | string | - | JWT issuer |
| `audience` | string | - | JWT audience |
| `jwks_cache_ttl_secs` | int | `300` | JWKS キャッシュ TTL（秒） |

### config.yaml（本番）

```yaml
app:
  name: "k1s0-inventory-server"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8311
  grpc_port: 50072

database:
  host: "postgres.k1s0-service.svc.cluster.local"
  port: 5432
  name: "k1s0_inventory"
  schema: "inventory_service"
  user: "app"
  password: ""
  ssl_mode: "require"
  max_connections: 25
  max_idle_conns: 5
  conn_max_lifetime: 300

kafka:
  brokers:
    - "kafka.k1s0-infra.svc.cluster.local:9092"
  inventory_reserved_topic: "k1s0.service.inventory.reserved.v1"
  inventory_released_topic: "k1s0.service.inventory.released.v1"
  security_protocol: "PLAINTEXT"

auth:
  jwks_url: "https://auth.k1s0.internal.example.com/realms/k1s0/protocol/openid-connect/certs"
  issuer: "https://auth.k1s0.internal.example.com/realms/k1s0"
  audience: "k1s0-service"
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
| domain/entity | `InventoryItem`, `InventoryFilter` | エンティティ・値オブジェクト定義 |
| domain/repository | `InventoryRepository` | リポジトリトレイト |
| usecase | `ReserveStockUseCase`, `ReleaseStockUseCase`, `GetInventoryUseCase`, `ListInventoryUseCase`, `UpdateStockUseCase` | ユースケース |
| adapter/handler | REST ハンドラー, gRPC ハンドラー | プロトコル変換（axum / tonic） |
| adapter/middleware | `auth_middleware`, `rbac_middleware` | JWT 認証・RBAC ミドルウェア |
| infrastructure/database | `InventoryPostgresRepository` | PostgreSQL リポジトリ実装 |
| infrastructure/config | Config ローダー | config.yaml の読み込みとバリデーション |
| infrastructure/kafka | `InventoryKafkaProducer` | Kafka プロデューサー（在庫イベント配信） |
| infrastructure/outbox | `OutboxPoller` | Outbox パターンによるイベントポーリング |

### ドメインモデル

#### InventoryItem

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | 在庫アイテムの一意識別子 |
| `product_id` | string | 商品 ID |
| `warehouse_id` | string | 倉庫 ID |
| `qty_available` | i32 | 利用可能数量（0 以上） |
| `qty_reserved` | i32 | 予約済み数量（0 以上） |
| `version` | i32 | バージョン番号（楽観的排他制御用） |
| `created_at` | timestamp | 作成日時 |
| `updated_at` | timestamp | 更新日時 |

### 依存ライブラリ

| ライブラリ | 説明 |
| --- | --- |
| `k1s0-auth` | JWKS ベース JWT 検証 |
| `k1s0-telemetry` | OpenTelemetry トレーシング・メトリクス |
| `k1s0-server-common` | 統一エラー型（`ServiceError`）、認証ミドルウェア、RBAC |

---

## 詳細設計ドキュメント

実装・データベースの詳細は以下の分割ドキュメントを参照。

- [service-inventory-implementation.md](implementation.md) -- Rust 実装詳細（Cargo.toml・ドメイン・リポジトリ・ユースケース・ハンドラー）
- [service-inventory-database.md](database.md) -- データベーススキーマ・マイグレーション・ER 図

---

## クライアント実装

| プラットフォーム | 配置パス | 技術スタック |
|----------------|---------|-------------|
| React | `regions/service/inventory/client/react/inventory/` | TanStack Query + Router, Zod, Axios |
| Flutter | `regions/service/inventory/client/flutter/inventory/` | Riverpod, go_router, Dio |

両クライアントとも BFF 経由で本サーバーの REST API を呼び出す。

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [service-order-server.md](../order/server.md) -- order-server 設計（注文管理）
- [service-payment-server.md](../payment/server.md) -- payment-server 設計（決済管理）
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka トピック・イベント設計

## ObservabilityConfig（log/trace/metrics）

本サーバーの observability 設定は共通仕様を採用する。log / trace / metrics の構造と推奨値は [共通実装](../_common/implementation.md) の「ObservabilityConfig（log/trace/metrics）」を参照。
