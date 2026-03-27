-- infra/docker/init-db/07-ratelimit-schema.sql
-- スキーマ定義はマイグレーション（ratelimit-db/migrations/）が担当する。
-- 本ファイルは DB 接続先の切り替えとスキーマ・拡張機能の初期作成・権限設定のみを行う。
-- CREATE TABLE / ALTER TABLE / CREATE INDEX / CREATE TRIGGER は含まない。

\c ratelimit_db;

-- pgcrypto 拡張（gen_random_uuid 関数に必要）
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ratelimit スキーマの作成（マイグレーション実行前にスキーマが存在する必要がある）
CREATE SCHEMA IF NOT EXISTS ratelimit;

-- k1s0ユーザーへのアクセス権限付与（H-17 監査対応）
-- ratelimit スキーマへの DML 権限を k1s0_ratelimit_rw ロールに付与する
-- 注意: k1s0_ratelimit_rw ロールは 16-roles.sh に未定義のため、追加が必要（別担当エージェントが対応）
GRANT USAGE ON SCHEMA ratelimit TO k1s0_ratelimit_rw;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA ratelimit TO k1s0_ratelimit_rw;
ALTER DEFAULT PRIVILEGES IN SCHEMA ratelimit GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0_ratelimit_rw;
