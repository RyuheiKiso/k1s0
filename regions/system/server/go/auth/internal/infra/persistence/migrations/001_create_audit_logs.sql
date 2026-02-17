-- 001_create_audit_logs.sql
-- 監査ログテーブルの作成

CREATE TABLE IF NOT EXISTS audit_logs (
    id          UUID PRIMARY KEY,
    event_type  VARCHAR(100) NOT NULL,
    user_id     UUID NOT NULL,
    ip_address  VARCHAR(45),
    user_agent  TEXT,
    resource    VARCHAR(500),
    action      VARCHAR(20),
    result      VARCHAR(20) NOT NULL CHECK (result IN ('SUCCESS', 'FAILURE')),
    metadata    JSONB DEFAULT '{}',
    recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- インデックス
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs (user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_type ON audit_logs (event_type);
CREATE INDEX IF NOT EXISTS idx_audit_logs_recorded_at ON audit_logs (recorded_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_result ON audit_logs (result);
