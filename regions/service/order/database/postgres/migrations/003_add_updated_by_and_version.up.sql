-- べき等性ガード: カラム追加が重複実行されても安全に処理する
SET search_path TO order_service;

DO $$ BEGIN
    -- updated_by カラムの追加
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'order_service' AND table_name = 'orders' AND column_name = 'updated_by'
    ) THEN
        ALTER TABLE orders ADD COLUMN updated_by VARCHAR(255);
    END IF;

    -- version カラムの追加
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'order_service' AND table_name = 'orders' AND column_name = 'version'
    ) THEN
        ALTER TABLE orders ADD COLUMN version INT NOT NULL DEFAULT 1;
    END IF;
END $$;
