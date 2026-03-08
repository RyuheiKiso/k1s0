# service-order-database設計

service Tier の注文管理データベース（order-db）の設計を定義する。
配置先: `regions/service/order/database/postgres/`

## 概要

order-db は service Tier に属する PostgreSQL 17 データベースであり、注文データと注文明細を管理する。Outbox Pattern によるイベント配信の信頼性を確保するための outbox_events テーブルも備える。

[tier-architecture.md](../../architecture/overview/tier-architecture.md) の設計原則に従い、order-db へのアクセスは **order-server からのみ** 許可する。他サービスが注文情報を必要とする場合は、order-server が提供する REST API を経由する。

### 技術スタック

| コンポーネント | 技術 | バージョン |
|----------------|------|------------|
| RDBMS | PostgreSQL | 17 |
| マイグレーション | sqlx-cli（`sqlx::migrate!` マクロ） | - |
| ORM / クエリビルダー | sqlx（Rust） | 0.8 |
| シークレット管理 | HashiCorp Vault | 1.17 |

---

## ER図

```
┌─────────────────────────┐
│        orders            │
├─────────────────────────┤
│ id (PK)                  │
│ customer_id              │
│ status                   │
│ total_amount             │         ┌──────────────────────────┐
│ currency                 │         │      order_items          │
│ notes                    │         ├──────────────────────────┤
│ created_by               │    1:N  │ id (PK)                  │
│ updated_by               │────────>│ order_id (FK)            │
│ version                  │         │ product_id               │
│ created_at               │         │ product_name             │
│ updated_at               │         │ quantity                 │
└─────────────────────────┘         │ unit_price               │
                                     │ subtotal                 │
                                     │ created_at               │
                                     └──────────────────────────┘

┌──────────────────────────┐
│     outbox_events         │
├──────────────────────────┤
│ id (PK)                   │
│ aggregate_type             │
│ aggregate_id               │
│ event_type                 │
│ payload (JSONB)            │
│ created_at                 │
│ published_at               │
└──────────────────────────┘
```

### リレーション

| 関係 | カーディナリティ | 説明 |
|------|-----------------|------|
| orders - order_items | 1:N | 1 件の注文は複数の明細を持つ。注文削除時に明細も CASCADE 削除される |
| outbox_events（独立） | -- | イベント配信用の独立テーブル。orders との直接 FK なし |

---

## テーブル定義

### orders テーブル

注文を管理するメインテーブル。ステータスはステートマシンに従って遷移する。楽観的ロック用の version カラムを持ち、同時更新を検知する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK | 注文識別子 |
| customer_id | TEXT | NOT NULL | 顧客 ID |
| status | TEXT | NOT NULL DEFAULT 'pending' | 注文ステータス（pending/confirmed/processing/shipped/delivered/cancelled） |
| total_amount | BIGINT | NOT NULL DEFAULT 0 | 合計金額（最小通貨単位） |
| currency | TEXT | NOT NULL DEFAULT 'JPY' | 通貨コード |
| notes | TEXT | | 備考 |
| created_by | TEXT | NOT NULL | 作成者 |
| updated_by | VARCHAR(255) | | 最終更新者（migration 003 で追加） |
| version | INT | NOT NULL DEFAULT 1 | 楽観的ロック用バージョン番号（migration 003 で追加） |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時 |

### order_items テーブル

注文明細テーブル。各明細の数量と単価、および計算済みの小計を保持する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK | 明細識別子 |
| order_id | UUID | FK orders(id) ON DELETE CASCADE, NOT NULL | 親注文 ID |
| product_id | TEXT | NOT NULL | 商品 ID |
| product_name | TEXT | NOT NULL | 商品名 |
| quantity | INT | NOT NULL CHECK (quantity > 0) | 数量（1 以上） |
| unit_price | BIGINT | NOT NULL CHECK (unit_price >= 0) | 単価（0 以上） |
| subtotal | BIGINT | NOT NULL | 小計（`quantity * unit_price`） |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |

### outbox_events テーブル

Outbox Pattern によるイベント配信テーブル。注文の作成・更新・キャンセル時にイベントを記録し、非同期で Kafka に配信する。`published_at` が NULL のレコードが未配信イベント。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK | イベント識別子 |
| aggregate_type | TEXT | NOT NULL | 集約タイプ（例: `order`） |
| aggregate_id | TEXT | NOT NULL | 集約 ID（注文 UUID） |
| event_type | TEXT | NOT NULL | イベントタイプ（例: `order.created`, `order.updated`, `order.cancelled`） |
| payload | JSONB | NOT NULL | イベントペイロード |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | イベント作成日時 |
| published_at | TIMESTAMPTZ | | Kafka 配信完了日時（NULL = 未配信） |

---

## インデックス設計

### インデックス一覧

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| orders | idx_orders_customer_id | customer_id | B-tree | 顧客 ID による注文検索 |
| orders | idx_orders_status | status | B-tree | ステータスによるフィルタリング |
| orders | idx_orders_created_at | created_at DESC | B-tree | 作成日時による降順ソート |
| order_items | idx_order_items_order_id | order_id | B-tree | 注文 ID による明細取得 |
| order_items | idx_order_items_product_id | product_id | B-tree | 商品 ID による検索 |
| outbox_events | idx_outbox_unpublished | created_at (WHERE published_at IS NULL) | B-tree (部分) | 未配信イベントの効率的な取得 |

### 設計方針

- **CHECK 制約**: `order_items.quantity > 0` と `order_items.unit_price >= 0` により、データ整合性を DB レベルで保証する
- **CASCADE 削除**: `order_items` は `orders` の削除に連動して自動削除される
- **部分インデックス**: `outbox_events` の未配信イベント検索は部分インデックスで高速化する
- **楽観的ロック**: `orders.version` カラムで同時更新を検知する。更新時に `WHERE version = $expected` を条件にし、不一致時はエラーを返す

---

## データフロー

```
┌──────────────┐   REST API   ┌───────────────┐    SQL     ┌──────────────┐
│  クライアント │───────────>│ order-server   │─────────>│   order-db   │
│  (React /    │            │               │          │              │
│   Flutter)   │            │  (注文作成)     │          │ orders       │
│              │            │  (ステータス更新)│          │ order_items  │
└──────────────┘            │  (注文照会)     │          │ outbox_events│
                            └───────────────┘          └──────────────┘
                                    │
                                    │ Kafka (rdkafka)
                                    ▼
                    ┌───────────────────────────────┐
                    │ k1s0.service.order.created.v1 │
                    │ k1s0.service.order.updated.v1 │
                    │ k1s0.service.order.cancelled.v1│
                    └───────────────────────────────┘
```

### フロー詳細

1. **注文作成**: クライアントが `POST /api/v1/orders` を呼び出し、トランザクション内で `orders` と `order_items` を INSERT。合計金額はドメインサービスで計算。初期ステータスは `pending`
2. **ステータス更新**: `PUT /api/v1/orders/:order_id/status` でステートマシンに従った遷移を実行。楽観的ロック（version）で同時更新を検知
3. **イベント配信**: 注文の作成・更新・キャンセル時に Kafka トピックへイベントを発行。配信失敗時はログ警告のみでリクエストは成功とする
4. **Outbox Pattern**: `outbox_events` テーブルに未配信イベントを記録し、バックグラウンドで Kafka に配信（at-least-once delivery）

---

## 主要クエリパターン

### 注文取得

```sql
-- 注文 ID で取得
SELECT id, customer_id, status, total_amount, currency, notes,
       created_by, updated_by, version, created_at, updated_at
FROM orders
WHERE id = $1;

-- 注文の明細一覧取得
SELECT id, order_id, product_id, product_name, quantity, unit_price,
       subtotal, created_at
FROM order_items
WHERE order_id = $1
ORDER BY created_at ASC;
```

### 注文一覧取得（フィルタ付き）

```sql
-- customer_id / status による任意フィルタ + ページネーション
SELECT id, customer_id, status, total_amount, currency, notes,
       created_by, updated_by, version, created_at, updated_at
FROM orders
WHERE ($1::text IS NULL OR customer_id = $1)
  AND ($2::text IS NULL OR status = $2)
ORDER BY created_at DESC
LIMIT $3 OFFSET $4;

-- 件数カウント
SELECT COUNT(*) FROM orders
WHERE ($1::text IS NULL OR customer_id = $1)
  AND ($2::text IS NULL OR status = $2);
```

### 注文作成（トランザクション）

```sql
BEGIN;

-- 注文レコード挿入
INSERT INTO orders (id, customer_id, status, total_amount, currency, notes,
                    created_by, version, created_at, updated_at)
VALUES ($1, $2, 'pending', $3, $4, $5, $6, 1, $7, $8)
RETURNING *;

-- 各明細レコード挿入
INSERT INTO order_items (id, order_id, product_id, product_name, quantity,
                         unit_price, subtotal, created_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
RETURNING *;

COMMIT;
```

### ステータス更新（楽観的ロック）

```sql
UPDATE orders
SET status = $2, updated_by = $3, version = version + 1, updated_at = $4
WHERE id = $1 AND version = $5
RETURNING id, customer_id, status, total_amount, currency, notes,
          created_by, updated_by, version, created_at, updated_at;
```

バージョン不一致時は 0 行が返されるため、アプリケーション層でエラー（`SVC_ORDER_VERSION_CONFLICT`）として処理する。

### 注文削除（トランザクション）

```sql
BEGIN;
DELETE FROM order_items WHERE order_id = $1;
DELETE FROM orders WHERE id = $1;
COMMIT;
```

---

## マイグレーションファイル

配置先: `regions/service/order/database/postgres/migrations/`

命名規則は [テンプレート仕様-データベース](../../templates/data/データベース.md) に準拠する。

```
migrations/
├── 001_create_orders.up.sql                        # スキーマ・orders テーブル・インデックス
├── 001_create_orders.down.sql
├── 002_create_order_items.up.sql                   # order_items テーブル・インデックス・CHECK 制約
├── 002_create_order_items.down.sql
├── 003_add_updated_by_and_version.up.sql           # updated_by・version 列追加
├── 003_add_updated_by_and_version.down.sql
├── 004_create_outbox.up.sql                        # outbox_events テーブル・部分インデックス
└── 004_create_outbox.down.sql
```

### マイグレーション一覧

| 番号 | ファイル名 | 説明 |
|------|-----------|------|
| 001 | create_orders | `order_service` スキーマ・`orders` テーブル・customer_id / status / created_at インデックス |
| 002 | create_order_items | `order_items` テーブル・FK (CASCADE)・order_id / product_id インデックス・quantity / unit_price CHECK 制約 |
| 003 | add_updated_by_and_version | `orders` テーブルに `updated_by` (VARCHAR(255)) と `version` (INT DEFAULT 1) 列を追加 |
| 004 | create_outbox | `outbox_events` テーブル・未配信イベント用部分インデックス |

### 001_create_orders.up.sql

```sql
CREATE SCHEMA IF NOT EXISTS order_service;

SET search_path TO order_service;

CREATE TABLE IF NOT EXISTS orders (
    id            UUID         PRIMARY KEY,
    customer_id   TEXT         NOT NULL,
    status        TEXT         NOT NULL DEFAULT 'pending',
    total_amount  BIGINT       NOT NULL DEFAULT 0,
    currency      TEXT         NOT NULL DEFAULT 'JPY',
    notes         TEXT,
    created_by    TEXT         NOT NULL,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_orders_customer_id ON orders (customer_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders (status);
CREATE INDEX IF NOT EXISTS idx_orders_created_at ON orders (created_at DESC);
```

### 002_create_order_items.up.sql

```sql
SET search_path TO order_service;

CREATE TABLE IF NOT EXISTS order_items (
    id            UUID         PRIMARY KEY,
    order_id      UUID         NOT NULL REFERENCES orders (id) ON DELETE CASCADE,
    product_id    TEXT         NOT NULL,
    product_name  TEXT         NOT NULL,
    quantity      INT          NOT NULL CHECK (quantity > 0),
    unit_price    BIGINT       NOT NULL CHECK (unit_price >= 0),
    subtotal      BIGINT       NOT NULL,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_order_items_order_id ON order_items (order_id);
CREATE INDEX IF NOT EXISTS idx_order_items_product_id ON order_items (product_id);
```

### 003_add_updated_by_and_version.up.sql

```sql
SET search_path TO order_service;

ALTER TABLE orders
    ADD COLUMN updated_by VARCHAR(255),
    ADD COLUMN version    INT NOT NULL DEFAULT 1;
```

### 004_create_outbox.up.sql

```sql
SET search_path TO order_service;

CREATE TABLE IF NOT EXISTS outbox_events (
    id             UUID         PRIMARY KEY,
    aggregate_type TEXT         NOT NULL,
    aggregate_id   TEXT         NOT NULL,
    event_type     TEXT         NOT NULL,
    payload        JSONB        NOT NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    published_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_outbox_unpublished
    ON outbox_events (created_at)
    WHERE published_at IS NULL;
```

---

## 接続設定

### config.yaml（order サーバー用）

```yaml
app:
  name: "order-server"
  version: "0.1.0"
  tier: "service"
  environment: "dev"

database:
  host: "postgres.k1s0-service.svc.cluster.local"
  port: 5432
  name: "k1s0_service"
  schema: "order_service"
  user: "app"
  password: ""
  ssl_mode: "disable"
  max_connections: 25
  max_idle_conns: 5
  conn_max_lifetime: 300
```

### Vault によるクレデンシャル管理

| 用途 | Vault パス | 説明 |
|------|-----------|------|
| 静的パスワード | `secret/data/k1s0/service/order-server/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/order-server-rw` | TTL: 24時間 |
| 動的クレデンシャル（読み取り専用） | `database/creds/order-server-ro` | TTL: 24時間 |

---

## 関連ドキュメント

- [service-order-server.md](server.md) -- Order サーバー設計（API・アーキテクチャ）
- [service-order-implementation.md](implementation.md) -- Rust 実装詳細
- [system-database設計](../_common/database.md) -- 共通データベース設計パターン
- [tier-architecture](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャ・データベースアクセスルール
- [docker-compose設計](../../infrastructure/docker/docker-compose設計.md) -- ローカル開発用 PostgreSQL
- [可観測性設計](../../architecture/observability/可観測性設計.md) -- OpenTelemetry トレース ID 連携
