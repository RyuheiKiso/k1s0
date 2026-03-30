-- 011 のロールバック: encrypted_config カラムを削除する
BEGIN;

ALTER TABLE notification.channels DROP COLUMN IF EXISTS encrypted_config;

COMMIT;
