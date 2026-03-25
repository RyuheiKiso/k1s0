-- 開発環境用サービス別DBロール作成スクリプト
-- 本番環境では Terraform の roles.tf で管理すること
-- C-02（DB認証情報のシングルユーザー共有）対応として実装
-- パスワードは "dev-{service}" 形式の固定値（開発専用）

-- マイグレーション専用ロール（全スキーマのDDL権限を持つ）
-- マイグレーション実行時のみ使用する
CREATE ROLE k1s0_migration WITH LOGIN PASSWORD 'dev-migration';

-- auth サービス専用ロール（auth スキーマのみ DML 可）
CREATE ROLE k1s0_auth_rw WITH LOGIN PASSWORD 'dev-auth';

-- config サービス専用ロール（config スキーマのみ DML 可）
CREATE ROLE k1s0_config_rw WITH LOGIN PASSWORD 'dev-config';

-- saga サービス専用ロール（saga スキーマのみ DML 可）
CREATE ROLE k1s0_saga_rw WITH LOGIN PASSWORD 'dev-saga';

-- session サービス専用ロール（session スキーマのみ DML 可）
CREATE ROLE k1s0_session_rw WITH LOGIN PASSWORD 'dev-session';

-- tenant サービス専用ロール（tenant スキーマのみ DML 可）
CREATE ROLE k1s0_tenant_rw WITH LOGIN PASSWORD 'dev-tenant';

-- workflow サービス専用ロール（workflow スキーマのみ DML 可）
CREATE ROLE k1s0_workflow_rw WITH LOGIN PASSWORD 'dev-workflow';

-- dlq サービス専用ロール（dlq スキーマのみ DML 可）
CREATE ROLE k1s0_dlq_rw WITH LOGIN PASSWORD 'dev-dlq';

-- notification サービス専用ロール（notification スキーマのみ DML 可）
CREATE ROLE k1s0_notification_rw WITH LOGIN PASSWORD 'dev-notification';

-- vault サービス専用ロール（vault スキーマのみ DML 可）
CREATE ROLE k1s0_vault_rw WITH LOGIN PASSWORD 'dev-vault';

-- スキーマが存在する場合のみ権限を付与する（スキーマ未存在時のエラーを回避）（H-2 監査対応）
-- 本番環境では Terraform の postgresql_grant リソースが権限を管理する

DO $$ BEGIN
  -- auth スキーマへの DML 権限を k1s0_auth_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'auth') THEN
    GRANT USAGE ON SCHEMA auth TO k1s0_auth_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA auth TO k1s0_auth_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA auth TO k1s0_auth_rw;
  END IF;
END $$;

DO $$ BEGIN
  -- config スキーマへの DML 権限を k1s0_config_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'config') THEN
    GRANT USAGE ON SCHEMA config TO k1s0_config_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA config TO k1s0_config_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA config TO k1s0_config_rw;
  END IF;
END $$;

DO $$ BEGIN
  -- saga スキーマへの DML 権限を k1s0_saga_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'saga') THEN
    GRANT USAGE ON SCHEMA saga TO k1s0_saga_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA saga TO k1s0_saga_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA saga TO k1s0_saga_rw;
  END IF;
END $$;

DO $$ BEGIN
  -- workflow スキーマへの DML 権限を k1s0_workflow_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'workflow') THEN
    GRANT USAGE ON SCHEMA workflow TO k1s0_workflow_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA workflow TO k1s0_workflow_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA workflow TO k1s0_workflow_rw;
  END IF;
END $$;

DO $$ BEGIN
  -- dlq スキーマへの DML 権限を k1s0_dlq_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'dlq') THEN
    GRANT USAGE ON SCHEMA dlq TO k1s0_dlq_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA dlq TO k1s0_dlq_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA dlq TO k1s0_dlq_rw;
  END IF;
END $$;

DO $$ BEGIN
  -- notification スキーマへの DML 権限を k1s0_notification_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'notification') THEN
    GRANT USAGE ON SCHEMA notification TO k1s0_notification_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA notification TO k1s0_notification_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA notification TO k1s0_notification_rw;
  END IF;
END $$;

DO $$ BEGIN
  -- vault スキーマへの DML 権限を k1s0_vault_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'vault') THEN
    GRANT USAGE ON SCHEMA vault TO k1s0_vault_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA vault TO k1s0_vault_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA vault TO k1s0_vault_rw;
  END IF;
END $$;

DO $$ BEGIN
  -- session スキーマへの DML 権限を k1s0_session_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'session') THEN
    GRANT USAGE ON SCHEMA session TO k1s0_session_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA session TO k1s0_session_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA session TO k1s0_session_rw;
  END IF;
END $$;

DO $$ BEGIN
  -- tenant スキーマへの DML 権限を k1s0_tenant_rw ロールに付与する
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'tenant') THEN
    GRANT USAGE ON SCHEMA tenant TO k1s0_tenant_rw;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA tenant TO k1s0_tenant_rw;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA tenant TO k1s0_tenant_rw;
  END IF;
END $$;
