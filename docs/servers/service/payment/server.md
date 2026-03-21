# service-payment-server 設計

service tier の決済管理サーバー設計を定義する。決済の開始・照会・完了・失敗処理・返金を REST API + gRPC で提供し、決済イベントを Kafka に非同期配信する。
Rust で実装する。

## 概要

### RBAC対応表

| ロール名 | リソース/アクション |
|---------|-----------------|
| payment:read | 決済一覧取得・単体取得 |
| payment:write | 決済開始・完了・失敗・返金 |

Tier: `Tier::Service`。JWKS ベースの JWT 認証と、`require_permission(Tier::Service, "payment", action)` による権限チェックを行う。

service tier の決済管理サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| 決済開始 API | 注文に紐づく決済を開始し、初期ステータスを `initiated` に設定する |
| 決済取得 API | 決済 ID を指定して決済情報を取得する |
| 決済一覧取得 API | 注文 ID・顧客 ID・ステータスによるフィルタリング付きの決済一覧を取得する |
| 決済完了 API | 外部決済プロバイダからのトランザクション ID を記録して決済を完了する |
| 決済失敗 API | エラーコード・メッセージを記録して決済を失敗にする |
| 決済返金 API | 完了済み決済を返金する |
| 決済イベント配信 | Kafka トピックへの決済ライフサイクルイベントの非同期配信 |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| Kafka クライアント | rdkafka v0.36 |
| gRPC | tonic v0.12 |

### 配置パス

配置: `regions/service/payment/server/rust/payment/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SVC_PAYMENT_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/payments` | 決済開始 | `payment:write` |
| GET | `/api/v1/payments` | 決済一覧取得（フィルター付き） | `payment:read` |
| GET | `/api/v1/payments/{id}` | 決済詳細取得 | `payment:read` |
| POST | `/api/v1/payments/{id}/complete` | 決済完了 | `payment:write` |
| POST | `/api/v1/payments/{id}/fail` | 決済失敗 | `payment:write` |
| POST | `/api/v1/payments/{id}/refund` | 決済返金 | `payment:write` |
| GET | `/healthz` | ヘルスチェック | 不要（公開） |
| GET | `/readyz` | レディネスチェック | 不要（公開） |
| GET | `/metrics` | Prometheus メトリクス | 不要（公開） |

#### POST /api/v1/payments

決済を開始する。初期ステータスは `initiated`。

**リクエスト**

```json
{
  "order_id": "660e8400-e29b-41d4-a716-446655440111",
  "customer_id": "CUST-001",
  "amount": 3000,
  "currency": "JPY",
  "payment_method": "credit_card"
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `order_id` | string | Yes | 注文 ID |
| `customer_id` | string | Yes | 顧客 ID |
| `amount` | int64 | Yes | 決済金額（最小通貨単位） |
| `currency` | string | Yes | 通貨コード（例: `JPY`, `USD`） |
| `payment_method` | string | No | 決済方法（例: `credit_card`, `bank_transfer`） |

**レスポンス（201 Created）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "order_id": "660e8400-e29b-41d4-a716-446655440111",
  "customer_id": "CUST-001",
  "amount": 3000,
  "currency": "JPY",
  "status": "initiated",
  "payment_method": "credit_card",
  "version": 1,
  "created_at": "2026-03-01T00:00:00+00:00",
  "updated_at": "2026-03-01T00:00:00+00:00"
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SVC_PAYMENT_VALIDATION_FAILED",
    "message": "amount must be greater than 0",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/payments

決済一覧をフィルタ条件付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `order_id` | string | No | - | 注文 ID でフィルタ |
| `customer_id` | string | No | - | 顧客 ID でフィルタ |
| `status` | string | No | - | ステータスでフィルタ（`initiated`, `completed`, `failed`, `refunded`） |
| `limit` | int | No | 50 | 取得件数上限 |
| `offset` | int | No | 0 | オフセット |

**レスポンス（200 OK）**

```json
{
  "payments": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "order_id": "660e8400-e29b-41d4-a716-446655440111",
      "customer_id": "CUST-001",
      "amount": 3000,
      "currency": "JPY",
      "status": "initiated",
      "payment_method": "credit_card",
      "version": 1,
      "created_at": "2026-03-01T00:00:00+00:00",
      "updated_at": "2026-03-01T00:00:00+00:00"
    }
  ],
  "total": 42
}
```

#### GET /api/v1/payments/{id}

決済 ID を指定して詳細を取得する。

**レスポンス（200 OK）**

POST /api/v1/payments の 201 レスポンスと同一形式。

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SVC_PAYMENT_NOT_FOUND",
    "message": "Payment '550e8400-...' not found",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/payments/{id}/complete

決済を完了する。外部決済プロバイダのトランザクション ID を記録する。楽観的ロック付き。

**リクエスト**

```json
{
  "transaction_id": "txn_abc123def456"
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `transaction_id` | string | Yes | 外部決済プロバイダのトランザクション ID |

**レスポンス（200 OK）**

決済詳細レスポンスと同一形式（`status` が `completed`、`transaction_id` が記録される）。

**レスポンス（400 Bad Request — 不正なステータス遷移）**

```json
{
  "error": {
    "code": "SVC_PAYMENT_INVALID_STATUS_TRANSITION",
    "message": "cannot complete payment in status 'failed'",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/payments/{id}/fail

決済を失敗にする。エラーコードとメッセージを記録する。楽観的ロック付き。

**リクエスト**

```json
{
  "error_code": "CARD_DECLINED",
  "error_message": "The card was declined by the issuer"
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `error_code` | string | Yes | エラーコード |
| `error_message` | string | Yes | エラーメッセージ |

**レスポンス（200 OK）**

決済詳細レスポンスと同一形式（`status` が `failed`、`error_code` / `error_message` が記録される）。

#### POST /api/v1/payments/{id}/refund

完了済み決済を返金する。楽観的ロック付き。

**リクエスト**

```json
{
  "reason": "customer request"
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `reason` | string | No | 返金理由 |

**レスポンス（200 OK）**

決済詳細レスポンスと同一形式（`status` が `refunded`）。

### gRPC サービス定義

カノニカル定義ファイル: `api/proto/k1s0/service/payment/v1/payment.proto`

```protobuf
service PaymentService {
  rpc InitiatePayment(InitiatePaymentRequest) returns (InitiatePaymentResponse);
  rpc GetPayment(GetPaymentRequest) returns (GetPaymentResponse);
  rpc ListPayments(ListPaymentsRequest) returns (ListPaymentsResponse);
  rpc CompletePayment(CompletePaymentRequest) returns (CompletePaymentResponse);
  rpc FailPayment(FailPaymentRequest) returns (FailPaymentResponse);
  rpc RefundPayment(RefundPaymentRequest) returns (RefundPaymentResponse);
}
```

---

## 決済ステータス ステートマシン

```
┌───────────┐   complete   ┌───────────┐   refund   ┌──────────┐
│ initiated │─────────────>│ completed │──────────>│ refunded │
└─────┬─────┘              └───────────┘            └──────────┘
      │
      │ fail
      ▼
┌──────────┐
│  failed  │
└──────────┘
```

| 遷移元 | 遷移先 |
| --- | --- |
| initiated | completed, failed |
| completed | refunded |

`failed` と `refunded` は終端ステータスであり、他のステータスへ遷移できない。

---

## エラーコード

| エラーコード | HTTP Status | 説明 |
| --- | --- | --- |
| `SVC_PAYMENT_NOT_FOUND` | 404 | 指定された決済が見つからない |
| `SVC_PAYMENT_VALIDATION_FAILED` | 400 | リクエストのバリデーションエラー |
| `SVC_PAYMENT_INVALID_STATUS_TRANSITION` | 400 | 不正なステータス遷移 |
| `SVC_PAYMENT_VERSION_CONFLICT` | 409 | 楽観的ロックによるバージョン競合 |
| `SVC_PAYMENT_INTERNAL_ERROR` | 500 | 内部サーバーエラー |

---

## 冪等性保証

### create の ON CONFLICT 方式

`payment_repository.rs` の `create` メソッドは `ON CONFLICT (order_id) DO NOTHING` を使用して冪等な INSERT を実現する。同一 `order_id` で競合した場合は RETURNING が空になり、既存レコードをフォールバック SELECT で取得して返す。

```
INSERT INTO payments ... ON CONFLICT (order_id) DO NOTHING RETURNING *
→ 競合時は None → SELECT で既存レコードを取得
```

`InitiatePaymentUseCase` は `find_by_order_id` による事前チェック + `create` の二段構成で、アプリケーション層とデータベース層の双方で重複防止を実現する（二重課金防止）。

---

## Kafka イベント

決済のライフサイクルイベントを Kafka トピックに非同期配信する。Outbox Pattern で at-least-once delivery を保証する。

| トピック | イベント | トリガー |
| --- | --- | --- |
| `k1s0.service.payment.initiated.v1` | payment.initiated | 決済開始時 |
| `k1s0.service.payment.completed.v1` | payment.completed | 決済完了時 |
| `k1s0.service.payment.failed.v1` | payment.failed | 決済失敗時 |
| `k1s0.service.payment.refunded.v1` | payment.refunded | 決済返金時 |

### イベントペイロード例

**payment.initiated**

```json
{
  "metadata": {
    "event_id": "550e8400-e29b-41d4-a716-446655440000",
    "event_type": "payment.initiated",
    "source": "payment-server",
    "timestamp": 1740787200000,
    "schema_version": 1
  },
  "payment_id": "550e8400-e29b-41d4-a716-446655440000",
  "order_id": "660e8400-e29b-41d4-a716-446655440111",
  "customer_id": "CUST-001",
  "amount": 3000,
  "currency": "JPY"
}
```

**payment.completed**

```json
{
  "metadata": {
    "event_id": "550e8400-e29b-41d4-a716-446655440001",
    "event_type": "payment.completed",
    "source": "payment-server",
    "timestamp": 1740787200000,
    "schema_version": 1
  },
  "payment_id": "550e8400-e29b-41d4-a716-446655440000",
  "order_id": "660e8400-e29b-41d4-a716-446655440111",
  "transaction_id": "txn_abc123def456",
  "amount": 3000,
  "currency": "JPY"
}
```

**payment.failed**

```json
{
  "metadata": {
    "event_id": "550e8400-e29b-41d4-a716-446655440002",
    "event_type": "payment.failed",
    "source": "payment-server",
    "timestamp": 1740787200000,
    "schema_version": 1
  },
  "payment_id": "550e8400-e29b-41d4-a716-446655440000",
  "order_id": "660e8400-e29b-41d4-a716-446655440111",
  "error_code": "CARD_DECLINED",
  "error_message": "The card was declined by the issuer"
}
```

**payment.refunded**

```json
{
  "metadata": {
    "event_id": "550e8400-e29b-41d4-a716-446655440003",
    "event_type": "payment.refunded",
    "source": "payment-server",
    "timestamp": 1740787200000,
    "schema_version": 1
  },
  "payment_id": "550e8400-e29b-41d4-a716-446655440000",
  "order_id": "660e8400-e29b-41d4-a716-446655440111",
  "amount": 3000,
  "currency": "JPY",
  "reason": "customer request"
}
```

---

## 設定フィールド

### server

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `host` | string | `0.0.0.0` | バインドアドレス |
| `port` | int | `8312` | REST API ポート |
| `grpc_port` | int | `50073` | gRPC ポート |

### database

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `host` | string | - | PostgreSQL ホスト |
| `port` | int | `5432` | PostgreSQL ポート |
| `name` | string | - | データベース名 |
| `schema` | string | `payment_service` | スキーマ名 |
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
| `payment_initiated_topic` | string | 決済開始イベントのトピック名 |
| `payment_completed_topic` | string | 決済完了イベントのトピック名 |
| `payment_failed_topic` | string | 決済失敗イベントのトピック名 |
| `payment_refunded_topic` | string | 決済返金イベントのトピック名 |
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
  name: "k1s0-payment-server"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8312
  grpc_port: 50073

database:
  host: "postgres.k1s0-service.svc.cluster.local"
  port: 5432
  name: "k1s0_payment"
  schema: "payment_service"
  user: "app"
  password: ""
  ssl_mode: "require"
  max_connections: 25
  max_idle_conns: 5
  conn_max_lifetime: 300

kafka:
  brokers:
    - "kafka.k1s0-infra.svc.cluster.local:9092"
  payment_initiated_topic: "k1s0.service.payment.initiated.v1"
  payment_completed_topic: "k1s0.service.payment.completed.v1"
  payment_failed_topic: "k1s0.service.payment.failed.v1"
  payment_refunded_topic: "k1s0.service.payment.refunded.v1"
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
| domain/entity | `Payment`, `PaymentStatus`, `InitiatePayment`, `PaymentFilter` | エンティティ・値オブジェクト定義 |
| domain/repository | `PaymentRepository` | リポジトリトレイト |
| usecase | `InitiatePaymentUseCase`, `GetPaymentUseCase`, `ListPaymentsUseCase`, `CompletePaymentUseCase`, `FailPaymentUseCase`, `RefundPaymentUseCase` | ユースケース |
| adapter/handler | REST ハンドラー, gRPC ハンドラー | プロトコル変換（axum / tonic） |
| adapter/middleware | `auth_middleware`, `rbac_middleware` | JWT 認証・RBAC ミドルウェア |
| infrastructure/database | `PaymentPostgresRepository` | PostgreSQL リポジトリ実装 |
| infrastructure/config | Config ローダー | config.yaml の読み込みとバリデーション |
| infrastructure/kafka | `PaymentKafkaProducer` | Kafka プロデューサー（決済イベント配信） |
| infrastructure/outbox | `OutboxPoller` | Outbox パターンによるイベントポーリング |

### ドメインモデル

#### Payment

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | 決済の一意識別子 |
| `order_id` | string | 注文 ID |
| `customer_id` | string | 顧客 ID |
| `amount` | i64 | 決済金額（最小通貨単位） |
| `currency` | string | 通貨コード |
| `status` | PaymentStatus | 決済ステータス（`initiated`, `completed`, `failed`, `refunded`） |
| `payment_method` | string? | 決済方法 |
| `transaction_id` | string? | 外部決済プロバイダのトランザクション ID（完了時に記録） |
| `error_code` | string? | エラーコード（失敗時に記録） |
| `error_message` | string? | エラーメッセージ（失敗時に記録） |
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

- [service-payment-implementation.md](implementation.md) -- Rust 実装詳細（Cargo.toml・ドメイン・リポジトリ・ユースケース・ハンドラー）
- [service-payment-database.md](database.md) -- データベーススキーマ・マイグレーション・ER 図

---

## クライアント実装

| プラットフォーム | 配置パス | 技術スタック |
|----------------|---------|-------------|
| React | `regions/service/payment/client/react/payment/` | TanStack Query + Router, Zod, Axios |
| Flutter | `regions/service/payment/client/flutter/payment/` | Riverpod, go_router, Dio |

両クライアントとも BFF 経由で本サーバーの REST API を呼び出す。

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [service-order-server.md](../order/server.md) -- order-server 設計（注文管理）
- [service-inventory-server.md](../inventory/server.md) -- inventory-server 設計（在庫管理）
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Kafka トピック・イベント設計

## ObservabilityConfig（log/trace/metrics）

本サーバーの observability 設定は共通仕様を採用する。log / trace / metrics の構造と推奨値は [共通実装](../_common/implementation.md) の「ObservabilityConfig（log/trace/metrics）」を参照。
