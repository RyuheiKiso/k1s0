-- 018_add_api_keys_rls のロールバック: RLS ポリシーと設定を削除する
BEGIN;
DROP POLICY IF EXISTS tenant_isolation ON auth.api_keys;
ALTER TABLE auth.api_keys DISABLE ROW LEVEL SECURITY;
ALTER TABLE auth.api_keys NO FORCE ROW LEVEL SECURITY;
COMMIT;
