CREATE SCHEMA IF NOT EXISTS board_service;

SET search_path TO board_service;

-- ボードカラムテーブル（Kanbanボードのカラムを管理する。project_id × status_code の組み合わせで管理。）
CREATE TABLE IF NOT EXISTS board_columns (
    id          UUID         PRIMARY KEY,
    project_id  TEXT         NOT NULL,
    status_code TEXT         NOT NULL,
    wip_limit   INTEGER      NOT NULL DEFAULT 0,
    task_count  INTEGER      NOT NULL DEFAULT 0,
    version     INTEGER      NOT NULL DEFAULT 1,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_board_columns_project_status UNIQUE (project_id, status_code),
    CONSTRAINT chk_task_count CHECK (task_count >= 0)
);

CREATE INDEX IF NOT EXISTS idx_board_columns_project_id ON board_columns (project_id);
CREATE INDEX IF NOT EXISTS idx_board_columns_status_code ON board_columns (status_code);
