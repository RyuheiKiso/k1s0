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

-- スキーマが存在する場合のみ権限付与
-- （スキーマはマイグレーションで作成されるため、ロール作成のみ行う）
-- 本番環境では Terraform の postgresql_grant リソースが権限を管理する
