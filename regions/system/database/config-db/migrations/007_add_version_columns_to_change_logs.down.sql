ALTER TABLE config.config_change_logs
DROP COLUMN IF EXISTS old_version,
DROP COLUMN IF EXISTS new_version;
