-- vault の全テーブルにテナント分離を実装する。
-- CRITICAL-DB-001 監査対応: マルチテナント環境でテナント間データ漏洩を防止する。
-- secrets / secret_versions / access_logs / access_policies の全テーブルに tenant_id + RLS を追加する。
-- secrets テーブルの UNIQUE(key_path) は UNIQUE(tenant_id, key_path) に変更して、テナント間の重複を許可する。
-- 既存データは 'system' テナントとしてバックフィルし、新規挿入を強制する。

BEGIN;

SET LOCAL search_path TO vault, public;

-- secrets テーブルに tenant_id カラムを追加する
ALTER TABLE vault.secrets
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE vault.secrets
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_secrets_tenant_id
    ON vault.secrets (tenant_id);

-- 既存の key_path UNIQUE 制約を削除し、テナントスコープの UNIQUE 制約に変更する
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'secrets_key_path_key' AND conrelid = 'vault.secrets'::regclass
    ) THEN
        ALTER TABLE vault.secrets DROP CONSTRAINT secrets_key_path_key;
    END IF;
END $$;
DROP INDEX IF EXISTS vault.idx_secrets_key_path;
CREATE UNIQUE INDEX IF NOT EXISTS uq_secrets_tenant_key_path
    ON vault.secrets (tenant_id, key_path);

ALTER TABLE vault.secrets ENABLE ROW LEVEL SECURITY;
ALTER TABLE vault.secrets FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON vault.secrets;
CREATE POLICY tenant_isolation ON vault.secrets
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- secret_versions テーブルに tenant_id カラムを追加する（親テーブル secrets と整合性を保つ）
ALTER TABLE vault.secret_versions
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE vault.secret_versions
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_secret_versions_tenant_id
    ON vault.secret_versions (tenant_id);

ALTER TABLE vault.secret_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE vault.secret_versions FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON vault.secret_versions;
CREATE POLICY tenant_isolation ON vault.secret_versions
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- access_logs テーブルに tenant_id カラムを追加する
ALTER TABLE vault.access_logs
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE vault.access_logs
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_access_logs_tenant_id
    ON vault.access_logs (tenant_id);

ALTER TABLE vault.access_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE vault.access_logs FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON vault.access_logs;
CREATE POLICY tenant_isolation ON vault.access_logs
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- access_policies テーブルに tenant_id カラムを追加する
ALTER TABLE vault.access_policies
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

ALTER TABLE vault.access_policies
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_access_policies_tenant_id
    ON vault.access_policies (tenant_id);

ALTER TABLE vault.access_policies ENABLE ROW LEVEL SECURITY;
ALTER TABLE vault.access_policies FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON vault.access_policies;
CREATE POLICY tenant_isolation ON vault.access_policies
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
