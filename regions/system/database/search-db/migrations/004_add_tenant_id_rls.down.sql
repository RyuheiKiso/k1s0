-- 004_add_tenant_id_rls.up.sql のロールバック: RLS ポリシーと tenant_id カラムを削除する

BEGIN;

-- search_indices の RLS を無効化する
ALTER TABLE search.search_indices DISABLE ROW LEVEL SECURITY;
ALTER TABLE search.search_indices NO FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON search.search_indices;

-- search_documents の RLS を無効化する
ALTER TABLE search.search_documents DISABLE ROW LEVEL SECURITY;
ALTER TABLE search.search_documents NO FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON search.search_documents;

-- インデックスを削除する
DROP INDEX IF EXISTS search.idx_search_indices_tenant_id;
DROP INDEX IF EXISTS search.idx_search_indices_tenant_name;
DROP INDEX IF EXISTS search.idx_search_documents_tenant_id;

-- tenant_id カラムを削除する
ALTER TABLE search.search_indices DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE search.search_documents DROP COLUMN IF EXISTS tenant_id;

COMMIT;
