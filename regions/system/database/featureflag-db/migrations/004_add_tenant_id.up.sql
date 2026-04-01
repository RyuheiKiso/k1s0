-- STATIC-CRITICAL-001 監査対応: feature_flags にテナント分離カラムを追加する
-- 既存データには システムテナント UUID (00000000-0000-0000-0000-000000000001) を割り当てる

ALTER TABLE featureflag.feature_flags
    ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001';

ALTER TABLE featureflag.flag_audit_logs
    ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001';

-- 既存の UNIQUE 制約を削除して tenant_id を含む新しい制約に変更する
ALTER TABLE featureflag.feature_flags
    DROP CONSTRAINT IF EXISTS feature_flags_flag_key_key;

ALTER TABLE featureflag.feature_flags
    ADD CONSTRAINT uq_feature_flags_tenant_flag_key UNIQUE (tenant_id, flag_key);

-- テナントIDによる高速検索のためのインデックスを追加する
CREATE INDEX IF NOT EXISTS idx_feature_flags_tenant_id
    ON featureflag.feature_flags (tenant_id);

CREATE INDEX IF NOT EXISTS idx_flag_audit_logs_tenant_id
    ON featureflag.flag_audit_logs (tenant_id);

-- ADD COLUMN 後は DEFAULT 制約を削除し、今後のINSERTで明示的に指定させる
ALTER TABLE featureflag.feature_flags ALTER COLUMN tenant_id DROP DEFAULT;
ALTER TABLE featureflag.flag_audit_logs ALTER COLUMN tenant_id DROP DEFAULT;
