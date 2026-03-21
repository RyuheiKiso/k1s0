-- 007_improve_quota_policies.up.sql の変更をロールバックする。
-- subject_id のデフォルト空文字列を復元し、period の CHECK 制約を元の状態に戻す。
BEGIN;

-- period の CHECK 制約を元の 'daily'/'monthly' のみに戻す
ALTER TABLE quota.quota_policies DROP CONSTRAINT IF EXISTS chk_quota_policies_period;
ALTER TABLE quota.quota_policies ADD CONSTRAINT chk_quota_policies_period
    CHECK (period IN ('daily', 'monthly'));

-- subject_id のデフォルト値を空文字列に戻す
ALTER TABLE quota.quota_policies ALTER COLUMN subject_id DROP DEFAULT;
ALTER TABLE quota.quota_policies ALTER COLUMN subject_id SET DEFAULT '';

COMMIT;
