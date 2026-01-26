-- config-service: 設定テーブルの削除
-- 注意: このマイグレーションはデータを完全に削除します

-- トリガーの削除
DROP TRIGGER IF EXISTS tr_fw_m_setting_updated ON fw_m_setting;
DROP TRIGGER IF EXISTS tr_fw_m_setting_history ON fw_m_setting;

-- 関数の削除
DROP FUNCTION IF EXISTS fw_setting_history();
-- fw_update_timestamp は他のサービスでも使用される可能性があるため残す

-- テーブルの削除
DROP TABLE IF EXISTS fw_h_setting CASCADE;
DROP TABLE IF EXISTS fw_m_setting CASCADE;
