-- infra/docker/init-db/08-tenant-schema.sql
-- スキーマ定義はマイグレーション（tenant-db/migrations/）が担当する。
-- 本ファイルは DB 接続先の切り替えとスキーマ・拡張機能の初期作成・権限設定のみを行う。
-- CREATE TABLE / ALTER TABLE / CREATE INDEX / CREATE TRIGGER は含まない。
-- 対応マイグレーション:
--   001_create_schema         : スキーマ・拡張機能・updated_at トリガー関数
--   002_create_tenants        : tenants テーブル（settings JSONB, plan VARCHAR(50) + CHECK）
--   003_create_tenant_members : tenant_members テーブル（role CHECK あり）
--   004_add_tenant_fields     : keycloak_realm, db_schema カラム追加
--   005_add_owner_id          : owner_id カラム追加（VARCHAR(255)）
--   006_change_owner_id_to_uuid : owner_id を UUID 型に変更
--   007_conditional_unique    : tenants.name を条件付きユニーク制約に変更（削除済みを除外）

\c tenant_db;

-- pgcrypto 拡張（gen_random_uuid 関数に必要）
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- tenant スキーマの作成（マイグレーション実行前にスキーマが存在する必要がある）
CREATE SCHEMA IF NOT EXISTS tenant;

-- k1s0_tenant_rw ロールへの権限付与（H-17 監査対応）
-- k1s0_tenant_rw ロールは 01z-create-roles.sh で作成される
-- tenant スキーマの利用権限を付与する
GRANT USAGE ON SCHEMA tenant TO k1s0_tenant_rw;
-- 全テーブルへの DML 権限を付与する
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA tenant TO k1s0_tenant_rw;
-- 今後作成されるテーブルへの DML 権限をデフォルトで付与する
ALTER DEFAULT PRIVILEGES IN SCHEMA tenant GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0_tenant_rw;
