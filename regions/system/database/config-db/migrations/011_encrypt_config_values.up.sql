-- STATIC-HIGH-002: config_entries に暗号化カラムを追加する
-- AES-256-GCM で機密設定値（system.auth.*, system.database.* 等）を暗号化保存するための準備。
-- is_encrypted = true の場合、value_json は空 JSON、encrypted_value に暗号文（base64）が入る。

ALTER TABLE config.config_entries
    ADD COLUMN IF NOT EXISTS encrypted_value TEXT,
    ADD COLUMN IF NOT EXISTS is_encrypted    BOOLEAN NOT NULL DEFAULT false;

-- 暗号化フラグでのフィルタリング用インデックス
CREATE INDEX IF NOT EXISTS idx_config_entries_is_encrypted
    ON config.config_entries (is_encrypted)
    WHERE is_encrypted = true;
