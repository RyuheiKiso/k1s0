-- infra/docker/init-db/20-service-catalog-schema.sql
-- スキーマ定義はマイグレーション（regions/system/database/service-catalog-db/migrations/）が担当する。
-- 本ファイルは DB 接続先の切り替えとスキーマの初期作成・権限設定のみを行う。
-- CREATE TABLE / ALTER TABLE / CREATE INDEX / CREATE TRIGGER は含まない。
-- service_catalog スキーマは service_catalog_db データベース内に作成する

\c service_catalog_db;

-- service_catalog スキーマの作成（マイグレーション実行前にスキーマが存在する必要がある）
CREATE SCHEMA IF NOT EXISTS service_catalog;

-- k1s0 アプリユーザーへのスキーマ・テーブル権限付与
GRANT USAGE ON SCHEMA service_catalog TO k1s0;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA service_catalog TO k1s0;
-- 将来追加されるテーブルにも自動で権限を付与する
ALTER DEFAULT PRIVILEGES IN SCHEMA service_catalog
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0;
