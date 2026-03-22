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
resource "vault_database_secret_backend_role" "task_rw" {
  backend             = "database"
  name                = "service-task-rw"
  db_name             = vault_database_secret_backend_connection.task_db.name
  creation_statements = ["CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT ALL ON ALL TABLES IN SCHEMA public TO \"{{name}}\";"]
  # 動的クレデンシャルのリース失効時にロールと権限を確実に削除する
  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM \"{{name}}\";",
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public FROM \"{{name}}\";",
    "REVOKE USAGE ON SCHEMA public FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
  default_ttl         = 86400  # 24 hours
  max_ttl             = 172800 # 48 hours
}

# Task service - Read-Only role
resource "vault_database_secret_backend_role" "task_ro" {
  backend             = "database"
  name                = "service-task-ro"
  db_name             = vault_database_secret_backend_connection.task_db.name
  creation_statements = ["CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT SELECT ON ALL TABLES IN SCHEMA public TO \"{{name}}\";"]
  # 動的クレデンシャルのリース失効時にロールと権限を確実に削除する
  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM \"{{name}}\";",
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public FROM \"{{name}}\";",
    "REVOKE USAGE ON SCHEMA public FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
  default_ttl         = 86400  # 24 hours
  max_ttl             = 172800 # 48 hours
}

# Auth service - Read/Write role
resource "vault_database_secret_backend_role" "auth_rw" {
  backend             = "database"
  name                = "system-auth-rw"
  db_name             = vault_database_secret_backend_connection.auth_db.name
  creation_statements = ["CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT ALL ON ALL TABLES IN SCHEMA public TO \"{{name}}\";"]
  # 動的クレデンシャルのリース失効時にロールと権限を確実に削除する
  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM \"{{name}}\";",
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public FROM \"{{name}}\";",
    "REVOKE USAGE ON SCHEMA public FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
  default_ttl         = 86400  # 24 hours
  max_ttl             = 172800 # 48 hours
}

# Auth service - Read-Only role
resource "vault_database_secret_backend_role" "auth_ro" {
  backend             = "database"
  name                = "system-auth-ro"
  db_name             = vault_database_secret_backend_connection.auth_db.name
  creation_statements = ["CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT SELECT ON ALL TABLES IN SCHEMA public TO \"{{name}}\";"]
  # 動的クレデンシャルのリース失効時にロールと権限を確実に削除する
  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM \"{{name}}\";",
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public FROM \"{{name}}\";",
    "REVOKE USAGE ON SCHEMA public FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
  default_ttl         = 86400  # 24 hours
  max_ttl             = 172800 # 48 hours
}
