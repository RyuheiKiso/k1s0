-- C-005 監査対応: notification.channels.config の機密情報（SMTP パスワード、Webhook URL、API キー等）を暗号化する
-- AES-256-GCM 暗号化済みデータを encrypted_config TEXT カラムに保存する
-- dual-read 移行戦略: encrypted_config が NULL でない行は暗号化データを優先使用し、NULL の行は config JSONB にフォールバックする

BEGIN;

-- 暗号化済み設定値を格納するカラムを追加する（nonce は Base64 エンコード時に先頭12バイトとして埋め込む）
ALTER TABLE notification.channels
    ADD COLUMN encrypted_config TEXT;

COMMENT ON COLUMN notification.channels.encrypted_config IS
    'AES-256-GCM 暗号化済み設定値（Base64: nonce[12B] || ciphertext）。NULL の場合は config JSONB を使用（移行期間中のみ）。NOTIFICATION_CHANNEL_ENCRYPTION_KEY が設定されている場合は必須。';

COMMIT;
