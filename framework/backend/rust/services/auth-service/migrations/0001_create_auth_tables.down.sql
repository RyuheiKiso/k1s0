-- auth-service: 認証・認可関連テーブルの削除
-- 注意: このマイグレーションはデータを完全に削除します

-- トリガーの削除
DROP TRIGGER IF EXISTS tr_fw_m_user_updated ON fw_m_user;
DROP TRIGGER IF EXISTS tr_fw_m_role_updated ON fw_m_role;
DROP TRIGGER IF EXISTS tr_fw_m_permission_updated ON fw_m_permission;

-- 関数の削除
DROP FUNCTION IF EXISTS fw_update_timestamp();

-- テーブルの削除（依存関係の逆順）
DROP TABLE IF EXISTS fw_t_refresh_token CASCADE;
DROP TABLE IF EXISTS fw_m_role_permission CASCADE;
DROP TABLE IF EXISTS fw_m_user_role CASCADE;
DROP TABLE IF EXISTS fw_m_permission CASCADE;
DROP TABLE IF EXISTS fw_m_role CASCADE;
DROP TABLE IF EXISTS fw_m_user CASCADE;
