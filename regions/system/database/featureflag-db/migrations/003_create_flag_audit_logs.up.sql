-- featureflag-db: flag_audit_logs テーブルの作成

CREATE TABLE IF NOT EXISTS featureflag.flag_audit_logs (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    flag_id     UUID         NOT NULL REFERENCES featureflag.feature_flags(id) ON DELETE CASCADE,
    flag_key    VARCHAR(255) NOT NULL,
    action      VARCHAR(50)  NOT NULL,
    before_json JSONB,
    after_json  JSONB,
    changed_by  VARCHAR(255) NOT NULL,
    trace_id    VARCHAR(255),
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_flag_audit_logs_flag_id ON featureflag.flag_audit_logs (flag_id);
CREATE INDEX IF NOT EXISTS idx_flag_audit_logs_created_at ON featureflag.flag_audit_logs (created_at);
CREATE INDEX IF NOT EXISTS idx_flag_audit_logs_action ON featureflag.flag_audit_logs (action);
