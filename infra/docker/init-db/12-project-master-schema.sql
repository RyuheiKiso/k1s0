-- infra/docker/init-db/12-project-master-schema.sql
-- スキーマ定義はマイグレーション（regions/business/taskmanagement/...）が担当する。
-- 本ファイルは DB 接続先の切り替えとスキーマ・拡張機能の初期作成・権限設定のみを行う。
-- CREATE TABLE / ALTER TABLE / CREATE INDEX / CREATE TRIGGER / CREATE POLICY は含まない。

\c k1s0_business;

-- pgcrypto 拡張（gen_random_uuid 関数に必要）
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- project_master スキーマの作成（マイグレーション実行前にスキーマが存在する必要がある）
CREATE SCHEMA IF NOT EXISTS project_master;

-- k1s0 アプリユーザーへのスキーマ使用権限とテーブル操作権限を付与する
GRANT USAGE ON SCHEMA project_master TO k1s0;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA project_master TO k1s0;
ALTER DEFAULT PRIVILEGES IN SCHEMA project_master GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0;

-- sqlx が k1s0_business 内に新規スキーマを作成できるように DATABASE レベルの CREATE 権限を付与する
GRANT CREATE ON DATABASE k1s0_business TO k1s0;
-- sqlx マイグレーションが project_master スキーマ内に _sqlx_migrations テーブルを作成できるように
-- スキーマレベルの CREATE 権限を付与する（GRANT CREATE ON DATABASE とは別物）（C-2 監査対応）
GRANT CREATE ON SCHEMA project_master TO k1s0;
