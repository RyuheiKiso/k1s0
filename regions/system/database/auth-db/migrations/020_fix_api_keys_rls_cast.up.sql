-- api_keys テーブルの RLS ポリシーに ::TEXT キャストを追加し、
-- tenant_id 型 (TEXT) と current_setting() の TEXT 型を明示的に統一する。
-- 019 で prefix 長が拡張されたため、本マイグレーションは 020 として適用する。
BEGIN;

DROP POLICY IF EXISTS tenant_isolation ON auth.api_keys;
CREATE POLICY tenant_isolation ON auth.api_keys
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
