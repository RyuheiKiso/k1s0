CREATE SCHEMA IF NOT EXISTS task_service;

SET search_path TO task_service;

-- タスクテーブル（タスクの基本情報を管理する）
CREATE TABLE IF NOT EXISTS tasks (
    id            UUID         PRIMARY KEY,
    project_id    TEXT         NOT NULL,
    title         TEXT         NOT NULL,
    description   TEXT,
    status        TEXT         NOT NULL DEFAULT 'open',
    priority      TEXT         NOT NULL DEFAULT 'medium',
    assignee_id   TEXT,
    reporter_id   TEXT         NOT NULL,
    due_date      TIMESTAMPTZ,
    labels        JSONB        NOT NULL DEFAULT '[]',
    created_by    TEXT         NOT NULL,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks (project_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks (status);
CREATE INDEX IF NOT EXISTS idx_tasks_assignee_id ON tasks (assignee_id);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks (created_at DESC);
