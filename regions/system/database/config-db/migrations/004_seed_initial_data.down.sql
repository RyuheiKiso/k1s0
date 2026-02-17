-- 初期データのロールバック
-- system Tier の設定エントリをすべて削除

DELETE FROM config.config_entries WHERE namespace LIKE 'system.%';
