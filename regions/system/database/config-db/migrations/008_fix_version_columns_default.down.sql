ALTER TABLE config.config_change_logs
    ALTER COLUMN old_version DROP DEFAULT,
    ALTER COLUMN old_version DROP NOT NULL,
    ALTER COLUMN new_version DROP DEFAULT,
    ALTER COLUMN new_version DROP NOT NULL;
