-- infra/docker/init-db/19-scheduler-schema.sql
-- scheduler スキーマ作成（全マイグレーション 001〜006 を統合した最終状態）
-- 権威ソース: regions/system/database/scheduler-db/migrations/
-- scheduler スキーマは scheduler_db データベース内に作成する

\c scheduler_db;

-- 拡張機能の有効化
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- スキーマの作成
CREATE SCHEMA IF NOT EXISTS scheduler;

-- updated_at 自動更新トリガー関数
CREATE OR REPLACE FUNCTION scheduler.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ================================================================
-- scheduler_jobs テーブル（スケジュールジョブ定義）
-- 権威: regions/system/server/rust/scheduler/src/infrastructure/
-- ID は 'job_' プレフィックス付き VARCHAR(64)（migration 006 で UUID から変換済み）
-- ================================================================
CREATE TABLE IF NOT EXISTS scheduler.scheduler_jobs (
    id              VARCHAR(64)  PRIMARY KEY,
    name            VARCHAR(255) NOT NULL UNIQUE,
    cron_expression VARCHAR(255) NOT NULL,
    job_type        VARCHAR(50)  NOT NULL DEFAULT 'default',
    payload         JSONB        NOT NULL DEFAULT '{}',
    enabled         BOOLEAN      NOT NULL DEFAULT true,
    max_retries     INT          NOT NULL DEFAULT 3,
    -- migration 004 で追加されたフィールド
    description     TEXT,
    timezone        VARCHAR(100) NOT NULL DEFAULT 'UTC',
    target_type     VARCHAR(50)  NOT NULL DEFAULT 'kafka',
    target          TEXT,
    last_run_at     TIMESTAMPTZ,
    next_run_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_scheduler_jobs_name ON scheduler.scheduler_jobs (name);
CREATE INDEX IF NOT EXISTS idx_scheduler_jobs_enabled ON scheduler.scheduler_jobs (enabled);
CREATE INDEX IF NOT EXISTS idx_scheduler_jobs_next_run_at ON scheduler.scheduler_jobs (next_run_at);

CREATE TRIGGER trigger_scheduler_jobs_update_updated_at
    BEFORE UPDATE ON scheduler.scheduler_jobs
    FOR EACH ROW
    EXECUTE FUNCTION scheduler.update_updated_at();

-- ================================================================
-- job_executions テーブル（ジョブ実行履歴）
-- ID は 'exec_' プレフィックス付き VARCHAR(64)（migration 006 で UUID から変換済み）
-- status は 'running' | 'succeeded' | 'failed'（migration 005 で 'completed' → 'succeeded' に変更）
-- ================================================================
CREATE TABLE IF NOT EXISTS scheduler.job_executions (
    id            VARCHAR(64)  PRIMARY KEY,
    job_id        VARCHAR(64)  NOT NULL REFERENCES scheduler.scheduler_jobs(id) ON DELETE CASCADE,
    -- migration 005: status の 'completed' を 'succeeded' に変更
    status        VARCHAR(50)  NOT NULL DEFAULT 'running',
    -- migration 005 で追加されたフィールド: 実行トリガー種別
    triggered_by  VARCHAR(50)  NOT NULL DEFAULT 'scheduler',
    started_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    completed_at  TIMESTAMPTZ,
    error_message TEXT,

    CONSTRAINT chk_job_executions_status CHECK (status IN ('running', 'succeeded', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_job_executions_job_id ON scheduler.job_executions (job_id);
CREATE INDEX IF NOT EXISTS idx_job_executions_status ON scheduler.job_executions (status);
CREATE INDEX IF NOT EXISTS idx_job_executions_started_at ON scheduler.job_executions (started_at DESC);

-- k1s0 アプリユーザーへのスキーマ・テーブル権限付与
GRANT USAGE ON SCHEMA scheduler TO k1s0;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA scheduler TO k1s0;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA scheduler TO k1s0;
-- 将来追加されるテーブルにも自動で権限を付与する
ALTER DEFAULT PRIVILEGES IN SCHEMA scheduler
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0;
