-- infra/docker/init-db/09-vault-schema.sql
-- スキーマ定義はマイグレーション（vault-db/migrations/）が担当する。
-- 本ファイルは DB 接続先の切り替えとスキーマ・拡張機能の初期作成・権限設定のみを行う。
-- CREATE TABLE / ALTER TABLE / CREATE INDEX は含まない。

\c vault_db;

-- pgcrypto 拡張（gen_random_uuid 関数に必要）
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- vault スキーマの作成（マイグレーション実行前にスキーマが存在する必要がある）
CREATE SCHEMA IF NOT EXISTS vault;

-- k1s0ユーザーへのアクセス権限付与（H-17 監査対応）
-- vault スキーマへの DML 権限を k1s0_vault_rw ロールに付与する
-- k1s0_vault_rw ロールは 16-roles.sh で作成される
GRANT USAGE ON SCHEMA vault TO k1s0_vault_rw;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA vault TO k1s0_vault_rw;
ALTER DEFAULT PRIVILEGES IN SCHEMA vault GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0_vault_rw;
