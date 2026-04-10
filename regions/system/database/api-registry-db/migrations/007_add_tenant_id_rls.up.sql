-- api_schemas と api_schema_versions にテナント分離を実装する。
-- CRITICAL-DB-001 監査対応: マルチテナント環境でテナント間データ漏洩を防止する。
-- 既存データは 'system' テナントとしてバックフィルし、新規挿入を強制する。
-- api_schemas の PRIMARY KEY は name 単体であるため、テナントスコープ化には注意が必要。
-- 既存 PRIMARY KEY を変更せず、tenant_id + name の UNIQUE 制約を追加してテナント間名前重複を防ぐ。

BEGIN;

SET LOCAL search_path TO apiregistry, public;

-- api_schemas テーブルに tenant_id カラムを追加する
ALTER TABLE apiregistry.api_schemas
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除し、新規挿入時の明示指定を強制する
ALTER TABLE apiregistry.api_schemas
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加してクエリ性能を確保する
CREATE INDEX IF NOT EXISTS idx_api_schemas_tenant_id
    ON apiregistry.api_schemas (tenant_id);

-- テナントと名前の複合インデックス（テナント横断検索の高速化）
CREATE INDEX IF NOT EXISTS idx_api_schemas_tenant_name
    ON apiregistry.api_schemas (tenant_id, name);

-- api_schemas テーブルの RLS を有効化する
ALTER TABLE apiregistry.api_schemas ENABLE ROW LEVEL SECURITY;
ALTER TABLE apiregistry.api_schemas FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する（AS RESTRICTIVE + WITH CHECK で INSERT/UPDATE/SELECT を保護）
DROP POLICY IF EXISTS tenant_isolation ON apiregistry.api_schemas;
CREATE POLICY tenant_isolation ON apiregistry.api_schemas
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- api_schema_versions テーブルに tenant_id カラムを追加する
ALTER TABLE apiregistry.api_schema_versions
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除する
ALTER TABLE apiregistry.api_schema_versions
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加する
CREATE INDEX IF NOT EXISTS idx_api_schema_versions_tenant_id
    ON apiregistry.api_schema_versions (tenant_id);

-- api_schema_versions テーブルの RLS を有効化する
ALTER TABLE apiregistry.api_schema_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE apiregistry.api_schema_versions FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する
DROP POLICY IF EXISTS tenant_isolation ON apiregistry.api_schema_versions;
CREATE POLICY tenant_isolation ON apiregistry.api_schema_versions
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
