-- search_indices と search_documents に tenant_id カラムを追加し、RLS でテナント分離を実現する。
-- CRIT-005 監査対応: マルチテナント SaaS として他テナントのデータ参照を防ぐ。
-- 既存データは tenant_id = 'system' でバックフィルし、その後 DEFAULT を削除して NOT NULL を維持する。

BEGIN;

-- search_indices テーブルに tenant_id カラムを追加する（既存データのバックフィルとして 'system' をデフォルト値とする）
ALTER TABLE search.search_indices
    ADD COLUMN IF NOT EXISTS tenant_id VARCHAR(255) NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除し、新規挿入時の明示指定を強制する
ALTER TABLE search.search_indices
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加してクエリ性能を確保する
CREATE INDEX IF NOT EXISTS idx_search_indices_tenant_id
    ON search.search_indices (tenant_id);

-- テナントと名前の複合インデックスを追加する（テナント横断クエリの高速化）
CREATE INDEX IF NOT EXISTS idx_search_indices_tenant_name
    ON search.search_indices (tenant_id, name);

-- search_documents テーブルに tenant_id カラムを追加する
ALTER TABLE search.search_documents
    ADD COLUMN IF NOT EXISTS tenant_id VARCHAR(255) NOT NULL DEFAULT 'system';

ALTER TABLE search.search_documents
    ALTER COLUMN tenant_id DROP DEFAULT;

CREATE INDEX IF NOT EXISTS idx_search_documents_tenant_id
    ON search.search_documents (tenant_id);

-- search_indices テーブルの RLS を有効化する
ALTER TABLE search.search_indices ENABLE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する（app.current_tenant_id セッション変数でフィルタリング）
DROP POLICY IF EXISTS tenant_isolation ON search.search_indices;
CREATE POLICY tenant_isolation ON search.search_indices
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- search_documents テーブルの RLS を有効化する
ALTER TABLE search.search_documents ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON search.search_documents;
CREATE POLICY tenant_isolation ON search.search_documents
    USING (tenant_id = current_setting('app.current_tenant_id', true)::TEXT);

-- スーパーユーザー・オーナーロールも RLS の適用対象とする（バイパスを防止）
ALTER TABLE search.search_indices FORCE ROW LEVEL SECURITY;
ALTER TABLE search.search_documents FORCE ROW LEVEL SECURITY;

COMMIT;
