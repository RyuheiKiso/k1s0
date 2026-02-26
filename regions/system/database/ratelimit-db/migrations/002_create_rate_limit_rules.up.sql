CREATE TABLE IF NOT EXISTS ratelimit.rate_limit_rules (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(255) NOT NULL UNIQUE,
    key         TEXT         NOT NULL,
    limit_count BIGINT       NOT NULL,
    window_secs BIGINT       NOT NULL,
    algorithm   VARCHAR(50)  NOT NULL DEFAULT 'token_bucket',
    enabled     BOOLEAN      NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_ratelimit_rules_algorithm CHECK (algorithm IN ('token_bucket', 'fixed_window', 'sliding_window')),
    CONSTRAINT chk_ratelimit_rules_limit_count CHECK (limit_count > 0),
    CONSTRAINT chk_ratelimit_rules_window_secs CHECK (window_secs > 0)
);

CREATE INDEX IF NOT EXISTS idx_ratelimit_rules_name ON ratelimit.rate_limit_rules (name);
CREATE INDEX IF NOT EXISTS idx_ratelimit_rules_enabled ON ratelimit.rate_limit_rules (enabled) WHERE enabled = true;

CREATE TRIGGER trigger_ratelimit_rules_update_updated_at
    BEFORE UPDATE ON ratelimit.rate_limit_rules
    FOR EACH ROW
    EXECUTE FUNCTION ratelimit.update_updated_at();
