-- べき等性ガード: カラム追加が重複実行されても安全に処理する
-- 共通 outbox ライブラリとの整合性を確保するために idempotency_key カラムを追加する
DO $$ BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'outbox_events' AND column_name = 'idempotency_key'
    ) THEN
        -- idempotency_key カラムを追加
        ALTER TABLE outbox_events ADD COLUMN idempotency_key VARCHAR(255);

        -- 既存行にはUUIDをデフォルト値として設定
        UPDATE outbox_events SET idempotency_key = gen_random_uuid()::text WHERE idempotency_key IS NULL;

        -- NOT NULL 制約を追加
        ALTER TABLE outbox_events ALTER COLUMN idempotency_key SET NOT NULL;
    END IF;
END $$;

-- UNIQUE インデックスを追加（ON CONFLICT 句で使用）
CREATE UNIQUE INDEX IF NOT EXISTS idx_outbox_events_idempotency_key ON outbox_events (idempotency_key);
