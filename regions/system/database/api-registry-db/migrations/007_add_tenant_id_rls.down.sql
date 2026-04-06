-- api_schemas と api_schema_versions のテナント分離を元に戻す。
-- RLS ポリシー・インデックス・tenant_id カラムを削除する。

BEGIN;

SET LOCAL search_path TO apiregistry, public;

-- api_schema_versions の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON apiregistry.api_schema_versions;
ALTER TABLE apiregistry.api_schema_versions DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS apiregistry.idx_api_schema_versions_tenant_id;
ALTER TABLE apiregistry.api_schema_versions DROP COLUMN IF EXISTS tenant_id;

-- api_schemas の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON apiregistry.api_schemas;
ALTER TABLE apiregistry.api_schemas DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS apiregistry.idx_api_schemas_tenant_name;
DROP INDEX IF EXISTS apiregistry.idx_api_schemas_tenant_id;
ALTER TABLE apiregistry.api_schemas DROP COLUMN IF EXISTS tenant_id;

COMMIT;
