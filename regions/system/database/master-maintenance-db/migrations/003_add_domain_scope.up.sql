-- べき等性ガード: カラム追加が重複実行されても安全に処理する

-- table_definitions に domain_scope カラムを追加
ALTER TABLE master_maintenance.table_definitions
    ADD COLUMN IF NOT EXISTS domain_scope VARCHAR(100) DEFAULT NULL;

-- name の一意制約を name + domain_scope の複合一意インデックスに変更
DO $$ BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'table_definitions_name_key'
    ) THEN
        ALTER TABLE master_maintenance.table_definitions
            DROP CONSTRAINT table_definitions_name_key;
    END IF;
END $$;

-- 複合一意インデックスを作成（べき等性あり）
CREATE UNIQUE INDEX IF NOT EXISTS uq_table_definitions_name_domain
    ON master_maintenance.table_definitions (name, COALESCE(domain_scope, '__system__'));

-- domain_scope フィルタリング用インデックスを作成（べき等性あり）
CREATE INDEX IF NOT EXISTS idx_table_definitions_domain_scope
    ON master_maintenance.table_definitions(domain_scope);

-- change_logs に domain_scope カラムを追加
ALTER TABLE master_maintenance.change_logs
    ADD COLUMN IF NOT EXISTS domain_scope VARCHAR(100) DEFAULT NULL;

-- change_logs の domain_scope フィルタリング用インデックスを作成（べき等性あり）
CREATE INDEX IF NOT EXISTS idx_change_logs_domain_scope
    ON master_maintenance.change_logs(domain_scope);
