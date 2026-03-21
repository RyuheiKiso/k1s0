# service-inventory-database設計

service Tier の在庫管理データベース（inventory-db）の設計を定義する。
配置先: `regions/service/inventory/database/postgres/`

## 概要

inventory-db は service Tier に属する PostgreSQL 17 データベースであり、在庫データを管理する。Outbox Pattern によるイベント配信の信頼性を確保するための outbox_events テーブルも備える。

[tier-architecture.md](../../architecture/overview/tier-architecture.md) の設計原則に従い、inventory-db へのアクセスは **inventory-server からのみ** 許可する。他サービスが在庫情報を必要とする場合は、inventory-server が提供する REST/gRPC API を経由する。

### 技術スタック

| コンポーネント | 技術 | バージョン |
|----------------|------|------------|
| RDBMS | PostgreSQL | 17 |
| マイグレーション | sqlx-cli（`sqlx::migrate!` マクロ） | - |
| ORM / クエリビルダー | sqlx（Rust） | 0.8 |
| シークレット管理 | HashiCorp Vault | 1.17 |

---

## ER図

### リレーション

| 関係 | カーディナリティ | 説明 |
|------|-----------------|------|
| inventory_items（独立） | -- | 在庫アイテムのメインテーブル。product_id + warehouse_id で一意 |
| outbox_events（独立） | -- | イベント配信用の独立テーブル。inventory_items との直接 FK なし |
| inventory_reservations → inventory_items | 多対一 | 在庫予約テーブル。order_id + inventory_item_id で一意。Saga 補償用 |

---

## テーブル定義

### inventory_items テーブル

商品の在庫を倉庫単位で管理するメインテーブル。楽観的ロック用の version カラムを持ち、同時更新を検知する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK | 在庫アイテム識別子 |
| product_id | TEXT | NOT NULL, UNIQUE(product_id, warehouse_id) | 商品 ID |
| warehouse_id | TEXT | NOT NULL, UNIQUE(product_id, warehouse_id) | 倉庫 ID |
| qty_available | INT | NOT NULL DEFAULT 0, CHECK (>= 0) | 利用可能数量 |
| qty_reserved | INT | NOT NULL DEFAULT 0, CHECK (>= 0) | 予約済み数量 |
| version | INT | NOT NULL DEFAULT 1 | 楽観的ロック用バージョン番号 |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時 |

### inventory_reservations テーブル

在庫予約の注文別追跡テーブル。`reserve_stock` 呼び出し時に同一トランザクション内で挿入される。
Saga 補償トランザクションで `order_id` から解放対象の予約を逆引きするために使用する。
`status` が `'reserved'` のレコードのみが補償対象となるため、解放済み・確定済みの二重処理を防止する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK | 予約識別子 |
| order_id | TEXT | NOT NULL | 注文 ID |
| inventory_item_id | UUID | NOT NULL, FK → inventory_items(id) | 在庫アイテム ID |
| product_id | TEXT | NOT NULL | 商品 ID（逆引き高速化のため非正規化） |
| warehouse_id | TEXT | NOT NULL | 倉庫 ID（逆引き高速化のため非正規化） |
| quantity | INT | NOT NULL, CHECK (> 0) | 予約数量 |
| status | TEXT | NOT NULL DEFAULT 'reserved', CHECK IN ('reserved','released','confirmed') | 予約ステータス |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時 |

### outbox_events テーブル

Outbox Pattern によるイベント配信テーブル。在庫の予約・解放時にイベントを記録し、非同期で Kafka に配信する。`published_at` が NULL のレコードが未配信イベント。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK | イベント識別子 |
| aggregate_type | TEXT | NOT NULL | 集約タイプ（例: `inventory`） |
| aggregate_id | TEXT | NOT NULL | 集約 ID（在庫 UUID） |
| event_type | TEXT | NOT NULL | イベントタイプ（例: `inventory.reserved`, `inventory.released`） |
| payload | JSONB | NOT NULL | イベントペイロード |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | イベント作成日時 |
| published_at | TIMESTAMPTZ | | Kafka 配信完了日時（NULL = 未配信） |

---

## インデックス設計

### インデックス一覧

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| inventory_items | uq_product_warehouse | (product_id, warehouse_id) | UNIQUE | 商品×倉庫の一意制約 |
| inventory_items | idx_inventory_product | product_id | B-tree | 商品 ID による在庫検索 |
| inventory_items | idx_inventory_warehouse | warehouse_id | B-tree | 倉庫 ID による在庫検索 |
| outbox_events | idx_outbox_unpublished | created_at (WHERE published_at IS NULL) | B-tree (部分) | 未配信イベントの効率的な取得 |
| inventory_reservations | idx_reservation_order_id_status | order_id (WHERE status='reserved') | B-tree (部分) | Saga 補償用の注文別予約逆引き |
| inventory_reservations | uq_reservation_order_item | (order_id, inventory_item_id) | UNIQUE | 同一注文×在庫アイテムの重複予約防止 |

### 設計方針

- **CHECK 制約**: `qty_available >= 0` と `qty_reserved >= 0` により、在庫数量の非負を DB レベルで保証する
- **UNIQUE 制約**: `(product_id, warehouse_id)` の一意制約により、同一商品×倉庫の重複を防止する
- **部分インデックス**: `outbox_events` の未配信イベント検索は部分インデックスで高速化する
- **楽観的ロック**: `version` カラムで同時更新を検知する。更新時に `WHERE version = $expected` を条件にし、不一致時はエラーを返す

---

## データフロー

```
┌──────────────┐   REST/gRPC  ┌─────────────────┐    SQL     ┌──────────────────┐
│  クライアント │────────────>│ inventory-server │─────────>│   inventory-db    │
│  (order /    │             │                 │          │                    │
│   fulfillment)│             │  (在庫予約)      │          │ inventory_items    │
│              │             │  (在庫解放)      │          │ outbox_events      │
└──────────────┘             │  (在庫照会)      │          └──────────────────┘
                              └─────────────────┘
                                      │
                                      │ Kafka (rdkafka)
                                      ▼
                  ┌─────────────────────────────────────┐
                  │ k1s0.service.inventory.reserved.v1  │
                  │ k1s0.service.inventory.released.v1  │
                  └─────────────────────────────────────┘
```

### フロー詳細

1. **在庫予約**: 注文サービスから `POST /api/v1/inventory/reserve` を呼び出し、トランザクション内で `qty_available` を減算、`qty_reserved` を加算
2. **在庫解放**: キャンセル時に `POST /api/v1/inventory/release` を呼び出し、`qty_reserved` を減算、`qty_available` を加算
3. **在庫更新**: `PUT /api/v1/inventory/{id}` で数量を直接更新。楽観的ロック（version）で同時更新を検知
4. **Outbox Pattern**: `outbox_events` テーブルに未配信イベントを記録し、バックグラウンドで Kafka に配信（at-least-once delivery）

---

## 主要クエリパターン

### 在庫予約（楽観的ロック + CHECK 制約）

```sql
UPDATE inventory_items
SET qty_available = qty_available - $2,
    qty_reserved = qty_reserved + $2,
    version = version + 1,
    updated_at = NOW()
WHERE product_id = $3 AND warehouse_id = $4
  AND qty_available >= $2
RETURNING *;
```

`qty_available >= $2` 条件で在庫不足を検知。0 行返却時はアプリケーション層で `SVC_INVENTORY_INSUFFICIENT_STOCK` として処理する。

### 在庫解放

```sql
UPDATE inventory_items
SET qty_available = qty_available + $2,
    qty_reserved = qty_reserved - $2,
    version = version + 1,
    updated_at = NOW()
WHERE product_id = $3 AND warehouse_id = $4
RETURNING *;
```

### 在庫一覧取得（フィルタ付き）

```sql
SELECT * FROM inventory_items
WHERE ($1::text IS NULL OR product_id = $1)
  AND ($2::text IS NULL OR warehouse_id = $2)
ORDER BY created_at DESC
LIMIT $3 OFFSET $4;
```

---

## マイグレーションファイル

配置先: `regions/service/inventory/database/postgres/migrations/`

```
migrations/
├── 001_create_inventory_items.up.sql    # スキーマ・inventory_items テーブル・インデックス・制約
├── 001_create_inventory_items.down.sql
├── 002_create_outbox.up.sql             # outbox_events テーブル・部分インデックス
└── 002_create_outbox.down.sql
```

### マイグレーション一覧

| 番号 | ファイル名 | 説明 |
|------|-----------|------|
| 001 | create_inventory_items | `inventory_service` スキーマ・`inventory_items` テーブル・UNIQUE 制約・CHECK 制約・product_id / warehouse_id インデックス |
| 002 | create_outbox | `outbox_events` テーブル・未配信イベント用部分インデックス |

### 001_create_inventory_items.up.sql

```sql
CREATE SCHEMA IF NOT EXISTS inventory_service;
SET search_path TO inventory_service;

CREATE TABLE IF NOT EXISTS inventory_items (
    id UUID PRIMARY KEY,
    product_id TEXT NOT NULL,
    warehouse_id TEXT NOT NULL,
    qty_available INT NOT NULL DEFAULT 0,
    qty_reserved INT NOT NULL DEFAULT 0,
    version INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_product_warehouse UNIQUE (product_id, warehouse_id),
    CONSTRAINT chk_qty_available CHECK (qty_available >= 0),
    CONSTRAINT chk_qty_reserved CHECK (qty_reserved >= 0)
);

CREATE INDEX idx_inventory_product ON inventory_items (product_id);
CREATE INDEX idx_inventory_warehouse ON inventory_items (warehouse_id);
```

### 002_create_outbox.up.sql

```sql
SET search_path TO inventory_service;

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

### config.yaml（inventory サーバー用）

```yaml
database:
  host: "postgres.k1s0-service.svc.cluster.local"
  port: 5432
  name: "k1s0_inventory"
  schema: "inventory_service"
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
| 静的パスワード | `secret/data/k1s0/service/inventory-server/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/inventory-server-rw` | TTL: 24時間 |
| 動的クレデンシャル（読み取り専用） | `database/creds/inventory-server-ro` | TTL: 24時間 |

---

## 関連ドキュメント

- [service-inventory-server.md](server.md) -- Inventory サーバー設計（API・アーキテクチャ）
- [service-inventory-implementation.md](implementation.md) -- Rust 実装詳細
- [system-database設計](../_common/database.md) -- 共通データベース設計パターン
- [tier-architecture](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャ・データベースアクセスルール
