ALTER TABLE config.config_change_logs
ADD COLUMN old_version INTEGER,
ADD COLUMN new_version INTEGER;
