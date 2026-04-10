# business-task-server データベース設計

> **注意（MEDIUM-004 対応）**: このサービスは `regions/service/` に実装されており、正式な設計書は `docs/servers/service/task/` にあります。
> このファイルは移行経過として残存していますが、上記の service tier 設計書を参照してください。
> ADR-0026「Service Tier DB 統合」を参照。

business Tier のタスクサービスデータベース（k1s0_service DB の `task_service` スキーマ）の設計を定義する。
配置先: `regions/service/task/database/postgres/`

> **注意**: board / task / activity の 3 サービスは同一 PostgreSQL データベース（k1s0_service）を共有し、
> スキーマで分離する設計となっている。テナント間の参照・操作リスクは RLS で防止する。

---

## スキーマ

スキーマ名: `task_service`

```sql
CREATE SCHEMA IF NOT EXISTS task_service;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| tasks | タスクの基本情報（タイトル・ステータス・優先度・担当者等） |
| task_checklist_items | タスク内のサブタスク / チェック項目 |
| outbox_events | タスク変更イベントの Outbox パターン送信キュー |

---

## ER 図

```
tasks 1──* task_checklist_items
outbox_events（単独テーブル、tasks との FK なし・Outbox パターン）
```

---

## テーブル定義

### tasks（タスク）

タスクの基本情報を管理する。`status` と `priority` は CHECK 制約で有効値を DB レベルで強制する。
migration 008 で `project_id` が TEXT から UUID 型に変更された（型整合性強化）。
`version` フィールドにより楽観的ロックを実現する。

```sql
CREATE TABLE IF NOT EXISTS task_service.tasks (
    id            UUID         PRIMARY KEY,
    project_id    UUID         NOT NULL,
    title         TEXT         NOT NULL,
    description   TEXT,
    status        TEXT         NOT NULL DEFAULT 'open',
    priority      TEXT         NOT NULL DEFAULT 'medium',
    assignee_id   TEXT,
    reporter_id   TEXT         NOT NULL,
    due_date      TIMESTAMPTZ,
    labels        JSONB        NOT NULL DEFAULT '[]',
    created_by    TEXT         NOT NULL,
    updated_by    TEXT         NOT NULL DEFAULT '',
    version       INT          NOT NULL DEFAULT 1,
    tenant_id     TEXT         NOT NULL DEFAULT 'system',
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_tasks_status
        CHECK (status IN ('open', 'in_progress', 'review', 'done', 'cancelled')),
    CONSTRAINT chk_tasks_priority
        CHECK (priority IN ('low', 'medium', 'high', 'critical'))
);

CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON task_service.tasks (project_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON task_service.tasks (status);
CREATE INDEX IF NOT EXISTS idx_tasks_assignee_id ON task_service.tasks (assignee_id);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON task_service.tasks (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_tasks_tenant_id ON task_service.tasks (tenant_id);
CREATE INDEX IF NOT EXISTS idx_tasks_tenant_project ON task_service.tasks (tenant_id, project_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| project_id | UUID | NOT NULL | 所属プロジェクト ID |
| title | TEXT | NOT NULL | タスクタイトル |
| description | TEXT | | タスク説明 |
| status | TEXT | NOT NULL, DEFAULT 'open', CHECK | ステータス（open / in_progress / review / done / cancelled） |
| priority | TEXT | NOT NULL, DEFAULT 'medium', CHECK | 優先度（low / medium / high / critical） |
| assignee_id | TEXT | | 担当者 ID |
| reporter_id | TEXT | NOT NULL | 報告者 ID |
| due_date | TIMESTAMPTZ | | 期限日時 |
| labels | JSONB | NOT NULL, DEFAULT '[]' | ラベル一覧（JSON 配列） |
| created_by | TEXT | NOT NULL | 作成者 ID |
| updated_by | TEXT | NOT NULL, DEFAULT '' | 最終更新者 ID |
| version | INT | NOT NULL, DEFAULT 1 | 楽観的ロック用バージョン番号 |
| tenant_id | TEXT | NOT NULL, DEFAULT 'system' | テナント ID |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

#### ステータス遷移

```
open → in_progress → review → done
                    ↓
                 cancelled（任意の状態から遷移可能）
```

ADR-0083 にてステータス遷移の強制実装方針を定義している。

#### RLS（Row Level Security）

migration 005 で RLS を有効化し、migration 011 で FORCE + AS RESTRICTIVE + WITH CHECK を追加した（HIGH-BIZ-005 対応）。

```sql
ALTER TABLE task_service.tasks ENABLE ROW LEVEL SECURITY;
ALTER TABLE task_service.tasks FORCE ROW LEVEL SECURITY;

-- テーブルオーナーを含む全ロールに RLS を強制し、INSERT/UPDATE 時のテナント検証も行う
CREATE POLICY tenant_isolation ON task_service.tasks
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));
```

---

### task_checklist_items（タスクチェックリスト項目）

タスク内のサブタスク / チェック項目を管理する。`tasks.id` への FK で親タスク削除時に CASCADE 削除される。
`sort_order` で表示順を制御する。

```sql
CREATE TABLE IF NOT EXISTS task_service.task_checklist_items (
    id           UUID         PRIMARY KEY,
    task_id      UUID         NOT NULL REFERENCES task_service.tasks(id) ON DELETE CASCADE,
    title        TEXT         NOT NULL,
    is_completed BOOLEAN      NOT NULL DEFAULT FALSE,
    sort_order   INTEGER      NOT NULL DEFAULT 0,
    tenant_id    TEXT         NOT NULL DEFAULT 'system',
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_task_checklist_items_task_id ON task_service.task_checklist_items (task_id);
CREATE INDEX IF NOT EXISTS idx_task_checklist_items_tenant_id ON task_service.task_checklist_items (tenant_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| task_id | UUID | FK → tasks.id ON DELETE CASCADE, NOT NULL | 親タスク ID |
| title | TEXT | NOT NULL | チェック項目テキスト |
| is_completed | BOOLEAN | NOT NULL, DEFAULT FALSE | 完了フラグ |
| sort_order | INTEGER | NOT NULL, DEFAULT 0 | 表示順序 |
| tenant_id | TEXT | NOT NULL, DEFAULT 'system' | テナント ID |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |

#### RLS（Row Level Security）

migration 005 で RLS を有効化し、migration 011 で FORCE + AS RESTRICTIVE + WITH CHECK を追加した（HIGH-BIZ-005 対応）。

```sql
ALTER TABLE task_service.task_checklist_items ENABLE ROW LEVEL SECURITY;
ALTER TABLE task_service.task_checklist_items FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON task_service.task_checklist_items
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));
```

---

### outbox_events（Outbox イベント）

Outbox パターンによりタスク変更イベントを Kafka へ確実に配信するためのキューテーブル。
`published_at IS NULL` の部分インデックスにより未送信イベントの高速スキャンを実現する。

```sql
CREATE TABLE IF NOT EXISTS task_service.outbox_events (
    id             UUID         PRIMARY KEY,
    aggregate_type TEXT         NOT NULL,
    aggregate_id   TEXT         NOT NULL,
    event_type     TEXT         NOT NULL,
    payload        JSONB        NOT NULL,
    tenant_id      TEXT         NOT NULL DEFAULT 'system',
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    published_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_outbox_unpublished
    ON task_service.outbox_events (created_at)
    WHERE published_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_outbox_events_tenant_id ON task_service.outbox_events (tenant_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| aggregate_type | TEXT | NOT NULL | 集約タイプ（例: `task`） |
| aggregate_id | TEXT | NOT NULL | 集約 ID |
| event_type | TEXT | NOT NULL | イベント種別（例: `TaskCreated`） |
| payload | JSONB | NOT NULL | イベントペイロード |
| tenant_id | TEXT | NOT NULL, DEFAULT 'system' | テナント ID |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| published_at | TIMESTAMPTZ | | Kafka 送信完了日時（NULL = 未送信） |

#### RLS（Row Level Security）

migration 009 で RLS を設定（HIGH-005 対応）。バックグラウンドパブリッシャーは `set_config` を呼ばないため
`current_setting` が NULL の場合は全テナントのイベントを参照可能とする設計となっている。

```sql
ALTER TABLE task_service.outbox_events ENABLE ROW LEVEL SECURITY;

-- バックグラウンドパブリッシャー（set_config 未呼出し）は全テナント参照可能
CREATE POLICY tenant_isolation ON task_service.outbox_events
    USING (
        current_setting('app.current_tenant_id', true) IS NULL
        OR tenant_id = current_setting('app.current_tenant_id', true)::TEXT
    );
```

---

## マルチテナント対応

全テーブルに `tenant_id TEXT NOT NULL DEFAULT 'system'` カラムと RLS ポリシーを設定する。
コンテキスト設定は `set_config('app.current_tenant_id', tenant_id, true)` を各 DB 操作前に実行する。

### RLS コンテキスト設定（Go サービス側）

```go
// 各DBクエリの前にセッション変数でテナントIDを設定する
_, err := tx.ExecContext(ctx,
    "SELECT set_config('app.current_tenant_id', $1, true)",
    tenantID,
)
```

### RLS ポリシー設計の差異

| テーブル | FORCE | AS RESTRICTIVE | WITH CHECK | バックグラウンド NULL 許可 |
| --- | --- | --- | --- | --- |
| tasks | あり | あり | あり | なし |
| task_checklist_items | あり | あり | あり | なし |
| outbox_events | なし | なし | なし | あり（Outbox パブリッシャー用） |

---

## マイグレーション

マイグレーションファイルは `regions/service/task/database/postgres/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_tasks.up.sql` | `task_service` スキーマ・`tasks` テーブル・インデックス作成 |
| `001_create_tasks.down.sql` | テーブル削除 |
| `002_create_task_checklist_items.up.sql` | `task_checklist_items` テーブル・インデックス作成 |
| `002_create_task_checklist_items.down.sql` | テーブル削除 |
| `003_add_updated_by_and_version.up.sql` | `tasks` に `updated_by VARCHAR(255)` と `version INT` カラム追加（冪等性 DO ブロック） |
| `003_add_updated_by_and_version.down.sql` | カラム削除 |
| `004_create_outbox.up.sql` | `outbox_events` テーブル・部分インデックス作成 |
| `004_create_outbox.down.sql` | テーブル削除 |
| `005_add_tenant_id_and_rls.up.sql` | `tasks` / `task_checklist_items` に `tenant_id` カラム追加・RLS 有効化・ポリシー作成 |
| `005_add_tenant_id_and_rls.down.sql` | `tenant_id` カラム・RLS ポリシー削除 |
| `006_add_check_constraints.up.sql` | `status`（open/in_progress/done/cancelled）と `priority` に CHECK 制約追加（多層防御） |
| `006_add_check_constraints.down.sql` | CHECK 制約削除 |
| `007_fix_updated_by_type.up.sql` | `updated_by` を VARCHAR(255) から TEXT NOT NULL DEFAULT '' に変更（型統一） |
| `007_fix_updated_by_type.down.sql` | `updated_by` を元の型に戻す |
| `008_alter_project_id_to_uuid.up.sql` | `tasks.project_id` を TEXT から UUID 型へ変更 |
| `008_alter_project_id_to_uuid.down.sql` | `project_id` を TEXT 型に戻す |
| `009_add_outbox_rls.up.sql` | `outbox_events` に `tenant_id` カラム追加・RLS 有効化・ポリシー作成（HIGH-005 対応） |
| `009_add_outbox_rls.down.sql` | `tenant_id` カラム・RLS ポリシー削除 |
| `010_fix_status_check_constraint.up.sql` | `chk_tasks_status` を再作成して `'review'` 状態を追加（C-005 対応） |
| `010_fix_status_check_constraint.down.sql` | `review` を除いた旧制約に戻す |
| `011_fix_rls_force_restrictive.up.sql` | `tasks` / `task_checklist_items` に FORCE ROW LEVEL SECURITY・AS RESTRICTIVE・WITH CHECK 追加（HIGH-BIZ-005 対応） |
| `011_fix_rls_force_restrictive.down.sql` | FORCE・RESTRICTIVE・WITH CHECK を削除してポリシーを再作成 |

---

## 関連ドキュメント

- [server.md](server.md) -- Task サーバー設計（API・アーキテクチャ）
- [implementation.md](implementation.md) -- Go 実装詳細
- [board/database.md](../board/database.md) -- Board サービスデータベース設計（同一 DB 内の別スキーマ）
- [activity/database.md](../activity/database.md) -- Activity サービスデータベース設計（同一 DB 内の別スキーマ）
- [docs/architecture/adr/0083-task-status-transition-enforcement.md](../../../../docs/architecture/adr/0083-task-status-transition-enforcement.md) -- タスクステータス遷移強制の ADR
