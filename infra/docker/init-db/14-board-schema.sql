-- Board Service (service tier)
\c k1s0_service;

CREATE SCHEMA IF NOT EXISTS board_service;

-- ボードカラムテーブル（Kanbanボードのカラムを管理する。project_id × status_code の組み合わせで管理。）
CREATE TABLE IF NOT EXISTS board_service.board_columns (
    id          UUID         PRIMARY KEY,
    project_id  TEXT         NOT NULL,
    status_code TEXT         NOT NULL,
    wip_limit   INTEGER      NOT NULL DEFAULT 0,
    task_count  INTEGER      NOT NULL DEFAULT 0,
    tenant_id   TEXT         NOT NULL DEFAULT 'system',
    version     INTEGER      NOT NULL DEFAULT 1,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_board_columns_project_status UNIQUE (project_id, status_code),
    CONSTRAINT chk_task_count CHECK (task_count >= 0)
);

CREATE INDEX IF NOT EXISTS idx_board_columns_project_id ON board_service.board_columns(project_id);
CREATE INDEX IF NOT EXISTS idx_board_columns_status_code ON board_service.board_columns(status_code);
CREATE INDEX IF NOT EXISTS idx_board_columns_tenant_id ON board_service.board_columns(tenant_id);
CREATE INDEX IF NOT EXISTS idx_board_columns_tenant_project ON board_service.board_columns(tenant_id, project_id);

-- Outboxイベントテーブル（ボードカラム変更イベントをKafkaへ送信するためのOutboxパターン）
CREATE TABLE IF NOT EXISTS board_service.outbox_events (
    id             UUID         PRIMARY KEY,
    aggregate_type TEXT         NOT NULL,
    aggregate_id   TEXT         NOT NULL,
    event_type     TEXT         NOT NULL,
    payload        JSONB        NOT NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    published_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_board_outbox_unpublished
    ON board_service.outbox_events(created_at)
    WHERE published_at IS NULL;

ALTER TABLE board_service.board_columns ENABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON board_service.board_columns;
CREATE POLICY tenant_isolation ON board_service.board_columns
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);
-- スーパーユーザーも含む全ユーザーに RLS を強制する
ALTER TABLE board_service.board_columns FORCE ROW LEVEL SECURITY;

-- k1s0 アプリユーザーへのスキーマ使用権限とテーブル操作権限を付与する
GRANT USAGE ON SCHEMA board_service TO k1s0;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA board_service TO k1s0;
ALTER DEFAULT PRIVILEGES IN SCHEMA board_service GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0;

-- sqlx マイグレーションが ALTER TABLE を実行できるようにテーブルオーナーを k1s0 に変更する
ALTER TABLE board_service.board_columns OWNER TO k1s0;
ALTER TABLE board_service.outbox_events OWNER TO k1s0;
