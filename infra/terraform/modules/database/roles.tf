# サービス別DBロールとGRANT定義
# 最小権限原則に基づき、各サービスは自スキーマのみアクセス可能
# C-02（DB認証情報のシングルユーザー共有）対応として実装

# データベース名を参照するためのローカル変数
locals {
  # サービス名とスキーマ名のマッピング
  # MED-029 監査対応: system tier の全サービスを網羅するよう追加。
  # 不足していたサービス（featureflag/scheduler/file/master_maintenance 等）を追記した。
  service_schemas = {
    auth                = "auth"
    config              = "config"
    saga                = "saga"
    session             = "session"
    tenant              = "tenant"
    workflow            = "workflow"
    dlq                 = "dlq"
    notification        = "notification"
    vault               = "vault"
    # MED-029 監査対応: 以下のサービスが不足していたため追加する
    featureflag         = "featureflag"
    scheduler           = "scheduler"
    file                = "file"
    master_maintenance  = "master_maintenance"
    search              = "search"
    api_registry        = "api_registry"
    ratelimit           = "ratelimit"
    policy              = "policy"
    quota               = "quota"
    service_catalog     = "service_catalog"
    navigation          = "navigation"
    event_store         = "event_store"
    # LOW-009 監査対応: DB を使用するが MED-029 で追加漏れだったサービスを補完する
    event_monitor       = "event_monitor"
    app_registry        = "app_registry"
    rule_engine         = "rule_engine"
  }
}

# マイグレーション専用ロール（全スキーマのDDL権限を持つ）
# マイグレーション実行時のみ使用し、通常のサービス接続には使用しない
resource "postgresql_role" "migration" {
  name     = "k1s0_migration"
  login    = true
  password = var.migration_password
}

# サービス別読み書きロール（DMLのみ、DDL権限なし）
# 各サービスは自スキーマのテーブル・シーケンスのみ操作可能
resource "postgresql_role" "service_rw" {
  for_each = local.service_schemas
  name     = "k1s0_${each.key}_rw"
  login    = true
  password = var.service_passwords[each.key]
}

# マイグレーションロールへのスキーマ権限付与（CREATE + USAGE）
# DDL操作（CREATE TABLE等）を実行するために必要
resource "postgresql_grant" "migration_schema" {
  for_each    = local.service_schemas
  database    = var.database_name
  role        = postgresql_role.migration.name
  schema      = each.value
  object_type = "schema"
  privileges  = ["CREATE", "USAGE"]
  depends_on  = [postgresql_role.migration]
}

# サービスロールへのスキーマ USAGE 権限
# スキーマ内のオブジェクトにアクセスするために最低限必要な権限
resource "postgresql_grant" "service_schema_usage" {
  for_each    = local.service_schemas
  database    = var.database_name
  role        = postgresql_role.service_rw[each.key].name
  schema      = each.value
  object_type = "schema"
  privileges  = ["USAGE"]
  depends_on  = [postgresql_role.service_rw]
}

# サービスロールへのテーブル DML 権限
# MED-029 監査対応: DELETE 権限の方針について
# 現状は全サービスに一律 DELETE を付与しているが、最小権限原則では以下を推奨する:
#   - 論理削除（soft delete）のみのサービス: DELETE 不要（UPDATE のみ）
#   - 物理削除が必要なサービス: DELETE を明示的に付与
# 今後の改善案: サービスごとに privileges を分けた map を定義し、
# DELETE が不要なサービス（例: event_store, audit 系）では DELETE を除外すること。
# 現時点では運用上の安全性を優先して DELETE を含む全 DML を付与する。
# SELECT/INSERT/UPDATE/DELETE のみ許可し、DDL操作（DROP, ALTER等）は禁止
resource "postgresql_grant" "service_table_dml" {
  for_each    = local.service_schemas
  database    = var.database_name
  role        = postgresql_role.service_rw[each.key].name
  schema      = each.value
  object_type = "table"
  privileges  = ["SELECT", "INSERT", "UPDATE", "DELETE"]
  depends_on  = [postgresql_role.service_rw]
}

# サービスロールへのシーケンス権限（AUTO INCREMENT 対応）
# SERIAL / GENERATED ALWAYS AS IDENTITY 型カラムの採番に必要
resource "postgresql_grant" "service_sequence" {
  for_each    = local.service_schemas
  database    = var.database_name
  role        = postgresql_role.service_rw[each.key].name
  schema      = each.value
  object_type = "sequence"
  privileges  = ["USAGE", "SELECT"]
  depends_on  = [postgresql_role.service_rw]
}
