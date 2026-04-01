-- STATIC-HIGH-002 ロールバック: 暗号化カラムを削除する
-- 注意: is_encrypted = true のレコードがある場合、value_json が空になっているため
-- ロールバック前に手動でのデータ復元が必要。

DROP INDEX IF EXISTS config.idx_config_entries_is_encrypted;

ALTER TABLE config.config_entries
    DROP COLUMN IF EXISTS encrypted_value,
    DROP COLUMN IF EXISTS is_encrypted;
