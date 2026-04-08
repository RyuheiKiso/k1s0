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
#   k1s0_event_monitor_rw — event-monitor サービス（イベント監視）（C-08 監査対応）
#   k1s0_featureflag_rw — featureflag サービス（フィーチャーフラグ管理）（H-006 監査対応）
#   k1s0_event_store_rw — event-store サービス（イベントソーシング）（H-006 監査対応）
#   k1s0_scheduler_rw  — scheduler サービス（スケジューラ管理）（H-006 監査対応）
#   k1s0_policy_rw     — policy サービス（アクセスポリシー管理）（H-006 監査対応）
#   k1s0_quota_rw      — quota サービス（クォータ管理）（H-006 監査対応）
#   k1s0_rule_engine_rw — rule-engine サービス（ルールエンジン）（H-006 監査対応）
#   k1s0_search_rw     — search サービス（全文検索管理）（H-006 監査対応）
#   k1s0_file_rw       — file サービス（ファイル管理）（H-006 監査対応）
#   k1s0_service_catalog_rw — service-catalog サービス（サービスカタログ管理）（H-006 監査対応）
#   k1s0_api_registry_rw — api-registry サービス（API スキーマレジストリ）（H-006 監査対応）
#   k1s0_app_registry_rw — app-registry サービス（アプリバージョン管理）（H-006 監査対応）
#   k1s0_master_maintenance_rw — master-maintenance サービス（マスターデータメンテナンス）（HIGH-003 監査対応）
#
# 環境変数が未設定の場合は開発用デフォルト値にフォールバックする（本番では必ず設定すること）。
# M-5 監査対応: 各ロールにどのサービスが使用するかコメントを追加。
set -e

# H-10 監査対応: SQL インジェクション対策
# HEREDOC 内でのシェル変数展開はパスワードに特殊文字（シングルクォート等）が含まれる場合に
# SQL 構文エラーやインジェクションのリスクがある。
# ロール作成は psql -c + ドル引用符（$$...$$）で安全にパスワードを渡す。
# 権限付与（DO ブロック）はパスワードを含まないため HEREDOC のまま維持する。

PSQL_CMD="psql -v ON_ERROR_STOP=1 --username ${POSTGRES_USER:-postgres} --dbname ${POSTGRES_DB:-postgres}"

# マイグレーション専用ロール（全スキーマのDDL権限を持つ）
# マイグレーション実行時のみ使用する（通常時はこのロールで接続しないこと）
$PSQL_CMD -c "CREATE ROLE k1s0_migration WITH LOGIN PASSWORD \$\$${K1S0_MIGRATION_PASSWORD:-dev-migration}\$\$ NOSUPERUSER NOCREATEDB NOCREATEROLE;"

# auth サービス専用ロール
# 使用サービス: regions/system/server/rust/auth（認証・APIキー管理）
# auth スキーマのみ DML 可（SELECT, INSERT, UPDATE, DELETE）
$PSQL_CMD -c "CREATE ROLE k1s0_auth_rw WITH LOGIN PASSWORD \$\$${K1S0_AUTH_PASSWORD:-dev-auth}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# config サービス専用ロール
# 使用サービス: regions/system/server/rust/config（設定管理）
# config スキーマのみ DML 可
$PSQL_CMD -c "CREATE ROLE k1s0_config_rw WITH LOGIN PASSWORD \$\$${K1S0_CONFIG_PASSWORD:-dev-config}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# saga サービス専用ロール
# 使用サービス: regions/system/server/rust/saga（Saga オーケストレーション）
# saga スキーマのみ DML 可
$PSQL_CMD -c "CREATE ROLE k1s0_saga_rw WITH LOGIN PASSWORD \$\$${K1S0_SAGA_PASSWORD:-dev-saga}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# session サービス専用ロール
# 使用サービス: regions/system/server/rust/session（セッション管理）
# session スキーマのみ DML 可
$PSQL_CMD -c "CREATE ROLE k1s0_session_rw WITH LOGIN PASSWORD \$\$${K1S0_SESSION_PASSWORD:-dev-session}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# tenant サービス専用ロール
# 使用サービス: regions/system/server/rust/tenant（テナント管理）
# tenant スキーマのみ DML 可
$PSQL_CMD -c "CREATE ROLE k1s0_tenant_rw WITH LOGIN PASSWORD \$\$${K1S0_TENANT_PASSWORD:-dev-tenant}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# workflow サービス専用ロール
# 使用サービス: regions/system/server/rust/workflow（ワークフロー管理）
# workflow スキーマのみ DML 可
$PSQL_CMD -c "CREATE ROLE k1s0_workflow_rw WITH LOGIN PASSWORD \$\$${K1S0_WORKFLOW_PASSWORD:-dev-workflow}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# dlq サービス専用ロール
# 使用サービス: regions/system/server/rust/dlq-manager（Dead Letter Queue 管理）
# dlq スキーマのみ DML 可
$PSQL_CMD -c "CREATE ROLE k1s0_dlq_rw WITH LOGIN PASSWORD \$\$${K1S0_DLQ_PASSWORD:-dev-dlq}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# notification サービス専用ロール
# 使用サービス: regions/service/notification/server（通知管理）
# notification スキーマのみ DML 可
$PSQL_CMD -c "CREATE ROLE k1s0_notification_rw WITH LOGIN PASSWORD \$\$${K1S0_NOTIFICATION_PASSWORD:-dev-notification}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# vault サービス専用ロール
# 使用サービス: regions/system/server/rust/vault（シークレット管理）
# vault スキーマのみ DML 可
$PSQL_CMD -c "CREATE ROLE k1s0_vault_rw WITH LOGIN PASSWORD \$\$${K1S0_VAULT_PASSWORD:-dev-vault}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# ratelimit サービス専用ロール（H-17 監査対応フォローアップ）
# 使用サービス: regions/system/server/rust/ratelimit（レートリミット管理）
# ratelimit スキーマのみ DML 可。07-ratelimit-schema.sql の GRANT 文と対応する。
$PSQL_CMD -c "CREATE ROLE k1s0_ratelimit_rw WITH LOGIN PASSWORD \$\$${K1S0_RATELIMIT_PASSWORD:-dev-ratelimit}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# event-monitor サービス専用ロール（C-08 監査対応）
# 使用サービス: regions/system/server/rust/event-monitor（イベント監視）
# event_monitor スキーマのみ DML 可。18-event-monitor-schema.sql の GRANT 文と対応する。
$PSQL_CMD -c "CREATE ROLE k1s0_event_monitor_rw WITH LOGIN PASSWORD \$\$${K1S0_EVENT_MONITOR_PASSWORD:-dev-event-monitor}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# H-006 監査対応: 以下11サービス専用ロールを追加する（既存の k1s0_auth_rw パターンに従う）
# 各ロールは対応するスキーマのみに DML 権限を付与し、最小権限の原則を遵守する

# featureflag サービス専用ロール
# 使用サービス: regions/system/server/rust/featureflag（フィーチャーフラグ管理）
# featureflag スキーマのみ DML 可。06-featureflag-schema.sql と対応する。
$PSQL_CMD -c "CREATE ROLE k1s0_featureflag_rw WITH LOGIN PASSWORD \$\$${K1S0_FEATUREFLAG_PASSWORD:-dev-featureflag}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# event-store サービス専用ロール
# 使用サービス: regions/system/server/rust/event-store（イベントソーシング）
# eventstore スキーマのみ DML 可。10-event-store-schema.sql と対応する。
$PSQL_CMD -c "CREATE ROLE k1s0_event_store_rw WITH LOGIN PASSWORD \$\$${K1S0_EVENT_STORE_PASSWORD:-dev-event-store}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# scheduler サービス専用ロール
# 使用サービス: regions/system/server/rust/scheduler（スケジューラ管理）
# scheduler スキーマのみ DML 可。19-scheduler-schema.sql と対応する。
$PSQL_CMD -c "CREATE ROLE k1s0_scheduler_rw WITH LOGIN PASSWORD \$\$${K1S0_SCHEDULER_PASSWORD:-dev-scheduler}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# policy サービス専用ロール
# 使用サービス: regions/system/server/rust/policy（アクセスポリシー管理）
# policy スキーマのみ DML 可
$PSQL_CMD -c "CREATE ROLE k1s0_policy_rw WITH LOGIN PASSWORD \$\$${K1S0_POLICY_PASSWORD:-dev-policy}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# quota サービス専用ロール
# 使用サービス: regions/system/server/rust/quota（クォータ管理）
# quota スキーマのみ DML 可
$PSQL_CMD -c "CREATE ROLE k1s0_quota_rw WITH LOGIN PASSWORD \$\$${K1S0_QUOTA_PASSWORD:-dev-quota}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# rule-engine サービス専用ロール
# 使用サービス: regions/system/server/rust/rule-engine（ルールエンジン）
# rule_engine スキーマのみ DML 可
$PSQL_CMD -c "CREATE ROLE k1s0_rule_engine_rw WITH LOGIN PASSWORD \$\$${K1S0_RULE_ENGINE_PASSWORD:-dev-rule-engine}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# search サービス専用ロール
# 使用サービス: regions/system/server/rust/search（全文検索管理）
# search スキーマのみ DML 可
$PSQL_CMD -c "CREATE ROLE k1s0_search_rw WITH LOGIN PASSWORD \$\$${K1S0_SEARCH_PASSWORD:-dev-search}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# file サービス専用ロール
# 使用サービス: regions/system/server/rust/file（ファイル管理）
# file スキーマのみ DML 可
$PSQL_CMD -c "CREATE ROLE k1s0_file_rw WITH LOGIN PASSWORD \$\$${K1S0_FILE_PASSWORD:-dev-file}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# service-catalog サービス専用ロール
# 使用サービス: regions/system/server/rust/service-catalog（サービスカタログ管理）
# service_catalog スキーマのみ DML 可。20-service-catalog-schema.sql と対応する。
$PSQL_CMD -c "CREATE ROLE k1s0_service_catalog_rw WITH LOGIN PASSWORD \$\$${K1S0_SERVICE_CATALOG_PASSWORD:-dev-service-catalog}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# api-registry サービス専用ロール
# 使用サービス: regions/system/server/rust/api-registry（API スキーマレジストリ）
# api_registry スキーマのみ DML 可
$PSQL_CMD -c "CREATE ROLE k1s0_api_registry_rw WITH LOGIN PASSWORD \$\$${K1S0_API_REGISTRY_PASSWORD:-dev-api-registry}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# app-registry サービス専用ロール
# 使用サービス: regions/system/server/rust/app-registry（アプリバージョン管理）
# app_registry スキーマのみ DML 可
$PSQL_CMD -c "CREATE ROLE k1s0_app_registry_rw WITH LOGIN PASSWORD \$\$${K1S0_APP_REGISTRY_PASSWORD:-dev-app-registry}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# HIGH-003 監査対応: master-maintenance サービス専用ロール追加
# 使用サービス: regions/system/server/rust/master-maintenance（マスターデータメンテナンス）
# master_maintenance スキーマのみ DML 可。Terraform の roles.tf と同様の定義。
$PSQL_CMD -c "CREATE ROLE k1s0_master_maintenance_rw WITH LOGIN PASSWORD \$\$${K1S0_MASTER_MAINTENANCE_PASSWORD:-dev-master-maintenance}\$\$ NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;"

# 権限付与は 99-finalize-grants.sh で全スキーマ作成後に実行する（HIGH-008 監査対応）
# スキーマ作成（02〜22番台の SQL ファイル）が完了した後に権限付与することで、
# IF EXISTS ガードによる権限付与スキップを防止する。
echo "ロール作成完了。権限付与は 99-finalize-grants.sh で実行されます。"
