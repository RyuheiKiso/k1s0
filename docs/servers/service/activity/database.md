# service-activity-database 設計

service-activity-server のデータベーススキーマ・マイグレーション・ER 図を定義する。
配置先: `regions/service/activity/database/postgres/`

## 概要

activity-db は service Tier に属する PostgreSQL 17 データベースであり、タスクへのコメント・作業時間・ステータス変更等の操作履歴を管理する。冪等性保証のための `idempotency_key` UNIQUE 制約と Outbox pattern によるイベント配信テーブルを備える。

[tier-architecture.md](../../architecture/overview/tier-architecture.md) の設計原則に従い、activity-db へのアクセスは **activity-server からのみ** 許可する。

### 技術スタック

| コンポーネント | 技術 | バージョン |
|----------------|------|------------|
| RDBMS | PostgreSQL | 17 |
| マイグレーション | sqlx-cli（`sqlx::migrate!` マクロ） | - |
| ORM / クエリビルダー | sqlx（Rust） | 0.8 |
| シークレット管理 | HashiCorp Vault | 1.17 |

| 項目 | 値 |
| --- | --- |
| データベース | `k1s0_service` |
| スキーマ | `activity_service` |
| マイグレーションパス | `regions/service/activity/database/postgres/migrations/` |

---

## ER 図

```
activities（独立）
outbox_events（独立、activities との直接 FK なし）
```

### リレーション

| 関係 | カーディナリティ | 説明 |
|------|-----------------|------|
| outbox_events（独立） | -- | イベント配信用の独立テーブル。activities との直接 FK なし |

---

## テーブル定義

### activities

タスクへのコメント・作業時間・ステータス変更等の操作履歴を管理する。`idempotency_key` カラムに UNIQUE 制約を設定し、同一キーによる重複作成を防ぐ。楽観的ロック用の `version` カラムを持つ。

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | アクティビティの一意識別子 |
| task_id | TEXT | NOT NULL | 対象タスク ID |
| actor_id | TEXT | NOT NULL | 操作実行者 ID |
| activity_type | TEXT | NOT NULL | タイプ（`comment`, `time_entry`, `status_change`, `assignment`） |
| content | TEXT | | コメント内容等 |
| duration_minutes | INTEGER | | 作業時間（time_entry タイプ時に使用） |
| status | TEXT | NOT NULL, DEFAULT 'active' | ステータス（`active`, `submitted`, `approved`, `rejected`） |
| metadata | JSONB | | 追加メタデータ |
| idempotency_key | TEXT | UNIQUE | 冪等性キー（重複リクエスト排除用） |
| version | INTEGER | NOT NULL, DEFAULT 1 | 楽観的ロック用バージョン番号 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

```sql
CREATE TABLE activity_service.activities (
    id                UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id           TEXT         NOT NULL,
    actor_id          TEXT         NOT NULL,
    activity_type     TEXT         NOT NULL,
    content           TEXT,
    duration_minutes  INTEGER,
    status            TEXT         NOT NULL DEFAULT 'active',
    metadata          JSONB,
    idempotency_key   TEXT         UNIQUE,
    version           INTEGER      NOT NULL DEFAULT 1,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_activities_task_id ON activity_service.activities (task_id);
CREATE INDEX idx_activities_actor_id ON activity_service.activities (actor_id);
CREATE INDEX idx_activities_activity_type ON activity_service.activities (activity_type);
CREATE INDEX idx_activities_status ON activity_service.activities (status);
CREATE INDEX idx_activities_created_at ON activity_service.activities (created_at DESC);
```

---

### outbox_events

Outbox pattern によるイベント配信テーブル。アクティビティの作成・承認時にイベントを記録し、非同期で Kafka に配信する。`published_at` が NULL のレコードが未配信イベント。

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | イベント識別子 |
| aggregate_type | TEXT | NOT NULL | 集約タイプ（`activity`） |
| aggregate_id | TEXT | NOT NULL | 集約 ID（activity UUID） |
| event_type | TEXT | NOT NULL | イベントタイプ（`activity.created`, `activity.approved`） |
| payload | JSONB | NOT NULL | イベントペイロード |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | イベント作成日時 |
| published_at | TIMESTAMPTZ | | Kafka 配信完了日時（NULL = 未配信） |

```sql
CREATE TABLE activity_service.outbox_events (
    id             UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type TEXT         NOT NULL,
    aggregate_id   TEXT         NOT NULL,
    event_type     TEXT         NOT NULL,
    payload        JSONB        NOT NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    published_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_activity_outbox_unpublished
    ON activity_service.outbox_events (created_at)
    WHERE published_at IS NULL;
```

---

## インデックス設計

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| activities | (UNIQUE) idempotency_key | idempotency_key | UNIQUE B-tree | 冪等性保証・高速重複チェック |
| activities | idx_activities_task_id | task_id | B-tree | タスク ID による検索 |
| activities | idx_activities_actor_id | actor_id | B-tree | 操作者 ID による検索 |
| activities | idx_activities_activity_type | activity_type | B-tree | タイプによるフィルタリング |
| activities | idx_activities_status | status | B-tree | ステータスによるフィルタリング |
| activities | idx_activities_created_at | created_at DESC | B-tree | 作成日時による降順ソート |
| outbox_events | idx_activity_outbox_unpublished | created_at (WHERE published_at IS NULL) | B-tree (部分) | 未配信イベントの効率的な取得 |

### 設計方針

- **冪等性**: `idempotency_key` の UNIQUE 制約により、DB レベルで重複アクティビティの作成を防ぐ
- **楽観的ロック**: `activities.version` カラムで同時更新を検知する
- **部分インデックス**: `outbox_events` の未配信イベント検索は部分インデックスで高速化する

---

## 冪等性の仕組み

`idempotency_key` カラムに UNIQUE 制約を設定することで、DB レベルで重複アクティビティの作成を防ぐ。

```sql
-- 同一キーの INSERT は UNIQUE 制約違反となる
INSERT INTO activity_service.activities (id, idempotency_key, ...)
VALUES ($1, $2, ...)
ON CONFLICT (idempotency_key) DO NOTHING
RETURNING *;
```

アプリケーション層では、`CreateActivity` 時に `find_by_idempotency_key()` で事前チェックし、存在すれば既存レコードを返す（HTTP 200 + `SVC_ACTIVITY_DUPLICATE_IDEMPOTENCY_KEY` レスポンス）。

---

## 主要クエリパターン

### アクティビティ取得

```sql
-- ID で取得
SELECT id, task_id, actor_id, activity_type, content, duration_minutes,
       status, metadata, idempotency_key, version, created_at, updated_at
FROM activity_service.activities
WHERE id = $1;

-- 冪等性キーで取得
SELECT id, task_id, actor_id, activity_type, content, duration_minutes,
       status, metadata, idempotency_key, version, created_at, updated_at
FROM activity_service.activities
WHERE idempotency_key = $1;
```

### アクティビティ一覧取得（フィルタ付き）

```sql
SELECT id, task_id, actor_id, activity_type, content, duration_minutes,
       status, metadata, idempotency_key, version, created_at, updated_at
FROM activity_service.activities
WHERE ($1::text IS NULL OR task_id = $1)
  AND ($2::text IS NULL OR actor_id = $2)
  AND ($3::text IS NULL OR activity_type = $3)
  AND ($4::text IS NULL OR status = $4)
ORDER BY created_at DESC
LIMIT $5 OFFSET $6;
```

### ステータス更新（楽観的ロック）

```sql
UPDATE activity_service.activities
SET status = $2, version = version + 1, updated_at = NOW()
WHERE id = $1 AND version = $3
RETURNING *;
```

---

## マイグレーションファイル

配置先: `regions/service/activity/database/postgres/migrations/`

```
migrations/
├── 001_create_activities.up.sql        # activity_service スキーマ・activities テーブル・インデックス・制約
├── 001_create_activities.down.sql
├── 002_create_outbox.up.sql            # outbox_events テーブル・部分インデックス
└── 002_create_outbox.down.sql
```

### マイグレーション一覧

| 番号 | ファイル名 | 説明 |
|------|-----------|------|
| 001 | create_activities | `activity_service` スキーマ・`activities` テーブル・UNIQUE 制約（idempotency_key）・インデックス |
| 002 | create_outbox | `outbox_events` テーブル・未配信イベント用部分インデックス |

### 001_create_activities.up.sql

```sql
CREATE SCHEMA IF NOT EXISTS activity_service;

SET search_path TO activity_service;

CREATE TABLE IF NOT EXISTS activities (
    id                UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id           TEXT         NOT NULL,
    actor_id          TEXT         NOT NULL,
    activity_type     TEXT         NOT NULL,
    content           TEXT,
    duration_minutes  INTEGER,
    status            TEXT         NOT NULL DEFAULT 'active',
    metadata          JSONB,
    idempotency_key   TEXT         UNIQUE,
    version           INTEGER      NOT NULL DEFAULT 1,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_activities_task_id ON activities (task_id);
CREATE INDEX IF NOT EXISTS idx_activities_actor_id ON activities (actor_id);
CREATE INDEX IF NOT EXISTS idx_activities_activity_type ON activities (activity_type);
CREATE INDEX IF NOT EXISTS idx_activities_status ON activities (status);
CREATE INDEX IF NOT EXISTS idx_activities_created_at ON activities (created_at DESC);
```

### 002_create_outbox.up.sql

```sql
SET search_path TO activity_service;

CREATE TABLE IF NOT EXISTS outbox_events (
    id             UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type TEXT         NOT NULL,
    aggregate_id   TEXT         NOT NULL,
    event_type     TEXT         NOT NULL,
    payload        JSONB        NOT NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    published_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_activity_outbox_unpublished
    ON outbox_events (created_at)
    WHERE published_at IS NULL;
```

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION activity_service.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_activities_updated_at
    BEFORE UPDATE ON activity_service.activities
    FOR EACH ROW EXECUTE FUNCTION activity_service.update_updated_at();
```

---

## Advisory Lock

マルチインスタンス環境での安全なマイグレーション実行のため、Advisory Lock を使用する。

| 設定 | 値 |
| --- | --- |
| Advisory Lock ID | `1000000012` |

---

## DB 初期化スクリプト

Docker Compose による開発環境起動時に `infra/docker/init-db/17-activity-schema.sql` を実行してスキーマを初期化する。

---

## 接続設定

```yaml
database:
  host: "postgres.k1s0-service.svc.cluster.local"
  port: 5432
  name: "k1s0_service"
  schema: "activity_service"
  user: "app"
  password: ""
  ssl_mode: "require"
  max_connections: 25
```

### Vault によるクレデンシャル管理

| 用途 | Vault パス | 説明 |
|------|-----------|------|
| 静的パスワード | `secret/data/k1s0/service/activity-server/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/activity-server-rw` | TTL: 24時間 |

---

## 関連ドキュメント

- [server.md](server.md) -- Activity サーバー設計（API・アーキテクチャ）
- [implementation.md](implementation.md) -- Rust 実装詳細
- [system-database設計](../_common/database.md) -- 共通データベース設計パターン
- [tier-architecture](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャ・データベースアクセスルール
