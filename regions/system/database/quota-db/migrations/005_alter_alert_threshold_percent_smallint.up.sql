ALTER TABLE quota.quota_policies
    ALTER COLUMN alert_threshold_percent TYPE SMALLINT
    USING alert_threshold_percent::SMALLINT;

ALTER TABLE quota.quota_policies
    DROP CONSTRAINT IF EXISTS chk_quota_policies_alert_threshold_percent;

ALTER TABLE quota.quota_policies
    ADD CONSTRAINT chk_quota_policies_alert_threshold_percent
    CHECK (alert_threshold_percent BETWEEN 0 AND 100);
