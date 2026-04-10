# service-task-database 設計

service-task-server のデータベーススキーマ・マイグレーション・ER 図を定義する。
配置先: `regions/service/task/database/postgres/`

## 概要

task-db は service Tier に属する PostgreSQL 17 データベースであり、タスクデータとチェックリストを管理する。Outbox pattern によるイベント配信の信頼性を確保するための outbox_events テーブルも備える。

[tier-architecture.md](../../architecture/overview/tier-architecture.md) の設計原則に従い、task-db へのアクセスは **task-server からのみ** 許可する。

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
| スキーマ | `task_service` |
| RLS | 有効（`tenant_id` ベース） |
| マイグレーションパス | `regions/service/task/database/postgres/migrations/` |

---

## ER 図

```
tasks 1──* task_checklist_items
outbox_events（独立、tasks との直接 FK なし）
```

### リレーション

| 関係 | カーディナリティ | 説明 |
|------|-----------------|------|
| tasks - task_checklist_items | 1:N | 1 件のタスクは複数のチェックリストアイテムを持つ。タスク削除時に CASCADE 削除される |
| outbox_events（独立） | -- | イベント配信用の独立テーブル。tasks との直接 FK なし |

---

## テーブル定義

### tasks

タスクの基本情報を管理するメインテーブル。ステータスはステートマシンに従って遷移する。楽観的ロック用の `version` カラムを持ち、同時更新を検知する。

| カラム | 型 | 制約 | 説明 |
|--------|------|-------------|-------------|
| id | UUID | PK | タスク識別子（アプリケーション側で UUID を生成して挿入） |
| project_id | UUID | NOT NULL | プロジェクト ID（migration 008 で TEXT → UUID に変更済み）<!-- H-22 監査対応: 型を UUID に修正 --> |
| title | TEXT | NOT NULL | タスクタイトル |
| description | TEXT | | タスク説明 |
| status | TEXT | NOT NULL DEFAULT 'open' | タスクステータス（open/in_progress/review/done/cancelled） |
| priority | TEXT | NOT NULL DEFAULT 'medium' | 優先度（low/medium/high/critical） |
| assignee_id | TEXT | | 担当者 ID |
| reporter_id | TEXT | NOT NULL | 報告者 ID |
| due_date | TIMESTAMPTZ | | 期限日時 |
| labels | JSONB | NOT NULL DEFAULT '[]' | ラベル配列 |
| tenant_id | TEXT | NOT NULL DEFAULT 'system' | テナント ID（RLS 用） |
| created_by | TEXT | NOT NULL | 作成者 |
| updated_by | VARCHAR(255) | | 最終更新者 |
| version | INT | NOT NULL DEFAULT 1 | 楽観的ロック用バージョン番号 |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時 |

<!-- DOCS-HIGH-004 対応: migration の実装に合わせて id カラムの DEFAULT を削除（2026-04-03） -->
<!-- 実装では UUID はアプリケーション（Rust）側で生成し DB に挿入するため DEFAULT gen_random_uuid() は不要 -->
```sql
CREATE TABLE task_service.tasks (
    id           UUID         PRIMARY KEY,
    project_id   UUID         NOT NULL,  -- H-22 監査対応: migration 008 で TEXT → UUID に変更済み
    title        TEXT         NOT NULL,
    description  TEXT,
    status       TEXT         NOT NULL DEFAULT 'open',
    priority     TEXT         NOT NULL DEFAULT 'medium',
    assignee_id  TEXT,
    reporter_id  TEXT         NOT NULL,
    due_date     TIMESTAMPTZ,
    labels       JSONB        NOT NULL DEFAULT '[]',
    tenant_id    TEXT         NOT NULL DEFAULT 'system',
    created_by   TEXT         NOT NULL,
    updated_by   VARCHAR(255),
    version      INT          NOT NULL DEFAULT 1,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tasks_project_id ON task_service.tasks (project_id);
CREATE INDEX idx_tasks_status ON task_service.tasks (status);
CREATE INDEX idx_tasks_assignee_id ON task_service.tasks (assignee_id);
CREATE INDEX idx_tasks_created_at ON task_service.tasks (created_at DESC);
CREATE INDEX idx_tasks_tenant_id ON task_service.tasks (tenant_id);
CREATE INDEX idx_tasks_tenant_project ON task_service.tasks (tenant_id, project_id);
```

---

### task_checklist_items

タスク内のサブタスク/チェック項目を管理する。

| カラム | 型 | 制約 | 説明 |
|--------|------|-------------|-------------|
| id | UUID | PK | チェックリストアイテム識別子 |
| task_id | UUID | FK tasks(id) ON DELETE CASCADE, NOT NULL | 親タスク ID |
| title | TEXT | NOT NULL | アイテムタイトル |
| is_completed | BOOLEAN | NOT NULL DEFAULT FALSE | 完了フラグ |
| sort_order | INTEGER | NOT NULL DEFAULT 0 | 表示順 |
| tenant_id | TEXT | NOT NULL DEFAULT 'system' | テナント ID |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | 更新日時 |

<!-- DOCS-HIGH-004 対応: migration の実装に合わせて id カラムの DEFAULT を削除（2026-04-03） -->
```sql
CREATE TABLE task_service.task_checklist_items (
    id           UUID         PRIMARY KEY,
    task_id      UUID         NOT NULL REFERENCES task_service.tasks(id) ON DELETE CASCADE,
    title        TEXT         NOT NULL,
    is_completed BOOLEAN      NOT NULL DEFAULT FALSE,
    sort_order   INTEGER      NOT NULL DEFAULT 0,
    tenant_id    TEXT         NOT NULL DEFAULT 'system',
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_task_checklist_items_task_id ON task_service.task_checklist_items (task_id);
CREATE INDEX idx_task_checklist_items_tenant ON task_service.task_checklist_items (tenant_id);
```

---

### outbox_events

Outbox pattern によるイベント配信テーブル。タスクの作成・更新・キャンセル時にイベントを記録し、非同期で Kafka に配信する。`published_at` が NULL のレコードが未配信イベント。

| カラム | 型 | 制約 | 説明 |
|--------|------|-------------|-------------|
| id | UUID | PK | イベント識別子 |
| aggregate_type | TEXT | NOT NULL | 集約タイプ（`task`） |
| aggregate_id | TEXT | NOT NULL | 集約 ID（task UUID） |
| event_type | TEXT | NOT NULL | イベントタイプ（`task.created`, `task.updated`, `task.cancelled`） |
| payload | JSONB | NOT NULL | イベントペイロード |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT NOW() | イベント作成日時 |
| published_at | TIMESTAMPTZ | | Kafka 配信完了日時（NULL = 未配信） |

```sql
CREATE TABLE task_service.outbox_events (
    id             UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type TEXT         NOT NULL,
    aggregate_id   TEXT         NOT NULL,
    event_type     TEXT         NOT NULL,
    payload        JSONB        NOT NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    published_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_task_outbox_unpublished
    ON task_service.outbox_events (created_at)
    WHERE published_at IS NULL;
```

---

## インデックス設計

| テーブル | インデックス名 | カラム | 種別 | 用途 |
|----------|---------------|--------|------|------|
| tasks | idx_tasks_project_id | project_id | B-tree | プロジェクト ID による検索 |
| tasks | idx_tasks_status | status | B-tree | ステータスによるフィルタリング |
| tasks | idx_tasks_assignee_id | assignee_id | B-tree | 担当者 ID による検索 |
| tasks | idx_tasks_created_at | created_at DESC | B-tree | 作成日時による降順ソート |
| tasks | idx_tasks_tenant_project | (tenant_id, project_id) | B-tree | テナント+プロジェクト複合検索 |
| task_checklist_items | idx_task_checklist_items_task_id | task_id | B-tree | タスク ID による明細取得 |
| outbox_events | idx_task_outbox_unpublished | created_at (WHERE published_at IS NULL) | B-tree (部分) | 未配信イベントの効率的な取得 |

### 設計方針

- **CASCADE 削除**: `task_checklist_items` は `tasks` の削除に連動して自動削除される
- **部分インデックス**: `outbox_events` の未配信イベント検索は部分インデックスで高速化する
- **楽観的ロック**: `tasks.version` カラムで同時更新を検知する

---

## Row Level Security (RLS)

`tasks` および `task_checklist_items` テーブルに RLS を設定し、`app.current_tenant_id` セッション変数でテナントを分離する。

```sql
ALTER TABLE task_service.tasks ENABLE ROW LEVEL SECURITY;
ALTER TABLE task_service.task_checklist_items ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON task_service.tasks
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

CREATE POLICY tenant_isolation ON task_service.task_checklist_items
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);
```

---

## マイグレーションファイル

配置先: `regions/service/task/database/postgres/migrations/`

```
migrations/
├── 001_create_tasks.up.sql                       # task_service スキーマ・tasks テーブル・インデックス
├── 001_create_tasks.down.sql
├── 002_create_task_checklist_items.up.sql         # task_checklist_items テーブル・FK・インデックス
├── 002_create_task_checklist_items.down.sql
├── 003_add_updated_by_and_version.up.sql          # updated_by・version 列追加
├── 003_add_updated_by_and_version.down.sql
├── 004_create_outbox.up.sql                       # outbox_events テーブル・部分インデックス
├── 004_create_outbox.down.sql
├── 005_add_tenant_id_and_rls.up.sql               # tenant_id カラム追加・RLS 設定
├── 005_add_tenant_id_and_rls.down.sql
├── 008_alter_project_id_to_uuid.up.sql            # project_id カラム TEXT → UUID 型変更（H-22 監査対応）
└── 008_alter_project_id_to_uuid.down.sql
```

### マイグレーション一覧

| 番号 | ファイル名 | 説明 |
|------|-----------|------|
| 001 | create_tasks | `task_service` スキーマ・`tasks` テーブル・インデックス |
| 002 | create_task_checklist_items | `task_checklist_items` テーブル・FK（CASCADE）・インデックス |
| 003 | add_updated_by_and_version | `tasks` テーブルに `updated_by` と `version` 列を追加 |
| 004 | create_outbox | `outbox_events` テーブル・未配信イベント用部分インデックス |
| 005 | add_tenant_id_and_rls | `tenant_id` カラム追加・RLS ポリシー設定 |
| 006 | add_check_constraints | CHECK 制約追加 |
| 007 | fix_updated_by_type | `updated_by` 型修正 |
| 008 | alter_project_id_to_uuid | `tasks.project_id` を TEXT → UUID 型に変更（H-22 監査対応） |
| 009 | add_outbox_rls | `outbox_events` に `tenant_id TEXT NOT NULL` と RLS ポリシー追加（HIGH-BIZ-005 対応） |
| 010 | fix_status_check_constraint | `tasks` ステータス CHECK 制約に 'review' を追加（C-005 監査対応） |
| 011 | fix_rls_force_restrictive | `tasks` / `task_checklist_items` に FORCE ROW LEVEL SECURITY、AS RESTRICTIVE、WITH CHECK を追加（HIGH-BIZ-005 対応） |

---

## マルチテナント対応（HIGH-BIZ-005）

`tasks` / `task_checklist_items` に FORCE / AS RESTRICTIVE / WITH CHECK を追加（migration 011）。

```sql
ALTER TABLE task_service.tasks FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON task_service.tasks
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));
```

### 004_create_outbox.up.sql

```sql
SET search_path TO task_service;

CREATE TABLE IF NOT EXISTS outbox_events (
    id             UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type TEXT         NOT NULL,
    aggregate_id   TEXT         NOT NULL,
    event_type     TEXT         NOT NULL,
    payload        JSONB        NOT NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    published_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_task_outbox_unpublished
    ON outbox_events (created_at)
    WHERE published_at IS NULL;
```

---

## Advisory Lock

マルチインスタンス環境での安全なマイグレーション実行のため、Advisory Lock を使用する。

| 設定 | 値 |
| --- | --- |
| Advisory Lock ID | `1000000010` |

---

## DB 初期化スクリプト

Docker Compose による開発環境起動時に `infra/docker/init-db/15-task-schema.sql` を実行してスキーマを初期化する。

---

## 接続設定

```yaml
database:
  host: "postgres.k1s0-service.svc.cluster.local"
  port: 5432
  name: "k1s0_service"
  schema: "task_service"
  user: "app"
  password: ""
  ssl_mode: "require"
  max_connections: 25
```

### Vault によるクレデンシャル管理

| 用途 | Vault パス | 説明 |
|------|-----------|------|
| 静的パスワード | `secret/data/k1s0/service/task-server/database` | キー: `password` |
| 動的クレデンシャル（読み書き） | `database/creds/task-server-rw` | TTL: 24時間 |

---

## 関連ドキュメント

- [server.md](server.md) -- Task サーバー設計（API・アーキテクチャ）
- [implementation.md](implementation.md) -- Rust 実装詳細
- [system-database設計](../_common/database.md) -- 共通データベース設計パターン
- [tier-architecture](../../architecture/overview/tier-architecture.md) -- Tier アーキテクチャ・データベースアクセスルール
