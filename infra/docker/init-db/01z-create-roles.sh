#!/bin/bash
# 開発環境用サービス別DBロール作成スクリプト
# 各マイクロサービスが専用ロールで接続することで、スキーマ間の不正アクセスを防ぐ（最小権限の原則）。
# パスワードは環境変数から取得し、デフォルト値は開発環境専用の値を使用する（C-2 監査対応）。
#
# 本番環境では Terraform の roles.tf でロールを管理すること（このスクリプトは開発環境専用）。
# 各サービスが使用するロールは以下の通り:
#   k1s0_migration  — DB マイグレーション実行時のみ使用（DDL 権限）
#   k1s0_auth_rw    — auth サービス（認証・APIキー管理）
#   k1s0_config_rw  — config サービス（設定管理）
#   k1s0_saga_rw    — saga サービス（Saga オーケストレーション）
#   k1s0_session_rw — session サービス（セッション管理）
#   k1s0_tenant_rw  — tenant サービス（テナント管理）
#   k1s0_workflow_rw — workflow サービス（ワークフロー管理）
#   k1s0_dlq_rw     — DLQ サービス（Dead Letter Queue 管理）
#   k1s0_notification_rw — notification サービス（通知管理）
#   k1s0_vault_rw   — vault サービス（シークレット管理）
#   k1s0_ratelimit_rw — ratelimit サービス（レートリミット管理）（H-17 監査対応フォローアップ）
#
# 環境変数が未設定の場合は開発用デフォルト値にフォールバックする（本番では必ず設定すること）。
# M-5 監査対応: 各ロールにどのサービスが使用するかコメントを追加。
set -e

psql -v ON_ERROR_STOP=1 \
  --username "${POSTGRES_USER:-postgres}" \
  --dbname "${POSTGRES_DB:-postgres}" <<-EOSQL

-- マイグレーション専用ロール（全スキーマのDDL権限を持つ）
-- マイグレーション実行時のみ使用する（通常時はこのロールで接続しないこと）
CREATE ROLE k1s0_migration WITH LOGIN PASSWORD '${K1S0_MIGRATION_PASSWORD:-dev-migration}'
  NOSUPERUSER NOCREATEDB NOCREATEROLE;

-- auth サービス専用ロール
-- 使用サービス: regions/system/server/rust/auth（認証・APIキー管理）
-- auth スキーマのみ DML 可（SELECT, INSERT, UPDATE, DELETE）
CREATE ROLE k1s0_auth_rw WITH LOGIN PASSWORD '${K1S0_AUTH_PASSWORD:-dev-auth}'
  NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;

-- config サービス専用ロール
-- 使用サービス: regions/system/server/rust/config（設定管理）
-- config スキーマのみ DML 可
CREATE ROLE k1s0_config_rw WITH LOGIN PASSWORD '${K1S0_CONFIG_PASSWORD:-dev-config}'
  NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;

-- saga サービス専用ロール
-- 使用サービス: regions/system/server/rust/saga（Saga オーケストレーション）
-- saga スキーマのみ DML 可
CREATE ROLE k1s0_saga_rw WITH LOGIN PASSWORD '${K1S0_SAGA_PASSWORD:-dev-saga}'
  NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;

-- session サービス専用ロール
-- 使用サービス: regions/system/server/rust/session（セッション管理）
-- session スキーマのみ DML 可
CREATE ROLE k1s0_session_rw WITH LOGIN PASSWORD '${K1S0_SESSION_PASSWORD:-dev-session}'
  NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;

-- tenant サービス専用ロール
-- 使用サービス: regions/system/server/rust/tenant（テナント管理）
-- tenant スキーマのみ DML 可
CREATE ROLE k1s0_tenant_rw WITH LOGIN PASSWORD '${K1S0_TENANT_PASSWORD:-dev-tenant}'
  NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;

-- workflow サービス専用ロール
-- 使用サービス: regions/system/server/rust/workflow（ワークフロー管理）
-- workflow スキーマのみ DML 可
CREATE ROLE k1s0_workflow_rw WITH LOGIN PASSWORD '${K1S0_WORKFLOW_PASSWORD:-dev-workflow}'
  NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;

-- dlq サービス専用ロール
-- 使用サービス: regions/system/server/rust/dlq-manager（Dead Letter Queue 管理）
-- dlq スキーマのみ DML 可
CREATE ROLE k1s0_dlq_rw WITH LOGIN PASSWORD '${K1S0_DLQ_PASSWORD:-dev-dlq}'
  NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;

-- notification サービス専用ロール
-- 使用サービス: regions/service/notification/server（通知管理）
-- notification スキーマのみ DML 可
CREATE ROLE k1s0_notification_rw WITH LOGIN PASSWORD '${K1S0_NOTIFICATION_PASSWORD:-dev-notification}'
  NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;

-- vault サービス専用ロール
-- 使用サービス: regions/system/server/rust/vault（シークレット管理）
-- vault スキーマのみ DML 可
CREATE ROLE k1s0_vault_rw WITH LOGIN PASSWORD '${K1S0_VAULT_PASSWORD:-dev-vault}'
  NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;

-- ratelimit サービス専用ロール（H-17 監査対応フォローアップ）
-- 使用サービス: regions/system/server/rust/ratelimit（レートリミット管理）
-- ratelimit スキーマのみ DML 可。07-ratelimit-schema.sql の GRANT 文と対応する。
CREATE ROLE k1s0_ratelimit_rw WITH LOGIN PASSWORD '${K1S0_RATELIMIT_PASSWORD:-dev-ratelimit}'
  NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;

-- スキーマが存在する場合のみ権限を付与する（スキーマ未存在時のエラーを回避）（H-2 監査対応）
-- 本番環境では Terraform の postgresql_grant リソースが権限を管理する

DO \$\$ BEGIN
  -- auth スキーマへの DML 権限を k1s0_auth_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'auth') THEN
    GRANT USAGE ON SCHEMA auth TO k1s0_auth_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA auth TO k1s0_auth_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA auth TO k1s0_auth_rw;
  END IF;
END \$\$;

DO \$\$ BEGIN
  -- config スキーマへの DML 権限を k1s0_config_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'config') THEN
    GRANT USAGE ON SCHEMA config TO k1s0_config_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA config TO k1s0_config_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA config TO k1s0_config_rw;
  END IF;
END \$\$;

DO \$\$ BEGIN
  -- saga スキーマへの DML 権限を k1s0_saga_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'saga') THEN
    GRANT USAGE ON SCHEMA saga TO k1s0_saga_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA saga TO k1s0_saga_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA saga TO k1s0_saga_rw;
  END IF;
END \$\$;

DO \$\$ BEGIN
  -- workflow スキーマへの DML 権限を k1s0_workflow_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'workflow') THEN
    GRANT USAGE ON SCHEMA workflow TO k1s0_workflow_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA workflow TO k1s0_workflow_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA workflow TO k1s0_workflow_rw;
  END IF;
END \$\$;

DO \$\$ BEGIN
  -- dlq スキーマへの DML 権限を k1s0_dlq_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'dlq') THEN
    GRANT USAGE ON SCHEMA dlq TO k1s0_dlq_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA dlq TO k1s0_dlq_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA dlq TO k1s0_dlq_rw;
  END IF;
END \$\$;

DO \$\$ BEGIN
  -- notification スキーマへの DML 権限を k1s0_notification_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'notification') THEN
    GRANT USAGE ON SCHEMA notification TO k1s0_notification_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA notification TO k1s0_notification_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA notification TO k1s0_notification_rw;
  END IF;
END \$\$;

DO \$\$ BEGIN
  -- vault スキーマへの DML 権限を k1s0_vault_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'vault') THEN
    GRANT USAGE ON SCHEMA vault TO k1s0_vault_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA vault TO k1s0_vault_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA vault TO k1s0_vault_rw;
  END IF;
END \$\$;

DO \$\$ BEGIN
  -- session スキーマへの DML 権限を k1s0_session_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'session') THEN
    GRANT USAGE ON SCHEMA session TO k1s0_session_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA session TO k1s0_session_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA session TO k1s0_session_rw;
  END IF;
END \$\$;

DO \$\$ BEGIN
  -- tenant スキーマへの DML 権限を k1s0_tenant_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'tenant') THEN
    GRANT USAGE ON SCHEMA tenant TO k1s0_tenant_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA tenant TO k1s0_tenant_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA tenant TO k1s0_tenant_rw;
  END IF;
END \$\$;

DO \$\$ BEGIN
  -- ratelimit スキーマへの DML 権限を k1s0_ratelimit_rw ロールに付与する（H-17 監査対応フォローアップ）
  -- このブロックは 07-ratelimit-schema.sql 実行後に ratelimit スキーマが存在する場合のみ権限を付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'ratelimit') THEN
    GRANT USAGE ON SCHEMA ratelimit TO k1s0_ratelimit_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA ratelimit TO k1s0_ratelimit_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA ratelimit TO k1s0_ratelimit_rw;
  END IF;
END \$\$;

EOSQL
