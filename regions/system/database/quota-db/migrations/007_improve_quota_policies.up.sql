-- quota_policies テーブルの設計を改善する。
-- subject_id のデフォルト空文字列を除去し、period の選択肢を拡張する。
BEGIN;

-- subject_id のデフォルト空文字列を除去（NULL を明示的にする）
ALTER TABLE quota.quota_policies ALTER COLUMN subject_id DROP DEFAULT;
ALTER TABLE quota.quota_policies ALTER COLUMN subject_id SET DEFAULT NULL;

-- period の CHECK 制約を拡張して weekly/yearly を追加
ALTER TABLE quota.quota_policies DROP CONSTRAINT IF EXISTS chk_quota_policies_period;
ALTER TABLE quota.quota_policies ADD CONSTRAINT chk_quota_policies_period
    CHECK (period IN ('daily', 'weekly', 'monthly', 'yearly'));

COMMIT;
