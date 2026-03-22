SET search_path TO task_service;

-- タスクチェックリスト項目テーブル（タスク内のサブタスク/チェック項目を管理する）
CREATE TABLE IF NOT EXISTS task_checklist_items (
    id           UUID         PRIMARY KEY,
    task_id      UUID         NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    title        TEXT         NOT NULL,
    is_completed BOOLEAN      NOT NULL DEFAULT FALSE,
    sort_order   INTEGER      NOT NULL DEFAULT 0,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_task_checklist_items_task_id ON task_checklist_items (task_id);
