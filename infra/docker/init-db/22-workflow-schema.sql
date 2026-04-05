-- infra/docker/init-db/22-workflow-schema.sql
-- CRIT-004 監査対応: workflow サービス起動に必要なスキーマ初期化 SQL が欠落していたため追加する
-- スキーマ定義はマイグレーション（regions/system/database/workflow-db/migrations/）が担当する。
-- 本ファイルは DB 接続先の切り替えとスキーマ・拡張機能の初期作成・権限設定のみを行う。
-- CREATE TABLE / ALTER TABLE / CREATE INDEX / CREATE TRIGGER は含まない。

\c workflow_db;

-- pgcrypto 拡張（gen_random_uuid 関数に必要）
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- workflow スキーマの作成（マイグレーション実行前にスキーマが存在する必要がある）
CREATE SCHEMA IF NOT EXISTS workflow;

-- k1s0 アプリユーザーへのスキーマ・テーブル権限付与
GRANT USAGE ON SCHEMA workflow TO k1s0;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA workflow TO k1s0;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA workflow TO k1s0;
-- 将来追加されるテーブルにも自動で権限を付与する
ALTER DEFAULT PRIVILEGES IN SCHEMA workflow
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0;
-- マイグレーション用ロールにもスキーマ操作権限を付与する
GRANT ALL ON SCHEMA workflow TO k1s0_migration;
