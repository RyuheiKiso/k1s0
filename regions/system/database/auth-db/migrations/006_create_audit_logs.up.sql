-- auth-db: audit_logs テーブル作成（月次パーティショニング）

CREATE TABLE IF NOT EXISTS auth.audit_logs (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID         REFERENCES auth.users(id) ON DELETE SET NULL,
    event_type  VARCHAR(100) NOT NULL,
    action      VARCHAR(100) NOT NULL,
    resource    VARCHAR(255),
    resource_id VARCHAR(255),
    result      VARCHAR(50)  NOT NULL DEFAULT 'SUCCESS',
    detail      JSONB,
    ip_address  INET,
    user_agent  TEXT,
    trace_id    VARCHAR(64),
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
) PARTITION BY RANGE (created_at);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id_created_at
    ON auth.audit_logs (user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_type_created_at
    ON auth.audit_logs (event_type, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action_created_at
    ON auth.audit_logs (action, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_trace_id
    ON auth.audit_logs (trace_id)
    WHERE trace_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource
    ON auth.audit_logs (resource, resource_id)
    WHERE resource IS NOT NULL;

-- 初期パーティション（直近3ヶ月 + 将来3ヶ月）
-- 本番運用では cron ジョブまたは pg_partman で自動作成する
CREATE TABLE IF NOT EXISTS auth.audit_logs_2026_01 PARTITION OF auth.audit_logs
    FOR VALUES FROM ('2026-01-01') TO ('2026-02-01');
CREATE TABLE IF NOT EXISTS auth.audit_logs_2026_02 PARTITION OF auth.audit_logs
    FOR VALUES FROM ('2026-02-01') TO ('2026-03-01');
CREATE TABLE IF NOT EXISTS auth.audit_logs_2026_03 PARTITION OF auth.audit_logs
    FOR VALUES FROM ('2026-03-01') TO ('2026-04-01');
CREATE TABLE IF NOT EXISTS auth.audit_logs_2026_04 PARTITION OF auth.audit_logs
    FOR VALUES FROM ('2026-04-01') TO ('2026-05-01');
CREATE TABLE IF NOT EXISTS auth.audit_logs_2026_05 PARTITION OF auth.audit_logs
    FOR VALUES FROM ('2026-05-01') TO ('2026-06-01');
CREATE TABLE IF NOT EXISTS auth.audit_logs_2026_06 PARTITION OF auth.audit_logs
    FOR VALUES FROM ('2026-06-01') TO ('2026-07-01');

-- デフォルトパーティション（パーティションが存在しない範囲のデータを受け付ける）
CREATE TABLE IF NOT EXISTS auth.audit_logs_default PARTITION OF auth.audit_logs DEFAULT;
