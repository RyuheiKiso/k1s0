#!/bin/bash
# HIGH-008 監査対応: スキーマ作成後の権限付与スクリプト
# 全スキーマ初期化 SQL（02〜22番台）実行後に権限を付与することで、
# 01z-create-roles.sh 実行時にスキーマが未存在で権限付与がスキップされる問題を解消する。
#
# 実行順序: 01z（ロール作成）→ 02〜22（スキーマ作成）→ 99（権限付与）
# 本番環境では Terraform の postgresql_grant リソースが権限を管理する（本スクリプトは開発環境専用）
#
# 重要: 各スキーマは専用 DB に存在するため、権限付与は \c でDB切り替え後に実行する必要がある。
# postgres DB に接続したまま他DBのスキーマを参照すると pg_namespace 検索が常に空になる。
set -e

PSQL_BASE="psql -v ON_ERROR_STOP=1 --username ${POSTGRES_USER:-postgres}"

# 各DBに接続して権限付与する関数
# 引数: $1=DB名 $2=スキーマ名 $3=ロール名
grant_schema() {
  local dbname="$1"
  local schema="$2"
  local role="$3"
  ${PSQL_BASE} --dbname "${dbname}" <<-EOSQL
DO \$\$ BEGIN
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = '${schema}') THEN
    GRANT USAGE ON SCHEMA ${schema} TO ${role};
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA ${schema} TO ${role};
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA ${schema} TO ${role};
    ALTER DEFAULT PRIVILEGES IN SCHEMA ${schema}
      GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO ${role};
    ALTER DEFAULT PRIVILEGES IN SCHEMA ${schema}
      GRANT USAGE, SELECT ON SEQUENCES TO ${role};
  ELSE
    RAISE NOTICE 'スキーマ % が DB % に存在しないためスキップします', '${schema}', '${dbname}';
  END IF;
END \$\$;
EOSQL
}

# auth スキーマ → auth_db
grant_schema "auth_db" "auth" "k1s0_auth_rw"

# config スキーマ → config_db
grant_schema "config_db" "config" "k1s0_config_rw"

# saga スキーマ → k1s0_saga
grant_schema "k1s0_saga" "saga" "k1s0_saga_rw"

# workflow スキーマ → workflow_db（CRIT-004 監査対応で追加）
grant_schema "workflow_db" "workflow" "k1s0_workflow_rw"

# dlq スキーマ → dlq_db
grant_schema "dlq_db" "dlq" "k1s0_dlq_rw"

# notification スキーマ → notification_db
grant_schema "notification_db" "notification" "k1s0_notification_rw"

# vault スキーマ → vault_db
grant_schema "vault_db" "vault" "k1s0_vault_rw"

# session スキーマ → session_db
grant_schema "session_db" "session" "k1s0_session_rw"

# tenant スキーマ → tenant_db
grant_schema "tenant_db" "tenant" "k1s0_tenant_rw"

# ratelimit スキーマ → ratelimit_db（H-17 監査対応フォローアップ）
grant_schema "ratelimit_db" "ratelimit" "k1s0_ratelimit_rw"

# event_monitor スキーマ → k1s0_event_monitor（C-08 / CRIT-001 監査対応）
grant_schema "k1s0_event_monitor" "event_monitor" "k1s0_event_monitor_rw"

# H-006 監査対応: 11サービス分の GRANT を追加する
# 各サービスが自身の DB スキーマのみに DML 権限を持つよう最小権限の原則に従い設定する

# featureflag スキーマ → featureflag_db（06-featureflag-schema.sql と対応）
grant_schema "featureflag_db" "featureflag" "k1s0_featureflag_rw"

# eventstore スキーマ → event_store_db（10-event-store-schema.sql と対応）
grant_schema "event_store_db" "eventstore" "k1s0_event_store_rw"

# scheduler スキーマ → scheduler_db（19-scheduler-schema.sql と対応）
grant_schema "scheduler_db" "scheduler" "k1s0_scheduler_rw"

# policy スキーマ → policy_db
grant_schema "policy_db" "policy" "k1s0_policy_rw"

# quota スキーマ → quota_db
grant_schema "quota_db" "quota" "k1s0_quota_rw"

# rule_engine スキーマ → rule_engine_db
grant_schema "rule_engine_db" "rule_engine" "k1s0_rule_engine_rw"

# search スキーマ → search_db
grant_schema "search_db" "search" "k1s0_search_rw"

# file スキーマ → file_db
grant_schema "file_db" "file" "k1s0_file_rw"

# service_catalog スキーマ → service_catalog_db（20-service-catalog-schema.sql と対応）
grant_schema "service_catalog_db" "service_catalog" "k1s0_service_catalog_rw"

# api_registry スキーマ → api_registry_db
grant_schema "api_registry_db" "api_registry" "k1s0_api_registry_rw"

# app_registry スキーマ → app_registry_db
grant_schema "app_registry_db" "app_registry" "k1s0_app_registry_rw"

# master_maintenance スキーマ → k1s0_master_maintenance（CRIT-001 監査対応）
# master-maintenance サービスは k1s0 汎用ユーザーを使用するため、
# 専用 _rw ロールが未作成の場合は k1s0 に直接付与する
${PSQL_BASE} --dbname "k1s0_master_maintenance" <<-EOSQL
DO \$\$ BEGIN
  IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'master_maintenance') THEN
    GRANT USAGE ON SCHEMA master_maintenance TO k1s0;
    GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA master_maintenance TO k1s0;
    GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA master_maintenance TO k1s0;
    ALTER DEFAULT PRIVILEGES IN SCHEMA master_maintenance
      GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO k1s0;
    ALTER DEFAULT PRIVILEGES IN SCHEMA master_maintenance
      GRANT USAGE, SELECT ON SEQUENCES TO k1s0;
  ELSE
    RAISE NOTICE 'スキーマ master_maintenance が k1s0_master_maintenance DB に存在しないためスキップします';
  END IF;
END \$\$;
EOSQL

echo "権限付与完了（全スキーマ: 各 DB に正しく接続して実行）。"
