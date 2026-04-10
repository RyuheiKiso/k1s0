# service-board-database 設計

service-board-server のデータベーススキーマ・マイグレーション・ER 図を定義する。
配置先: `regions/service/board/database/postgres/`

## 概要

board-db は service Tier に属する PostgreSQL 17 データベースであり、Kanban ボードのカラム情報（タスク数・WIP制限）を管理する。Outbox pattern によるイベント配信の信頼性を確保するための outbox_events テーブルも備える。

[tier-architecture.md](../../architecture/overview/tier-architecture.md) の設計原則に従い、board-db へのアクセスは **board-server からのみ** 許可する。

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
| スキーマ | `board_service` |
| マイグレーションパス | `regions/service/board/database/postgres/migrations/` |

---

## ER 図

```
board_columns（独立）
outbox_events（独立、board_columns との直接 FK なし）
```

### リレーション

| 関係 | カーディナリティ | 説明 |
|------|-----------------|------|
| outbox_events（独立） | -- | イベント配信用の独立テーブル。board_columns との直接 FK なし |

---

## テーブル定義

### board_columns

Kanban ボードのカラム情報を管理する。`project_id` + `status_code` の組み合わせに対して UNIQUE 制約を設ける。`wip_limit = 0` は制限なし（無制限）を表す。`task_count` は 0 以上の CHECK 制約で保護する。楽観的ロック用の `version` カラムを持ち、同時更新を検知する。

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | カラムの一意識別子（アプリケーション側で UUID を生成して挿入） |
| project_id | TEXT | NOT NULL, UNIQUE(project_id, status_code) | プロジェクト ID |
| status_code | TEXT | NOT NULL, UNIQUE(project_id, status_code) | ステータスコード（project-master の status_definitions.code と対応） |
| task_count | INTEGER | NOT NULL, DEFAULT 0, CHECK >= 0 | 現在のタスク数 |
| wip_limit | INTEGER | NOT NULL, DEFAULT 0, CHECK >= 0 | WIP 制限（0 = 無制限） |
| version | INTEGER | NOT NULL, DEFAULT 1 | 楽観的ロック用バージョン番号 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

<!-- DOCS-HIGH-003 対応: migration の実装に合わせて id カラムの DEFAULT を削除（2026-04-03） -->
<!-- 実装では UUID はアプリケーション（Rust）側で生成し DB に挿入するため DEFAULT gen_random_uuid() は不要 -->
<!-- wip_limit の CHECK 制約は実装に含まれていないため設計書からも削除する -->
```sql
CREATE TABLE board_service.board_columns (
    id           UUID         PRIMARY KEY,
    project_id   TEXT         NOT NULL,
    status_code  TEXT         NOT NULL,
    task_count   INTEGER      NOT NULL DEFAULT 0 CHECK (task_count >= 0),
    wip_limit    INTEGER      NOT NULL DEFAULT 0,
    version      INTEGER      NOT NULL DEFAULT 1,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_board_columns_project_status UNIQUE (project_id, status_code)
);

CREATE INDEX IF NOT EXISTS idx_board_columns_project_id ON board_service.board_columns (project_id);
CREATE INDEX IF NOT EXISTS idx_board_columns_project_status ON board_service.board_columns (project_id, status_code);
```

---

### outbox_events

Outbox pattern によるイベント配信テーブル。カラムの更新（increment/decrement/wip-limit変更）時にイベントを記録し、非同期で Kafka に配信する。`published_at` が NULL のレコードが未配信イベント。

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | イベント識別子 |
| aggregate_type | TEXT | NOT NULL | 集約タイプ（`board_column`） |
| aggregate_id | TEXT | NOT NULL | 集約 ID（`{project_id}:{status_code}`） |
| event_type | TEXT | NOT NULL | イベントタイプ（`board.column_updated`） |
| payload | JSONB | NOT NULL | イベントペイロード |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | イベント作成日時 |
| published_at | TIMESTAMPTZ | | Kafka 配信完了日時（NULL = 未配信） |

```sql
CREATE TABLE board_service.outbox_events (
    id             UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type TEXT         NOT NULL,
    aggregate_id   TEXT         NOT NULL,
    event_type     TEXT         NOT NULL,
    payload        JSONB        NOT NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    published_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_board_outbox_unpublished
    ON board_service.outbox_events (created_at)
    WHERE published_at IS NULL;
```

---

## インデックス設計

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| board_columns | uq_board_columns_project_status | (project_id, status_code) | UNIQUE | カラム一意性保証・高速検索 |
| board_columns | idx_board_columns_project_id | project_id | B-tree | プロジェクト ID による全カラム取得 |
| outbox_events | idx_board_outbox_unpublished | created_at (WHERE published_at IS NULL) | B-tree (部分) | 未配信イベントの効率的な取得 |

### 設計方針

- **CHECK 制約**: `task_count >= 0` と `wip_limit >= 0` により、データ整合性を DB レベルで保証する
- **UNIQUE 制約**: `(project_id, status_code)` の複合 UNIQUE 制約で、1プロジェクト×1ステータスに対して1カラムのみ存在を保証する
- **楽観的ロック**: `board_columns.version` カラムで同時更新を検知する。更新時に `WHERE version = $expected` を条件にし、不一致時はエラーを返す
- **部分インデックス**: `outbox_events` の未配信イベント検索は部分インデックスで高速化する

---

## 主要クエリパターン

### カラム取得

```sql
-- project_id + status_code で取得
SELECT id, project_id, status_code, task_count, wip_limit, version, created_at, updated_at
FROM board_service.board_columns
WHERE project_id = $1 AND status_code = $2;

-- プロジェクトの全カラム取得
SELECT id, project_id, status_code, task_count, wip_limit, version, created_at, updated_at
FROM board_service.board_columns
WHERE project_id = $1
ORDER BY status_code ASC;
```

### タスク数増加（楽観的ロック）

```sql
-- アプリケーション層で WIP 制限チェック後に実行
UPDATE board_service.board_columns
SET task_count = task_count + 1, version = version + 1, updated_at = NOW()
WHERE project_id = $1 AND status_code = $2 AND version = $3
RETURNING *;
```

### タスク数減少（下限 0 保証）

```sql
UPDATE board_service.board_columns
SET task_count = GREATEST(task_count - 1, 0), version = version + 1, updated_at = NOW()
WHERE project_id = $1 AND status_code = $2 AND version = $3
RETURNING *;
```

### カラム upsert

```sql
INSERT INTO board_service.board_columns (id, project_id, status_code, task_count, wip_limit, version)
VALUES ($1, $2, $3, 0, 0, 1)
ON CONFLICT (project_id, status_code) DO UPDATE
SET wip_limit = EXCLUDED.wip_limit, version = board_columns.version + 1, updated_at = NOW()
RETURNING *;
```

---

## マイグレーションファイル

配置先: `regions/service/board/database/postgres/migrations/`

```
migrations/
├── 001_create_board_columns.up.sql     # board_service スキーマ・board_columns テーブル・インデックス・制約
├── 001_create_board_columns.down.sql
├── 002_create_outbox.up.sql            # outbox_events テーブル・部分インデックス
└── 002_create_outbox.down.sql
```

### マイグレーション一覧

| 番号 | ファイル名 | 説明 |
|------|-----------|------|
| 001 | create_board_columns | `board_service` スキーマ・`board_columns` テーブル・UNIQUE 制約・CHECK 制約・インデックス |
| 002 | create_outbox | `outbox_events` テーブル・未配信イベント用部分インデックス |
| 003 | add_tenant_id_and_rls | `board_columns` に `tenant_id TEXT NOT NULL` と RLS ポリシー追加 |
| 004 | add_outbox_rls | `outbox_events` に `tenant_id TEXT NOT NULL` と RLS ポリシー追加（HIGH-BIZ-002 対応） |
| 005 | project_id_uuid | `project_id` を UUID 型に変更 |
| 006 | fix_rls_force_restrictive | `board_columns` に FORCE ROW LEVEL SECURITY、AS RESTRICTIVE、WITH CHECK を追加（HIGH-BIZ-002 対応） |

---

## マルチテナント対応（HIGH-BIZ-002）

`board_columns` に FORCE / AS RESTRICTIVE / WITH CHECK を追加（migration 006）。

```sql
ALTER TABLE board_service.board_columns FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON board_service.board_columns
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));
```

### 001_create_board_columns.up.sql

```sql
CREATE SCHEMA IF NOT EXISTS board_service;

SET search_path TO board_service;

CREATE TABLE IF NOT EXISTS board_columns (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id   TEXT         NOT NULL,
    status_code  TEXT         NOT NULL,
    task_count   INTEGER      NOT NULL DEFAULT 0 CHECK (task_count >= 0),
    wip_limit    INTEGER      NOT NULL DEFAULT 0 CHECK (wip_limit >= 0),
    version      INTEGER      NOT NULL DEFAULT 1,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_board_columns_project_status UNIQUE (project_id, status_code)
);

CREATE INDEX IF NOT EXISTS idx_board_columns_project_id ON board_columns (project_id);
CREATE INDEX IF NOT EXISTS idx_board_columns_project_status ON board_columns (project_id, status_code);
```

### 002_create_outbox.up.sql

```sql
SET search_path TO board_service;

CREATE TABLE IF NOT EXISTS outbox_events (
    id             UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type TEXT         NOT NULL,
    aggregate_id   TEXT         NOT NULL,
    event_type     TEXT         NOT NULL,
    payload        JSONB        NOT NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    published_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_board_outbox_unpublished
    ON outbox_events (created_at)
    WHERE published_at IS NULL;
```

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION board_service.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_board_columns_updated_at
    BEFORE UPDATE ON board_service.board_columns
    FOR EACH ROW EXECUTE FUNCTION board_service.update_updated_at();
```

---

## Advisory Lock

マルチインスタンス環境での安全なマイグレーション実行のため、Advisory Lock を使用する。

| 設定 | 値 |
| --- | --- |
| Advisory Lock ID | `1000000011` |

---

## DB 初期化スクリプト

Docker Compose による開発環境起動時に `infra/docker/init-db/14-board-schema.sql` を実行してスキーマを初期化する。

SQLx マイグレーションが `_sqlx_migrations` テーブルをスキーマ内に作成できるよう、`GRANT CREATE ON DATABASE k1s0_service TO k1s0;` を付与している（CRIT-03 対応）。

---

## 接続設定

```yaml
database:
  host: "postgres.k1s0-service.svc.cluster.local"
  port: 5432
  name: "k1s0_service"
  schema: "board_service"
  user: "app"
  password: ""
  ssl_mode: "require"
  max_connections: 25
```

### Vault によるクレデンシャル管理

| 用途 | Vault パス | 説明 |
|------|-----------|------|
| 静的パスワード | `secret/data/k1s0/service/board-server/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/board-server-rw` | TTL: 24時間 |

---

## 関連ドキュメント

- [server.md](server.md) -- Board サーバー設計（API・アーキテクチャ）
- [implementation.md](implementation.md) -- Rust 実装詳細
- [system-database設計](../_common/database.md) -- 共通データベース設計パターン
- [tier-architecture](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャ・データベースアクセスルール
