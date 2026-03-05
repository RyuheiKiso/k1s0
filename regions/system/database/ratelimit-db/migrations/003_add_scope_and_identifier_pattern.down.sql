DROP INDEX IF EXISTS ratelimit.idx_ratelimit_rules_scope_identifier;

UPDATE ratelimit.rate_limit_rules
SET key = COALESCE(NULLIF(key, ''), NULLIF(identifier_pattern, ''), NULLIF(name, ''), '*');

ALTER TABLE ratelimit.rate_limit_rules
    ALTER COLUMN key SET NOT NULL;

ALTER TABLE ratelimit.rate_limit_rules
    DROP COLUMN IF EXISTS identifier_pattern,
    DROP COLUMN IF EXISTS scope;
