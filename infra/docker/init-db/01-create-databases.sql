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

-- task サービスは k1s0_service DB を使用するため k1s0_task は不要（MED-009 監査対応: 孤立 DB 削除）
-- task/config/default.yaml の database.name: k1s0_service を参照

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

-- saga サービス専用DB（CRIT-09 監査対応）
-- saga サービスを k1s0_system から分離し、_sqlx_migrations テーブルの競合を解消する。
-- master-maintenance サービスも k1s0_system を使用するため、同一 DB の共有は sqlx マイグレーション競合を引き起こす。
CREATE DATABASE k1s0_saga;

-- event-monitor サービス専用DB（CRIT-001 監査対応: k1s0_system からの分離で _sqlx_migrations 競合を解消）
-- event-monitor-db と master-maintenance-db が同一 DB を共有すると migration 番号が衝突する
CREATE DATABASE k1s0_event_monitor;

-- master-maintenance サービス専用DB（CRIT-001 監査対応: k1s0_system からの分離で _sqlx_migrations 競合を解消）
CREATE DATABASE k1s0_master_maintenance;

-- api-registry 用DB
CREATE DATABASE api_registry_db;

-- app-registry 用DB
CREATE DATABASE app_registry_db;

-- アプリケーションユーザー k1s0 へのデータベース接続権限を付与する
-- init-db/00-create-app-user.sql でロール作成後に実行される
GRANT CONNECT ON DATABASE k1s0_service TO k1s0;
GRANT CONNECT ON DATABASE k1s0_system TO k1s0;
GRANT CONNECT ON DATABASE k1s0_business TO k1s0;
-- k1s0_task は MED-009 対応で削除済みのため GRANT も削除
GRANT CONNECT ON DATABASE k1s0_event_monitor TO k1s0;
GRANT CONNECT ON DATABASE k1s0_master_maintenance TO k1s0;
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
GRANT CONNECT ON DATABASE k1s0_saga TO k1s0;
GRANT CONNECT ON DATABASE api_registry_db TO k1s0;
GRANT CONNECT ON DATABASE app_registry_db TO k1s0;
