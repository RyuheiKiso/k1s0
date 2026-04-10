-- app_registry の全テーブルにテナント分離を実装する。
-- CRITICAL-DB-001 監査対応: マルチテナント環境でテナント間データ漏洩を防止する。
-- apps / app_versions / download_stats の 3 テーブルに tenant_id + RLS を追加する。
-- 既存データは 'system' テナントとしてバックフィルし、新規挿入を強制する。

BEGIN;

SET LOCAL search_path TO app_registry, public;

-- apps テーブルに tenant_id カラムを追加する
ALTER TABLE app_registry.apps
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除し、新規挿入時の明示指定を強制する
ALTER TABLE app_registry.apps
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加してクエリ性能を確保する
CREATE INDEX IF NOT EXISTS idx_apps_tenant_id
    ON app_registry.apps (tenant_id);

-- apps テーブルの RLS を有効化する
ALTER TABLE app_registry.apps ENABLE ROW LEVEL SECURITY;
ALTER TABLE app_registry.apps FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する（AS RESTRICTIVE + WITH CHECK で INSERT/UPDATE/SELECT を保護）
DROP POLICY IF EXISTS tenant_isolation ON app_registry.apps;
CREATE POLICY tenant_isolation ON app_registry.apps
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- app_versions テーブルに tenant_id カラムを追加する
ALTER TABLE app_registry.app_versions
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除する
ALTER TABLE app_registry.app_versions
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加する
CREATE INDEX IF NOT EXISTS idx_app_versions_tenant_id
    ON app_registry.app_versions (tenant_id);

-- app_versions テーブルの RLS を有効化する
ALTER TABLE app_registry.app_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE app_registry.app_versions FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する
DROP POLICY IF EXISTS tenant_isolation ON app_registry.app_versions;
CREATE POLICY tenant_isolation ON app_registry.app_versions
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- download_stats テーブルに tenant_id カラムを追加する
ALTER TABLE app_registry.download_stats
    ADD COLUMN IF NOT EXISTS tenant_id TEXT NOT NULL DEFAULT 'system';

-- バックフィル後はデフォルト値を削除する
ALTER TABLE app_registry.download_stats
    ALTER COLUMN tenant_id DROP DEFAULT;

-- tenant_id のインデックスを追加する
CREATE INDEX IF NOT EXISTS idx_download_stats_tenant_id
    ON app_registry.download_stats (tenant_id);

-- download_stats テーブルの RLS を有効化する
ALTER TABLE app_registry.download_stats ENABLE ROW LEVEL SECURITY;
ALTER TABLE app_registry.download_stats FORCE ROW LEVEL SECURITY;

-- テナント分離ポリシーを設定する
DROP POLICY IF EXISTS tenant_isolation ON app_registry.download_stats;
CREATE POLICY tenant_isolation ON app_registry.download_stats
    AS RESTRICTIVE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

COMMIT;
