-- infra/docker/init-db/01-create-databases.sql

-- 認証用DB（Keycloak）
CREATE DATABASE keycloak;

-- API ゲートウェイ用DB（Kong）
CREATE DATABASE kong;

-- アプリケーション用DB（Tier ごとに分離）
CREATE DATABASE k1s0_system;
CREATE DATABASE k1s0_business;
CREATE DATABASE k1s0_service;

-- auth-server 用DB
CREATE DATABASE auth_db;

-- config-server 用DB
CREATE DATABASE config_db;

-- dlq-manager 用DB
CREATE DATABASE dlq_db;

-- featureflag-server 用DB
CREATE DATABASE featureflag_db;

-- ratelimit-server 用DB
CREATE DATABASE ratelimit_db;

-- tenant-server 用DB
CREATE DATABASE tenant_db;

-- vault-server 用DB
CREATE DATABASE vault_db;

-- task-server 用DB（M-2 監査対応: 使用意図を明記）
-- k1s0_task は将来実装予定の task-server サービスが使用するデータベース
-- 現時点では未実装のため孤立状態だが、将来的にタスク管理機能を提供するサービスで利用される予定
CREATE DATABASE k1s0_task;

-- event-store-server 用DB
CREATE DATABASE event_store_db;

-- scheduler-server 用DB
CREATE DATABASE scheduler_db;

-- notification-server 用DB
CREATE DATABASE notification_db;

-- navigation-server 用DB は削除済み（LOW-4 監査対応: navigation-rust は DB を使用せず YAML 設定のみで動作する）

-- policy-server 用DB
CREATE DATABASE policy_db;

-- quota-server 用DB
CREATE DATABASE quota_db;

-- rule-engine-server 用DB
CREATE DATABASE rule_engine_db;

-- search-server 用DB
CREATE DATABASE search_db;

-- session-server 用DB
CREATE DATABASE session_db;

-- workflow-server 用DB
CREATE DATABASE workflow_db;

-- file-server 用DB
CREATE DATABASE file_db;

-- service-catalog 用DB
CREATE DATABASE service_catalog_db;

-- saga 用レガシーDB（現在は未使用）
-- saga サービスは k1s0_system DB の saga スキーマを使用する（04-saga-schema.sql で k1s0_system に接続・スキーマ作成）
-- saga_db は当初の設計で用意されたが k1s0_system への統合後は未使用。互換性のため保持する。
CREATE DATABASE saga_db;

-- master-maintenance 用スキーマ (k1s0_system 内)
-- master_maintenance スキーマは k1s0_system DB 内で作成される

-- api-registry 用DB
CREATE DATABASE api_registry_db;

-- app-registry 用DB
CREATE DATABASE app_registry_db;

-- アプリケーションユーザー k1s0 へのデータベース接続権限を付与する
-- init-db/00-create-app-user.sql でロール作成後に実行される
GRANT CONNECT ON DATABASE k1s0_service TO k1s0;
GRANT CONNECT ON DATABASE k1s0_system TO k1s0;
GRANT CONNECT ON DATABASE k1s0_business TO k1s0;
GRANT CONNECT ON DATABASE k1s0_task TO k1s0;
GRANT CONNECT ON DATABASE auth_db TO k1s0;
GRANT CONNECT ON DATABASE config_db TO k1s0;
GRANT CONNECT ON DATABASE featureflag_db TO k1s0;
GRANT CONNECT ON DATABASE ratelimit_db TO k1s0;
GRANT CONNECT ON DATABASE tenant_db TO k1s0;
GRANT CONNECT ON DATABASE vault_db TO k1s0;
GRANT CONNECT ON DATABASE session_db TO k1s0;
GRANT CONNECT ON DATABASE event_store_db TO k1s0;
GRANT CONNECT ON DATABASE workflow_db TO k1s0;
GRANT CONNECT ON DATABASE scheduler_db TO k1s0;
GRANT CONNECT ON DATABASE notification_db TO k1s0;
-- navigation_db は削除済みのため GRANT も削除（LOW-4 監査対応）
GRANT CONNECT ON DATABASE policy_db TO k1s0;
GRANT CONNECT ON DATABASE quota_db TO k1s0;
GRANT CONNECT ON DATABASE rule_engine_db TO k1s0;
GRANT CONNECT ON DATABASE search_db TO k1s0;
GRANT CONNECT ON DATABASE dlq_db TO k1s0;
GRANT CONNECT ON DATABASE file_db TO k1s0;
GRANT CONNECT ON DATABASE service_catalog_db TO k1s0;
GRANT CONNECT ON DATABASE saga_db TO k1s0;
GRANT CONNECT ON DATABASE api_registry_db TO k1s0;
GRANT CONNECT ON DATABASE app_registry_db TO k1s0;
