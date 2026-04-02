-- infra/docker/init-db/21-master-maintenance-schema.sql
-- スキーマ定義はマイグレーション（regions/system/database/master-maintenance-db/migrations/）が担当する。
-- 本ファイルは DB 接続先の切り替えとスキーマ・拡張機能の初期作成・権限設定のみを行う。
-- CREATE TABLE / ALTER TABLE / CREATE INDEX / CREATE TRIGGER は含まない。
-- master_maintenance スキーマは k1s0_master_maintenance 専用データベース内に作成する
-- CRIT-001 監査対応: k1s0_system からの分離で event-monitor との _sqlx_migrations 競合を解消

\c k1s0_master_maintenance;

-- pgcrypto 拡張（gen_random_uuid 関数に必要）
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- master_maintenance スキーマの作成（マイグレーション実行前にスキーマが存在する必要がある）
CREATE SCHEMA IF NOT EXISTS master_maintenance;

-- k1s0 アプリユーザーへのスキーマ・テーブル権限付与
GRANT USAGE ON SCHEMA master_maintenance TO k1s0;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA master_maintenance TO k1s0;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA master_maintenance TO k1s0;
-- 将来追加されるテーブルにも自動で権限を付与する
ALTER DEFAULT PRIVILEGES IN SCHEMA master_maintenance
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0;
