-- べき等性ガード: カラム追加が重複実行されても安全に処理する

-- RBAC ロールカラムを table_definitions に追加
ALTER TABLE master_maintenance.table_definitions
    ADD COLUMN IF NOT EXISTS read_roles TEXT[] NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS write_roles TEXT[] NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS admin_roles TEXT[] NOT NULL DEFAULT '{}';
