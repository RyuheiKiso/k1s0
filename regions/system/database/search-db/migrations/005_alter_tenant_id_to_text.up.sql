-- search_indices / search_documents テーブルの tenant_id を VARCHAR(255) から TEXT 型に変更する。
-- CRITICAL-DB-002 監査対応: 全サービスで tenant_id を TEXT 型に統一する（ADR-0093）。
-- HIGH-DB-007: search_indices の UNIQUE(name) を UNIQUE(tenant_id, name) に変更する。
-- search-db 004 では AS RESTRICTIVE + WITH CHECK が欠落していたため、同時に修正する。
-- 既存の RLS ポリシーを DROP してから型変更し、完全なポリシーで再作成する。

BEGIN;

SET LOCAL search_path TO search, public;

-- 既存の RLS ポリシーを先に削除する
DROP POLICY IF EXISTS tenant_isolation ON search.search_indices;
DROP POLICY IF EXISTS tenant_isolation ON search.search_documents;

-- search_indices テーブルの tenant_id を TEXT 型に変更する
ALTER TABLE search.search_indices
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;

-- search_documents テーブルの tenant_id を TEXT 型に変更する
ALTER TABLE search.search_documents
    ALTER COLUMN tenant_id TYPE TEXT USING tenant_id::TEXT;

-- search_indices テーブルの UNIQUE(name) を UNIQUE(tenant_id, name) に変更する（HIGH-DB-007 対応）
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'search_indices_name_key' AND conrelid = 'search.search_indices'::regclass
    ) THEN
        ALTER TABLE search.search_indices DROP CONSTRAINT search_indices_name_key;
    END IF;
END $$;
-- 既存の name インデックスを削除してテナントスコープの UNIQUE INDEX を作成する
DROP INDEX IF EXISTS search.idx_search_indices_name;
CREATE UNIQUE INDEX IF NOT EXISTS uq_search_indices_tenant_name
    ON search.search_indices (tenant_id, name);

-- search_indices テーブルのポリシーを AS RESTRICTIVE + WITH CHECK 付きで再作成する
CREATE POLICY tenant_isolation ON search.search_indices
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- search_documents テーブルのポリシーを AS RESTRICTIVE + WITH CHECK 付きで再作成する
CREATE POLICY tenant_isolation ON search.search_documents
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
