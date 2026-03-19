-- Outbox イベントにべき等キーを追加（H-6 対応）
-- 共通 outbox ライブラリとの整合性を確保するために idempotency_key カラムを追加する
ALTER TABLE outbox_events ADD COLUMN idempotency_key VARCHAR(255);

-- 既存行にはUUIDをデフォルト値として設定
UPDATE outbox_events SET idempotency_key = gen_random_uuid()::text WHERE idempotency_key IS NULL;

-- NOT NULL 制約を追加
ALTER TABLE outbox_events ALTER COLUMN idempotency_key SET NOT NULL;

-- UNIQUE インデックスを追加（ON CONFLICT 句で使用）
CREATE UNIQUE INDEX IF NOT EXISTS idx_outbox_events_idempotency_key ON outbox_events (idempotency_key);
