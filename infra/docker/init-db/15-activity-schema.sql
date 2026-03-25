-- Activity Service (service tier)
\c k1s0_service;

CREATE SCHEMA IF NOT EXISTS activity_service;

-- アクティビティテーブル（タスクに対するコメント・作業時間・ステータス変更等の操作履歴を管理する）
CREATE TABLE IF NOT EXISTS activity_service.activities (
    id               UUID         PRIMARY KEY,
    task_id          TEXT         NOT NULL,
    actor_id         TEXT         NOT NULL,
    activity_type    TEXT         NOT NULL,
    content          TEXT,
    duration_minutes INTEGER,
    status           TEXT         NOT NULL DEFAULT 'active',
    metadata         JSONB,
    -- idempotency_key はテナントスコープで一意にする（SL-1 監査対応）
    -- システム全体でユニークにすると異なるテナントが同じキーを使った場合に競合が発生する
    idempotency_key  TEXT,
    tenant_id        TEXT         NOT NULL DEFAULT 'system',
    version          INTEGER      NOT NULL DEFAULT 1,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_activities_task_id ON activity_service.activities(task_id);
CREATE INDEX IF NOT EXISTS idx_activities_actor_id ON activity_service.activities(actor_id);
CREATE INDEX IF NOT EXISTS idx_activities_activity_type ON activity_service.activities(activity_type);
CREATE INDEX IF NOT EXISTS idx_activities_status ON activity_service.activities(status);
CREATE INDEX IF NOT EXISTS idx_activities_created_at ON activity_service.activities(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_activities_tenant_id ON activity_service.activities(tenant_id);
CREATE INDEX IF NOT EXISTS idx_activities_tenant_task ON activity_service.activities(tenant_id, task_id);

-- idempotency_key はテナントスコープで一意にする（SL-1 監査対応）
-- テナント間で同じキーが衝突しないよう (tenant_id, idempotency_key) の複合 UNIQUE 制約を使用する
CREATE UNIQUE INDEX IF NOT EXISTS uq_activities_tenant_idempotency
    ON activity_service.activities(tenant_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;

-- Outboxイベントテーブル（アクティビティ変更イベントをKafkaへ送信するためのOutboxパターン）
CREATE TABLE IF NOT EXISTS activity_service.outbox_events (
    id             UUID         PRIMARY KEY,
    aggregate_type TEXT         NOT NULL,
    aggregate_id   TEXT         NOT NULL,
    event_type     TEXT         NOT NULL,
    payload        JSONB        NOT NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    published_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_activity_outbox_unpublished
    ON activity_service.outbox_events(created_at)
    WHERE published_at IS NULL;

ALTER TABLE activity_service.activities ENABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON activity_service.activities;
CREATE POLICY tenant_isolation ON activity_service.activities
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- スーパーユーザーも含む全ユーザーに RLS を強制する
ALTER TABLE activity_service.activities FORCE ROW LEVEL SECURITY;

-- k1s0 アプリユーザーへのスキーマ使用権限とテーブル操作権限を付与する
GRANT USAGE ON SCHEMA activity_service TO k1s0;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA activity_service TO k1s0;
ALTER DEFAULT PRIVILEGES IN SCHEMA activity_service GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0;

-- sqlx マイグレーションが ALTER TABLE を実行できるようにテーブルオーナーを k1s0 に変更する
ALTER TABLE activity_service.activities OWNER TO k1s0;
ALTER TABLE activity_service.outbox_events OWNER TO k1s0;

-- sqlx が k1s0_service 内に新規スキーマを作成できるように DATABASE レベルの CREATE 権限を付与する
GRANT CREATE ON DATABASE k1s0_service TO k1s0;
-- sqlx マイグレーションが activity_service スキーマ内に _sqlx_migrations テーブルを作成できるように
-- スキーマレベルの CREATE 権限を付与する（GRANT CREATE ON DATABASE とは別物）（C-2 監査対応）
GRANT CREATE ON SCHEMA activity_service TO k1s0;
