-- infra/docker/init-db/13-task-schema.sql
-- スキーマ定義はマイグレーション（regions/service/task/...）が担当する。
-- 本ファイルは DB 接続先の切り替えとスキーマ・拡張機能の初期作成・権限設定のみを行う。
-- CREATE TABLE / ALTER TABLE / CREATE INDEX / CREATE TRIGGER / CREATE POLICY は含まない。

\c k1s0_service;

-- pgcrypto 拡張（gen_random_uuid 関数に必要）
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- task_service スキーマの作成（マイグレーション実行前にスキーマが存在する必要がある）
CREATE SCHEMA IF NOT EXISTS task_service;

-- k1s0 アプリユーザーへのスキーマ使用権限とテーブル操作権限を付与する
-- RLS ポリシーが正常に機能するため、k1s0 ロールは NOBYPASSRLS で作成されている
GRANT USAGE ON SCHEMA task_service TO k1s0;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA task_service TO k1s0;
ALTER DEFAULT PRIVILEGES IN SCHEMA task_service GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0;

-- sqlx が k1s0_service 内に新規スキーマを作成できるように DATABASE レベルの CREATE 権限を付与する
GRANT CREATE ON DATABASE k1s0_service TO k1s0;
-- sqlx マイグレーションが task_service スキーマ内に _sqlx_migrations テーブルを作成できるように
-- スキーマレベルの CREATE 権限を付与する（GRANT CREATE ON DATABASE とは別物）（C-2 監査対応）
GRANT CREATE ON SCHEMA task_service TO k1s0;
