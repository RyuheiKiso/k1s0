CREATE SCHEMA IF NOT EXISTS activity_service;

SET search_path TO activity_service;

-- アクティビティテーブル（タスクに対するコメント・作業時間・ステータス変更等の操作履歴を管理する）
CREATE TABLE IF NOT EXISTS activities (
    id               UUID         PRIMARY KEY,
    task_id          TEXT         NOT NULL,
    actor_id         TEXT         NOT NULL,
    activity_type    TEXT         NOT NULL,
    content          TEXT,
    duration_minutes INTEGER,
    status           TEXT         NOT NULL DEFAULT 'active',
    metadata         JSONB,
    idempotency_key  TEXT         UNIQUE,
    version          INTEGER      NOT NULL DEFAULT 1,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_activities_task_id ON activities (task_id);
CREATE INDEX IF NOT EXISTS idx_activities_actor_id ON activities (actor_id);
CREATE INDEX IF NOT EXISTS idx_activities_activity_type ON activities (activity_type);
CREATE INDEX IF NOT EXISTS idx_activities_status ON activities (status);
CREATE INDEX IF NOT EXISTS idx_activities_created_at ON activities (created_at DESC);
