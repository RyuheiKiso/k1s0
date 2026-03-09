-- updated_at 自動更新トリガー関数
CREATE OR REPLACE FUNCTION master_maintenance.update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

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
