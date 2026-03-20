-- べき等性ガード: カラム追加が重複実行されても安全に処理する
ALTER TABLE config.config_change_logs
    ADD COLUMN IF NOT EXISTS old_version INTEGER,
    ADD COLUMN IF NOT EXISTS new_version INTEGER;
