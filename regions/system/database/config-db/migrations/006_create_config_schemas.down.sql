-- config-db: config_schemas テーブル削除

DROP TRIGGER IF EXISTS trigger_config_schemas_update_updated_at ON config.config_schemas;
DROP TABLE IF EXISTS config.config_schemas;
