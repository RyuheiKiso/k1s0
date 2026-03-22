-- べき等性ガード: カラム追加が重複実行されても安全に処理する
SET search_path TO task_service;

DO $$ BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'task_service' AND table_name = 'tasks' AND column_name = 'updated_by'
    ) THEN
        ALTER TABLE tasks ADD COLUMN updated_by VARCHAR(255);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'task_service' AND table_name = 'tasks' AND column_name = 'version'
    ) THEN
        ALTER TABLE tasks ADD COLUMN version INT NOT NULL DEFAULT 1;
    END IF;
END $$;
