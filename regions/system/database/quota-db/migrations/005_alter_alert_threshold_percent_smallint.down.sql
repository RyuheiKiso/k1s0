ALTER TABLE quota.quota_policies
    DROP CONSTRAINT IF EXISTS chk_quota_policies_alert_threshold_percent;

ALTER TABLE quota.quota_policies
    ALTER COLUMN alert_threshold_percent TYPE DOUBLE PRECISION
    USING alert_threshold_percent::DOUBLE PRECISION;
