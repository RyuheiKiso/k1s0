-- saga-db: workflow_definitions テーブル作成

CREATE TABLE IF NOT EXISTS saga.workflow_definitions (
    name   VARCHAR(255) PRIMARY KEY,
    steps  JSONB        NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
