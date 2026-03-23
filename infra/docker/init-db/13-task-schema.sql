-- Task Service (service tier)
\c k1s0_service;

CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE SCHEMA IF NOT EXISTS task_service;

-- タスクテーブル（タスクの基本情報を管理する）
CREATE TABLE IF NOT EXISTS task_service.tasks (
    id            UUID         PRIMARY KEY,
    project_id    TEXT         NOT NULL,
    title         TEXT         NOT NULL,
    description   TEXT,
    status        TEXT         NOT NULL DEFAULT 'open'
              CONSTRAINT chk_tasks_status CHECK (status IN ('open', 'in_progress', 'done', 'cancelled')),
    priority      TEXT         NOT NULL DEFAULT 'medium'
              CONSTRAINT chk_tasks_priority CHECK (priority IN ('low', 'medium', 'high', 'critical')),
    assignee_id   TEXT,
    reporter_id   TEXT         NOT NULL,
    due_date      TIMESTAMPTZ,
    labels        JSONB        NOT NULL DEFAULT '[]',
    tenant_id     TEXT         NOT NULL DEFAULT 'system',
    created_by    TEXT         NOT NULL,
    updated_by    TEXT         NOT NULL DEFAULT '',
    version       INT          NOT NULL DEFAULT 1,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON task_service.tasks(project_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON task_service.tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_assignee_id ON task_service.tasks(assignee_id);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON task_service.tasks(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_tasks_tenant_id ON task_service.tasks(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tasks_tenant_project ON task_service.tasks(tenant_id, project_id);

-- タスクチェックリスト項目テーブル（タスク内のサブタスク/チェック項目を管理する）
CREATE TABLE IF NOT EXISTS task_service.task_checklist_items (
    id           UUID         PRIMARY KEY,
    task_id      UUID         NOT NULL REFERENCES task_service.tasks(id) ON DELETE CASCADE,
    title        TEXT         NOT NULL,
    is_completed BOOLEAN      NOT NULL DEFAULT FALSE,
    sort_order   INTEGER      NOT NULL DEFAULT 0,
    tenant_id    TEXT         NOT NULL DEFAULT 'system',
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_task_checklist_items_task_id ON task_service.task_checklist_items(task_id);
CREATE INDEX IF NOT EXISTS idx_task_checklist_items_tenant_id ON task_service.task_checklist_items(tenant_id);

-- Outboxイベントテーブル（タスク変更イベントをKafkaへ送信するためのOutboxパターン）
CREATE TABLE IF NOT EXISTS task_service.outbox_events (
    id             UUID         PRIMARY KEY,
    aggregate_type TEXT         NOT NULL,
    aggregate_id   TEXT         NOT NULL,
    event_type     TEXT         NOT NULL,
    payload        JSONB        NOT NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    published_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_task_outbox_unpublished
    ON task_service.outbox_events(created_at)
    WHERE published_at IS NULL;

ALTER TABLE task_service.tasks ENABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON task_service.tasks;
CREATE POLICY tenant_isolation ON task_service.tasks
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

ALTER TABLE task_service.task_checklist_items ENABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON task_service.task_checklist_items;
CREATE POLICY tenant_isolation ON task_service.task_checklist_items
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);
