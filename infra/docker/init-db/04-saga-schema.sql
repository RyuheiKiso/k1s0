-- infra/docker/init-db/04-saga-schema.sql
-- スキーマ定義はマイグレーション（regions/system/server/rust/saga/...）が担当する。（LOW-2 監査対応: go → rust に修正）
-- 本ファイルは DB 接続先の切り替えとスキーマ作成・権限設定のみを行う。
-- CREATE TABLE / ALTER TABLE / CREATE INDEX / CREATE TRIGGER は含まない。

-- CRIT-09 監査対応: saga サービス専用 DB（k1s0_saga）に接続する。
-- k1s0_system との共有は sqlx _sqlx_migrations テーブル競合を引き起こすため分離した。
\connect k1s0_saga;

-- saga スキーマの作成（マイグレーション実行前にスキーマが存在する必要がある）
CREATE SCHEMA IF NOT EXISTS saga;

-- k1s0ユーザーへのアクセス権限付与（H-17 監査対応）
-- saga スキーマへの DML 権限を k1s0_saga_rw ロールに付与する
-- k1s0_saga_rw ロールは 01z-create-roles.sh で作成される
GRANT USAGE ON SCHEMA saga TO k1s0_saga_rw;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA saga TO k1s0_saga_rw;
ALTER DEFAULT PRIVILEGES IN SCHEMA saga GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0_saga_rw;
