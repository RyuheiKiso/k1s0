-- app_registry の全テーブルのテナント分離を元に戻す。
-- RLS ポリシー・インデックス・tenant_id カラムを削除する。

BEGIN;

SET LOCAL search_path TO app_registry, public;

-- download_stats の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON app_registry.download_stats;
ALTER TABLE app_registry.download_stats DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS app_registry.idx_download_stats_tenant_id;
ALTER TABLE app_registry.download_stats DROP COLUMN IF EXISTS tenant_id;

-- app_versions の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON app_registry.app_versions;
ALTER TABLE app_registry.app_versions DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS app_registry.idx_app_versions_tenant_id;
ALTER TABLE app_registry.app_versions DROP COLUMN IF EXISTS tenant_id;

-- apps の RLS 無効化とポリシー削除
DROP POLICY IF EXISTS tenant_isolation ON app_registry.apps;
ALTER TABLE app_registry.apps DISABLE ROW LEVEL SECURITY;
DROP INDEX IF EXISTS app_registry.idx_apps_tenant_id;
ALTER TABLE app_registry.apps DROP COLUMN IF EXISTS tenant_id;

COMMIT;
