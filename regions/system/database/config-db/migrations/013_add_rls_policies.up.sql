-- HIGH-005対応: config-db テナント分離のための RLS ポリシーを追加する
-- config_entries と config_change_logs は tenant_id カラムを持つため
-- RLS を有効化してテナント間のデータ漏洩を防ぐ

SET LOCAL search_path TO config, public;

-- config_entries に RLS を有効化する
-- FORCE ROW LEVEL SECURITY によりテーブルオーナーにも RLS を適用する
ALTER TABLE config.config_entries ENABLE ROW LEVEL SECURITY;
ALTER TABLE config.config_entries FORCE ROW LEVEL SECURITY;

-- config_entries のテナント分離ポリシー
-- RESTRICTIVE ポリシーにより他の PERMISSIVE ポリシーと AND 結合される
-- current_setting の第2引数 true は設定が存在しない場合に NULL を返す（エラー回避）
CREATE POLICY tenant_isolation ON config.config_entries
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- config_change_logs に RLS を有効化する
ALTER TABLE config.config_change_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE config.config_change_logs FORCE ROW LEVEL SECURITY;

-- config_change_logs のテナント分離ポリシー
CREATE POLICY tenant_isolation ON config.config_change_logs
    AS RESTRICTIVE
    USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
    WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);

-- service_config_mappings テーブルも tenant_id を持つ場合は RLS を適用する
-- （migration 005 で作成、tenant_id の有無を条件付きで処理）
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'config'
          AND table_name   = 'service_config_mappings'
          AND column_name  = 'tenant_id'
    ) THEN
        ALTER TABLE config.service_config_mappings ENABLE ROW LEVEL SECURITY;
        ALTER TABLE config.service_config_mappings FORCE ROW LEVEL SECURITY;

        -- service_config_mappings のテナント分離ポリシー（既存ポリシーがない場合のみ作成）
        IF NOT EXISTS (
            SELECT 1 FROM pg_policies
            WHERE schemaname = 'config'
              AND tablename  = 'service_config_mappings'
              AND policyname = 'tenant_isolation'
        ) THEN
            CREATE POLICY tenant_isolation ON config.service_config_mappings
                AS RESTRICTIVE
                USING (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT)
                WITH CHECK (tenant_id::TEXT = current_setting('app.current_tenant_id', true)::TEXT);
        END IF;
    END IF;
END $$;
