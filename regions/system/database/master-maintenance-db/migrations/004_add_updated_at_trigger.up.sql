-- updated_at 自動更新トリガー関数（べき等性あり: CREATE OR REPLACE）
CREATE OR REPLACE FUNCTION master_maintenance.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- べき等性ガード: トリガーが既に存在する場合は削除してから再作成する
DROP TRIGGER IF EXISTS trg_table_definitions_updated_at ON master_maintenance.table_definitions;
DROP TRIGGER IF EXISTS trg_column_definitions_updated_at ON master_maintenance.column_definitions;
DROP TRIGGER IF EXISTS trg_consistency_rules_updated_at ON master_maintenance.consistency_rules;
DROP TRIGGER IF EXISTS trg_display_configs_updated_at ON master_maintenance.display_configs;

-- table_definitions テーブルのトリガー
CREATE TRIGGER trg_table_definitions_updated_at
    BEFORE UPDATE ON master_maintenance.table_definitions
    FOR EACH ROW
    EXECUTE FUNCTION master_maintenance.update_updated_at();

-- column_definitions テーブルのトリガー
CREATE TRIGGER trg_column_definitions_updated_at
    BEFORE UPDATE ON master_maintenance.column_definitions
    FOR EACH ROW
    EXECUTE FUNCTION master_maintenance.update_updated_at();

-- consistency_rules テーブルのトリガー
CREATE TRIGGER trg_consistency_rules_updated_at
    BEFORE UPDATE ON master_maintenance.consistency_rules
    FOR EACH ROW
    EXECUTE FUNCTION master_maintenance.update_updated_at();

-- display_configs テーブルのトリガー
CREATE TRIGGER trg_display_configs_updated_at
    BEFORE UPDATE ON master_maintenance.display_configs
    FOR EACH ROW
    EXECUTE FUNCTION master_maintenance.update_updated_at();
