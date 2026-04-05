# Vault Module - Secret Engine Configuration
# Configures database connections and roles for dynamic credential generation.
# database mount は vault-database サブモジュールが canonical owner のため、
# backend には固定パス "database" を使用する。

# ============================================================
# Database Secret Backend Connections
# ============================================================

# Task service database connection (service Tier)
resource "vault_database_secret_backend_connection" "task_db" {
  backend       = "database"
  name          = "service-task"
  allowed_roles = ["service-task-rw", "service-task-ro"]

  postgresql {
    connection_url = "postgresql://{{username}}:{{password}}@postgresql.k1s0-service.svc.cluster.local:5432/task_db"
  }
}

# Auth service database connection (system Tier)
resource "vault_database_secret_backend_connection" "auth_db" {
  backend       = "database"
  name          = "system-auth"
  allowed_roles = ["system-auth-rw", "system-auth-ro"]

  postgresql {
    connection_url = "postgresql://{{username}}:{{password}}@postgresql.k1s0-system.svc.cluster.local:5432/k1s0_system"
  }
}

# ============================================================
# Database Secret Backend Roles
# ============================================================

# Task service - Read/Write role
# HIGH-007 監査対応: public スキーマへの GRANT を task_service 固有スキーマに変更する
# task サービスの migration で SET search_path TO task_service を使用しているため task_service スキーマを指定する
resource "vault_database_secret_backend_role" "task_rw" {
  backend             = "database"
  name                = "service-task-rw"
  db_name             = vault_database_secret_backend_connection.task_db.name
  creation_statements = ["CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT USAGE ON SCHEMA task_service TO \"{{name}}\"; GRANT ALL ON ALL TABLES IN SCHEMA task_service TO \"{{name}}\"; GRANT ALL ON ALL SEQUENCES IN SCHEMA task_service TO \"{{name}}\";"]
  # 動的クレデンシャルのリース失効時にロールと権限を確実に削除する
  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA task_service FROM \"{{name}}\";",
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA task_service FROM \"{{name}}\";",
    "REVOKE USAGE ON SCHEMA task_service FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
  # ゼロトラスト設計に基づき TTL を短縮する（漏洩時のリスク期間を最小化）
  default_ttl         = 3600   # 1 hour
  max_ttl             = 14400  # 4 hours
}

# Task service - Read-Only role
# HIGH-007 監査対応: public スキーマへの GRANT を task_service 固有スキーマに変更する
resource "vault_database_secret_backend_role" "task_ro" {
  backend             = "database"
  name                = "service-task-ro"
  db_name             = vault_database_secret_backend_connection.task_db.name
  creation_statements = ["CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT USAGE ON SCHEMA task_service TO \"{{name}}\"; GRANT SELECT ON ALL TABLES IN SCHEMA task_service TO \"{{name}}\";"]
  # 動的クレデンシャルのリース失効時にロールと権限を確実に削除する
  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA task_service FROM \"{{name}}\";",
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA task_service FROM \"{{name}}\";",
    "REVOKE USAGE ON SCHEMA task_service FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
  # ゼロトラスト設計に基づき TTL を短縮する（漏洩時のリスク期間を最小化）
  default_ttl         = 3600   # 1 hour
  max_ttl             = 14400  # 4 hours
}

# Auth service - Read/Write role
# HIGH-007 監査対応: public スキーマへの GRANT を auth 固有スキーマに変更する
# auth サービスの migration で CREATE SCHEMA IF NOT EXISTS auth を使用しているため auth スキーマを指定する
resource "vault_database_secret_backend_role" "auth_rw" {
  backend             = "database"
  name                = "system-auth-rw"
  db_name             = vault_database_secret_backend_connection.auth_db.name
  creation_statements = ["CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT USAGE ON SCHEMA auth TO \"{{name}}\"; GRANT ALL ON ALL TABLES IN SCHEMA auth TO \"{{name}}\"; GRANT ALL ON ALL SEQUENCES IN SCHEMA auth TO \"{{name}}\";"]
  # 動的クレデンシャルのリース失効時にロールと権限を確実に削除する
  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA auth FROM \"{{name}}\";",
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA auth FROM \"{{name}}\";",
    "REVOKE USAGE ON SCHEMA auth FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
  # ゼロトラスト設計に基づき TTL を短縮する（漏洩時のリスク期間を最小化）
  default_ttl         = 3600   # 1 hour
  max_ttl             = 14400  # 4 hours
}

# Auth service - Read-Only role
# HIGH-007 監査対応: public スキーマへの GRANT を auth 固有スキーマに変更する
resource "vault_database_secret_backend_role" "auth_ro" {
  backend             = "database"
  name                = "system-auth-ro"
  db_name             = vault_database_secret_backend_connection.auth_db.name
  creation_statements = ["CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT USAGE ON SCHEMA auth TO \"{{name}}\"; GRANT SELECT ON ALL TABLES IN SCHEMA auth TO \"{{name}}\";"]
  # 動的クレデンシャルのリース失効時にロールと権限を確実に削除する
  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA auth FROM \"{{name}}\";",
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA auth FROM \"{{name}}\";",
    "REVOKE USAGE ON SCHEMA auth FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
  # ゼロトラスト設計に基づき TTL を短縮する（漏洩時のリスク期間を最小化）
  default_ttl         = 3600   # 1 hour
  max_ttl             = 14400  # 4 hours
}
