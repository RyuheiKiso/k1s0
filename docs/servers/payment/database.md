# service-payment-database設計

service Tier の決済管理データベース（payment-db）の設計を定義する。
配置先: `regions/service/payment/database/postgres/`

## 概要

payment-db は service Tier に属する PostgreSQL 17 データベースであり、決済データを管理する。Outbox Pattern によるイベント配信の信頼性を確保するための outbox_events テーブルも備える。

[tier-architecture.md](../../architecture/overview/tier-architecture.md) の設計原則に従い、payment-db へのアクセスは **payment-server からのみ** 許可する。他サービスが決済情報を必要とする場合は、payment-server が提供する REST/gRPC API を経由する。

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
| payments（独立） | -- | 決済のメインテーブル |
| outbox_events（独立） | -- | イベント配信用の独立テーブル。payments との直接 FK なし |

---

## テーブル定義

### payments テーブル

決済を管理するメインテーブル。ステータスはステートマシンに従って遷移する。楽観的ロック用の version カラムを持ち、同時更新を検知する。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK | 決済識別子 |
| order_id | TEXT | NOT NULL | 注文 ID |
| customer_id | TEXT | NOT NULL | 顧客 ID |
| amount | BIGINT | NOT NULL | 決済金額（最小通貨単位） |
| currency | TEXT | NOT NULL DEFAULT 'JPY' | 通貨コード |
| status | TEXT | NOT NULL DEFAULT 'initiated' | 決済ステータス（initiated/completed/failed/refunded） |
| payment_method | TEXT | | 決済方法 |
| transaction_id | TEXT | | 外部決済プロバイダのトランザクション ID |
| error_code | TEXT | | エラーコード（失敗時） |
| error_message | TEXT | | エラーメッセージ（失敗時） |
| version | INT | NOT NULL DEFAULT 1 | 楽観的ロック用バージョン番号 |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時 |

### outbox_events テーブル

Outbox Pattern によるイベント配信テーブル。決済ライフサイクルイベントを記録し、非同期で Kafka に配信する。`published_at` が NULL のレコードが未配信イベント。

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | UUID | PK | イベント識別子 |
| aggregate_type | TEXT | NOT NULL | 集約タイプ（例: `payment`） |
| aggregate_id | TEXT | NOT NULL | 集約 ID（決済 UUID） |
| event_type | TEXT | NOT NULL | イベントタイプ（例: `payment.initiated`, `payment.completed`） |
| payload | JSONB | NOT NULL | イベントペイロード |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | イベント作成日時 |
| published_at | TIMESTAMPTZ | | Kafka 配信完了日時（NULL = 未配信） |

---

## インデックス設計

### インデックス一覧

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| payments | idx_payments_order | order_id | B-tree | 注文 ID による決済検索 |
| payments | idx_payments_customer | customer_id | B-tree | 顧客 ID による決済検索 |
| payments | idx_payments_status | status | B-tree | ステータスによるフィルタリング |
| payments | idx_payments_created_at | created_at DESC | B-tree | 作成日時による降順ソート |
| outbox_events | idx_outbox_unpublished | created_at (WHERE published_at IS NULL) | B-tree (部分) | 未配信イベントの効率的な取得 |

### 設計方針

- **楽観的ロック**: `payments.version` カラムで同時更新を検知する。更新時に `WHERE version = $expected` を条件にし、不一致時はエラーを返す
- **部分インデックス**: `outbox_events` の未配信イベント検索は部分インデックスで高速化する
- **ステータス遷移**: `initiated → completed/failed`、`completed → refunded` のみ許可。アプリケーション層で検証する

---

## データフロー

```
┌──────────────┐   REST/gRPC  ┌───────────────┐    SQL     ┌──────────────┐
│  クライアント │────────────>│ payment-server │─────────>│   payment-db │
│  (order /    │             │               │          │              │
│   checkout)  │             │  (決済開始)     │          │ payments     │
│              │             │  (決済完了)     │          │ outbox_events│
│              │             │  (返金)        │          │              │
└──────────────┘             └───────────────┘          └──────────────┘
                                    │
                                    │ Kafka (rdkafka)
                                    ▼
                  ┌──────────────────────────────────────┐
                  │ k1s0.service.payment.initiated.v1    │
                  │ k1s0.service.payment.completed.v1    │
                  │ k1s0.service.payment.failed.v1       │
                  │ k1s0.service.payment.refunded.v1     │
                  └──────────────────────────────────────┘
```

### フロー詳細

1. **決済開始**: 注文確定後に `POST /api/v1/payments` で決済レコードを作成。初期ステータスは `initiated`
2. **決済完了**: 外部プロバイダからの確認後に `POST /api/v1/payments/:id/complete` で `transaction_id` を記録
3. **決済失敗**: エラー発生時に `POST /api/v1/payments/:id/fail` で `error_code` / `error_message` を記録
4. **返金**: 完了済み決済に対して `POST /api/v1/payments/:id/refund` でステータスを `refunded` に遷移
5. **Outbox Pattern**: 各ステータス遷移時に `outbox_events` テーブルにイベントを記録し、バックグラウンドで Kafka に配信（at-least-once delivery）

---

## 主要クエリパターン

### 決済開始

```sql
INSERT INTO payments (id, order_id, customer_id, amount, currency, status,
                      payment_method, version, created_at, updated_at)
VALUES ($1, $2, $3, $4, $5, 'initiated', $6, 1, NOW(), NOW())
RETURNING *;
```

### 決済完了（楽観的ロック）

```sql
UPDATE payments
SET status = 'completed', transaction_id = $2,
    version = version + 1, updated_at = NOW()
WHERE id = $1 AND version = $3
RETURNING *;
```

### 決済失敗（楽観的ロック）

```sql
UPDATE payments
SET status = 'failed', error_code = $2, error_message = $3,
    version = version + 1, updated_at = NOW()
WHERE id = $1 AND version = $4
RETURNING *;
```

### 決済返金（楽観的ロック）

```sql
UPDATE payments
SET status = 'refunded', version = version + 1, updated_at = NOW()
WHERE id = $1 AND version = $2
RETURNING *;
```

### 決済一覧取得（フィルタ付き）

```sql
SELECT * FROM payments
WHERE ($1::text IS NULL OR order_id = $1)
  AND ($2::text IS NULL OR customer_id = $2)
  AND ($3::text IS NULL OR status = $3)
ORDER BY created_at DESC
LIMIT $4 OFFSET $5;

SELECT COUNT(*) FROM payments
WHERE ($1::text IS NULL OR order_id = $1)
  AND ($2::text IS NULL OR customer_id = $2)
  AND ($3::text IS NULL OR status = $3);
```

---

## マイグレーションファイル

配置先: `regions/service/payment/database/postgres/migrations/`

```
migrations/
├── 001_create_payments.up.sql           # スキーマ・payments テーブル・インデックス
├── 001_create_payments.down.sql
├── 002_create_outbox.up.sql             # outbox_events テーブル・部分インデックス
└── 002_create_outbox.down.sql
```

### マイグレーション一覧

| 番号 | ファイル名 | 説明 |
|------|-----------|------|
| 001 | create_payments | `payment_service` スキーマ・`payments` テーブル・order_id / customer_id / status / created_at インデックス |
| 002 | create_outbox | `outbox_events` テーブル・未配信イベント用部分インデックス |

### 001_create_payments.up.sql

```sql
CREATE SCHEMA IF NOT EXISTS payment_service;
SET search_path TO payment_service;

CREATE TABLE IF NOT EXISTS payments (
    id              UUID         PRIMARY KEY,
    order_id        TEXT         NOT NULL,
    customer_id     TEXT         NOT NULL,
    amount          BIGINT       NOT NULL,
    currency        TEXT         NOT NULL DEFAULT 'JPY',
    status          TEXT         NOT NULL DEFAULT 'initiated',
    payment_method  TEXT,
    transaction_id  TEXT,
    error_code      TEXT,
    error_message   TEXT,
    version         INT          NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payments_order ON payments (order_id);
CREATE INDEX idx_payments_customer ON payments (customer_id);
CREATE INDEX idx_payments_status ON payments (status);
CREATE INDEX idx_payments_created_at ON payments (created_at DESC);
```

### 002_create_outbox.up.sql

```sql
SET search_path TO payment_service;

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

### config.yaml（payment サーバー用）

```yaml
database:
  host: "postgres.k1s0-service.svc.cluster.local"
  port: 5432
  name: "k1s0_payment"
  schema: "payment_service"
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
| 静的パスワード | `secret/data/k1s0/service/payment-server/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/payment-server-rw` | TTL: 24時間 |
| 動的クレデンシャル（読み取り専用） | `database/creds/payment-server-ro` | TTL: 24時間 |

---

## 関連ドキュメント

- [service-payment-server.md](server.md) -- Payment サーバー設計（API・アーキテクチャ）
- [service-payment-implementation.md](implementation.md) -- Rust 実装詳細
- [system-database設計](../_common/database.md) -- 共通データベース設計パターン
- [tier-architecture](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャ・データベースアクセスルール
