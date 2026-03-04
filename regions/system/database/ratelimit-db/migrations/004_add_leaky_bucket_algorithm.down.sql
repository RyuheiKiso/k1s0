ALTER TABLE ratelimit.rate_limit_rules
    DROP CONSTRAINT IF EXISTS chk_ratelimit_rules_algorithm;

ALTER TABLE ratelimit.rate_limit_rules
    ADD CONSTRAINT chk_ratelimit_rules_algorithm
    CHECK (algorithm IN ('token_bucket', 'fixed_window', 'sliding_window'));
