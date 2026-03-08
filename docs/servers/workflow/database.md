# system-workflow-server データベース設計

## スキーマ

スキーマ名: `workflow`

```sql
CREATE SCHEMA IF NOT EXISTS workflow;
```

---

## テーブル一覧

| テーブル名 | 説明 |
| --- | --- |
| workflow_definitions | ワークフロー定義 |
| workflow_instances | ワークフローインスタンス（実行中の申請） |
| workflow_tasks | ワークフロータスク（承認ステップ） |

---

## ER 図

```
workflow_definitions 1──* workflow_instances 1──* workflow_tasks
```

---

## テーブル定義

### workflow_definitions（ワークフロー定義）

ワークフローの定義（ステップ構成）を管理する。steps に承認ステップの JSON 定義を保持する。

```sql
CREATE TABLE IF NOT EXISTS workflow.workflow_definitions (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) NOT NULL UNIQUE,
    description TEXT         NOT NULL DEFAULT '',
    steps       JSONB        NOT NULL DEFAULT '[]',
    enabled     BOOLEAN      NOT NULL DEFAULT true,
    version     INT          NOT NULL DEFAULT 1,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_workflow_definitions_name ON workflow.workflow_definitions (name);
CREATE INDEX IF NOT EXISTS idx_workflow_definitions_enabled ON workflow.workflow_definitions (enabled);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| name | VARCHAR(255) | UNIQUE, NOT NULL | ワークフロー名 |
| description | TEXT | NOT NULL, DEFAULT '' | 説明 |
| steps | JSONB | NOT NULL, DEFAULT '[]' | ステップ定義 |
| enabled | BOOLEAN | NOT NULL, DEFAULT true | 有効フラグ |
| version | INT | NOT NULL, DEFAULT 1 | バージョン番号 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

### workflow_instances（ワークフローインスタンス）

ワークフロー定義に基づく個別の申請・実行インスタンスを管理する。

```sql
CREATE TABLE IF NOT EXISTS workflow.workflow_instances (
    id               UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    definition_id    UUID         NOT NULL REFERENCES workflow.workflow_definitions(id),
    workflow_name    VARCHAR(255) NOT NULL DEFAULT '',
    title            VARCHAR(255) NOT NULL DEFAULT '',
    initiator_id     VARCHAR(255) NOT NULL DEFAULT '',
    current_step_id  VARCHAR(255) NOT NULL DEFAULT '',
    status           VARCHAR(50)  NOT NULL DEFAULT 'running',
    context          JSONB        NOT NULL DEFAULT '{}',
    started_at       TIMESTAMPTZ,
    completed_at     TIMESTAMPTZ,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_instances_status CHECK (status IN ('running', 'completed', 'cancelled', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_workflow_instances_definition_id ON workflow.workflow_instances (definition_id);
CREATE INDEX IF NOT EXISTS idx_workflow_instances_status ON workflow.workflow_instances (status);
CREATE INDEX IF NOT EXISTS idx_workflow_instances_initiator_id ON workflow.workflow_instances (initiator_id);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| definition_id | UUID | FK → workflow_definitions.id, NOT NULL | ワークフロー定義 ID |
| workflow_name | VARCHAR(255) | NOT NULL, DEFAULT '' | ワークフロー名 |
| title | VARCHAR(255) | NOT NULL, DEFAULT '' | 申請タイトル |
| initiator_id | VARCHAR(255) | NOT NULL, DEFAULT '' | 申請者 ID |
| current_step_id | VARCHAR(255) | NOT NULL, DEFAULT '' | 現在のステップ ID |
| status | VARCHAR(50) | NOT NULL, DEFAULT 'running' | ステータス（running/completed/cancelled/failed） |
| context | JSONB | NOT NULL, DEFAULT '{}' | コンテキストデータ |
| started_at | TIMESTAMPTZ | | 開始日時 |
| completed_at | TIMESTAMPTZ | | 完了日時 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

### workflow_tasks（ワークフロータスク）

ワークフローインスタンス内の承認・却下タスクを管理する。

```sql
CREATE TABLE IF NOT EXISTS workflow.workflow_tasks (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    instance_id  UUID         NOT NULL REFERENCES workflow.workflow_instances(id) ON DELETE CASCADE,
    step_id      VARCHAR(255) NOT NULL DEFAULT '',
    step_name    VARCHAR(255) NOT NULL DEFAULT '',
    assignee_id  VARCHAR(255) NOT NULL DEFAULT '',
    status       VARCHAR(50)  NOT NULL DEFAULT 'pending',
    comment      TEXT,
    actor_id     VARCHAR(255),
    due_at       TIMESTAMPTZ,
    decided_at   TIMESTAMPTZ,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_tasks_status CHECK (status IN ('pending', 'assigned', 'approved', 'rejected'))
);

CREATE INDEX IF NOT EXISTS idx_workflow_tasks_instance_id ON workflow.workflow_tasks (instance_id);
CREATE INDEX IF NOT EXISTS idx_workflow_tasks_assignee_id ON workflow.workflow_tasks (assignee_id);
CREATE INDEX IF NOT EXISTS idx_workflow_tasks_status ON workflow.workflow_tasks (status);
CREATE INDEX IF NOT EXISTS idx_workflow_tasks_due_at ON workflow.workflow_tasks (due_at);
```

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| id | UUID | PK | 主キー |
| instance_id | UUID | FK → workflow_instances.id, NOT NULL | インスタンス ID |
| step_id | VARCHAR(255) | NOT NULL, DEFAULT '' | ステップ ID |
| step_name | VARCHAR(255) | NOT NULL, DEFAULT '' | ステップ名 |
| assignee_id | VARCHAR(255) | NOT NULL, DEFAULT '' | 担当者 ID |
| status | VARCHAR(50) | NOT NULL, DEFAULT 'pending' | ステータス（pending/assigned/approved/rejected） |
| comment | TEXT | | コメント |
| actor_id | VARCHAR(255) | | 操作者 ID |
| due_at | TIMESTAMPTZ | | 期限日時 |
| decided_at | TIMESTAMPTZ | | 決定日時 |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時 |

---

## マイグレーション

マイグレーションファイルは `regions/system/database/workflow-db/migrations/` に配置。

| ファイル | 内容 |
| --- | --- |
| `001_create_schema.up.sql` | `workflow` スキーマ・pgcrypto 拡張・updated_at 関数作成 |
| `001_create_schema.down.sql` | スキーマ削除 |
| `002_create_workflow_definitions.up.sql` | workflow_definitions テーブル作成 |
| `002_create_workflow_definitions.down.sql` | テーブル削除 |
| `003_create_workflow_instances.up.sql` | workflow_instances テーブル作成 |
| `003_create_workflow_instances.down.sql` | テーブル削除 |
| `004_create_workflow_tasks.up.sql` | workflow_tasks テーブル作成 |
| `004_create_workflow_tasks.down.sql` | テーブル削除 |

---

## updated_at 自動更新トリガー

```sql
CREATE OR REPLACE FUNCTION workflow.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_workflow_definitions_update_updated_at
    BEFORE UPDATE ON workflow.workflow_definitions
    FOR EACH ROW EXECUTE FUNCTION workflow.update_updated_at();

CREATE TRIGGER trigger_workflow_instances_update_updated_at
    BEFORE UPDATE ON workflow.workflow_instances
    FOR EACH ROW EXECUTE FUNCTION workflow.update_updated_at();

CREATE TRIGGER trigger_workflow_tasks_update_updated_at
    BEFORE UPDATE ON workflow.workflow_tasks
    FOR EACH ROW EXECUTE FUNCTION workflow.update_updated_at();
```
