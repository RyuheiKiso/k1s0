-- べき等性ガード: カラム追加が重複実行されても安全に処理する
DO $$ BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'apiregistry'
          AND table_name = 'api_schema_versions'
          AND column_name = 'breaking_change_details'
    ) THEN
        ALTER TABLE apiregistry.api_schema_versions
            ADD COLUMN breaking_change_details JSONB NOT NULL DEFAULT '[]';
    END IF;
END $$;
