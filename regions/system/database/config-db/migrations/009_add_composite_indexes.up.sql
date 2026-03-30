-- M-013 監査対応: config-db に頻繁に使用されるクエリパターン向けの複合インデックスを追加する

BEGIN;

-- config_entries の created_by によるフィルタリングクエリを最適化する
CREATE INDEX IF NOT EXISTS idx_config_entries_created_by
    ON config.config_entries (created_by);

COMMIT;
