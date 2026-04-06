-- search_indices / search_documents の tenant_id を TEXT から VARCHAR(255) に戻す。

BEGIN;

SET LOCAL search_path TO search, public;

DROP POLICY IF EXISTS tenant_isolation ON search.search_indices;
DROP POLICY IF EXISTS tenant_isolation ON search.search_documents;

-- テナントスコープ UNIQUE INDEX を削除し、元の name UNIQUE インデックスを復元する
DROP INDEX IF EXISTS search.uq_search_indices_tenant_name;
CREATE UNIQUE INDEX IF NOT EXISTS idx_search_indices_name ON search.search_indices (name);

ALTER TABLE search.search_indices
    ALTER COLUMN tenant_id TYPE VARCHAR(255) USING tenant_id::VARCHAR(255);

ALTER TABLE search.search_documents
    ALTER COLUMN tenant_id TYPE VARCHAR(255) USING tenant_id::VARCHAR(255);

-- 004 マイグレーション相当のポリシーを復元する（RESTRICTIVE なし、WITH CHECK なし）
CREATE POLICY tenant_isolation ON search.search_indices
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

CREATE POLICY tenant_isolation ON search.search_documents
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

COMMIT;
