-- STATIC-CRITICAL-001 監査対応: テナント分離カラムのロールバック

-- インデックスを削除する
DROP INDEX IF EXISTS featureflag.idx_flag_audit_logs_tenant_id;
DROP INDEX IF EXISTS featureflag.idx_feature_flags_tenant_id;

-- 新規 UNIQUE 制約を削除して元の制約に戻す
ALTER TABLE featureflag.feature_flags
    DROP CONSTRAINT IF EXISTS uq_feature_flags_tenant_flag_key;

ALTER TABLE featureflag.feature_flags
    ADD CONSTRAINT feature_flags_flag_key_key UNIQUE (flag_key);

-- tenant_id カラムを削除する
ALTER TABLE featureflag.feature_flags DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE featureflag.flag_audit_logs DROP COLUMN IF EXISTS tenant_id;
