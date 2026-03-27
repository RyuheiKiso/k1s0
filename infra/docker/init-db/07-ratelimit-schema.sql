-- infra/docker/init-db/07-ratelimit-schema.sql

\c ratelimit_db;

CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE SCHEMA IF NOT EXISTS ratelimit;

CREATE OR REPLACE FUNCTION ratelimit.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE IF NOT EXISTS ratelimit.rate_limit_rules (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) UNIQUE NOT NULL,
    key         VARCHAR(255) NOT NULL,
    limit_count BIGINT       NOT NULL,
    window_secs BIGINT       NOT NULL,
    algorithm   VARCHAR(50)  NOT NULL DEFAULT 'token_bucket',
    enabled     BOOLEAN      NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_rate_limit_rules_algorithm CHECK (algorithm IN ('token_bucket', 'fixed_window', 'sliding_window'))
);

CREATE INDEX IF NOT EXISTS idx_rate_limit_rules_key ON ratelimit.rate_limit_rules (key);
CREATE INDEX IF NOT EXISTS idx_rate_limit_rules_enabled ON ratelimit.rate_limit_rules (enabled);

CREATE TRIGGER trigger_rate_limit_rules_update_updated_at
    BEFORE UPDATE ON ratelimit.rate_limit_rules
    FOR EACH ROW EXECUTE FUNCTION ratelimit.update_updated_at();

-- k1s0ユーザーへのアクセス権限付与（H-17 監査対応）
-- ratelimit スキーマへの DML 権限を k1s0_ratelimit_rw ロールに付与する
-- 注意: k1s0_ratelimit_rw ロールは 16-roles.sh に未定義のため、追加が必要（別担当エージェントが対応）
GRANT USAGE ON SCHEMA ratelimit TO k1s0_ratelimit_rw;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA ratelimit TO k1s0_ratelimit_rw;
ALTER DEFAULT PRIVILEGES IN SCHEMA ratelimit GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0_ratelimit_rw;
