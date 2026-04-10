-- vault の全テーブルのテナント分離を元に戻す。
-- RLS ポリシー・インデックス・tenant_id カラムを削除し、secrets の UNIQUE 制約を復元する。

BEGIN;

SET LOCAL search_path TO vault, public;

-- access_policies の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON vault.access_policies;
ALTER TABLE vault.access_policies DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS vault.idx_access_policies_tenant_id;
ALTER TABLE vault.access_policies DROP COLUMN IF EXISTS tenant_id;

-- access_logs の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON vault.access_logs;
ALTER TABLE vault.access_logs DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS vault.idx_access_logs_tenant_id;
ALTER TABLE vault.access_logs DROP COLUMN IF EXISTS tenant_id;

-- secret_versions の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON vault.secret_versions;
ALTER TABLE vault.secret_versions DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS vault.idx_secret_versions_tenant_id;
ALTER TABLE vault.secret_versions DROP COLUMN IF EXISTS tenant_id;

-- secrets の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON vault.secrets;
ALTER TABLE vault.secrets DISABLE ROW LEVEL SECURITY;

-- テナントスコープ UNIQUE INDEX を削除し、元の key_path UNIQUE インデックスを復元する
DROP INDEX IF EXISTS vault.uq_secrets_tenant_key_path;
CREATE UNIQUE INDEX IF NOT EXISTS idx_secrets_key_path ON vault.secrets (key_path);

DROP INDEX IF EXISTS vault.idx_secrets_tenant_id;
ALTER TABLE vault.secrets DROP COLUMN IF EXISTS tenant_id;

COMMIT;
