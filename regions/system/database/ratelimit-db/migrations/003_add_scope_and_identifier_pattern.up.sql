ALTER TABLE ratelimit.rate_limit_rules
    ADD COLUMN IF NOT EXISTS scope VARCHAR(50),
    ADD COLUMN IF NOT EXISTS identifier_pattern VARCHAR(255);

UPDATE ratelimit.rate_limit_rules
SET
    scope = COALESCE(NULLIF(scope, ''), NULLIF(name, ''), 'service'),
    identifier_pattern = COALESCE(
        NULLIF(identifier_pattern, ''),
        CASE
            WHEN key LIKE '%:%' THEN split_part(key, ':', 2)
            ELSE key
        END,
        '*'
    );

ALTER TABLE ratelimit.rate_limit_rules
    ALTER COLUMN scope SET DEFAULT 'service',
    ALTER COLUMN scope SET NOT NULL,
    ALTER COLUMN identifier_pattern SET DEFAULT '*',
    ALTER COLUMN identifier_pattern SET NOT NULL,
    ALTER COLUMN key DROP NOT NULL;

CREATE INDEX IF NOT EXISTS idx_ratelimit_rules_scope_identifier
    ON ratelimit.rate_limit_rules (scope, identifier_pattern);
