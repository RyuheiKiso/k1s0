-- Fix: config_change_logs の version カラムに DEFAULT値と NOT NULL制約を追加
-- 既存NULLデータを先に修正してから制約を追加

UPDATE config.config_change_logs SET old_version = 0 WHERE old_version IS NULL;
UPDATE config.config_change_logs SET new_version = 0 WHERE new_version IS NULL;

ALTER TABLE config.config_change_logs
    ALTER COLUMN old_version SET DEFAULT 0,
    ALTER COLUMN old_version SET NOT NULL,
    ALTER COLUMN new_version SET DEFAULT 0,
    ALTER COLUMN new_version SET NOT NULL;
