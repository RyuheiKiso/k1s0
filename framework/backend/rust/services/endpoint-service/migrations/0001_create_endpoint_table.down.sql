-- endpoint-service: エンドポイントテーブルの削除
-- 注意: このマイグレーションはデータを完全に削除します

-- トリガーの削除
DROP TRIGGER IF EXISTS tr_fw_m_endpoint_updated ON fw_m_endpoint;
DROP TRIGGER IF EXISTS tr_fw_m_service_address_updated ON fw_m_service_address;

-- テーブルの削除（依存関係の逆順）
DROP TABLE IF EXISTS fw_m_endpoint_permission CASCADE;
DROP TABLE IF EXISTS fw_m_service_address CASCADE;
DROP TABLE IF EXISTS fw_m_endpoint CASCADE;
