# Vault Database Secret Engine Module
# Manages dynamic PostgreSQL credential generation for all k1s0 services.

terraform {
  required_providers {
    vault = {
      source  = "hashicorp/vault"
      version = "~> 4.0"
    }
  }
}

# ============================================================
# Database Secret Engine Mount
# ============================================================

resource "vault_mount" "database" {
  path        = "database"
  type        = "database"
  description = "Database secret engine for dynamic credential generation"
}

# ============================================================
# PostgreSQL Connection
# ============================================================

resource "vault_database_secret_backend_connection" "postgres" {
  backend       = vault_mount.database.path
  name          = "k1s0-postgres"
  allowed_roles = ["auth-server-rw", "auth-server-ro", "config-server-rw", "config-server-ro", "saga-server-rw", "saga-server-ro", "dlq-manager-rw", "dlq-manager-ro"]

  postgresql {
    connection_url = "postgresql://{{username}}:{{password}}@${var.postgres_host}:${var.postgres_port}/postgres?sslmode=${var.postgres_ssl_mode}"
  }
}

# ============================================================
# auth-server Roles
# ============================================================

resource "vault_database_secret_backend_role" "auth_server_rw" {
  backend             = vault_mount.database.path
  name                = "auth-server-rw"
  db_name             = vault_database_secret_backend_connection.postgres.name
  creation_statements = [
    "CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}';",
    "GRANT ALL ON ALL TABLES IN SCHEMA auth TO \"{{name}}\";",
    "GRANT USAGE ON SCHEMA auth TO \"{{name}}\";"
  ]
  # 動的クレデンシャルのリース失効時にロールと権限を確実に削除する
  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA auth FROM \"{{name}}\";",
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA auth FROM \"{{name}}\";",
    "REVOKE USAGE ON SCHEMA auth FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
  default_ttl = var.credential_ttl
  max_ttl     = var.credential_max_ttl
}

resource "vault_database_secret_backend_role" "auth_server_ro" {
  backend             = vault_mount.database.path
  name                = "auth-server-ro"
  db_name             = vault_database_secret_backend_connection.postgres.name
  creation_statements = [
    "CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}';",
    "GRANT SELECT ON ALL TABLES IN SCHEMA auth TO \"{{name}}\";",
    "GRANT USAGE ON SCHEMA auth TO \"{{name}}\";"
  ]
  # 動的クレデンシャルのリース失効時にロールと権限を確実に削除する
  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA auth FROM \"{{name}}\";",
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA auth FROM \"{{name}}\";",
    "REVOKE USAGE ON SCHEMA auth FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
  default_ttl = var.credential_ttl
  max_ttl     = var.credential_max_ttl
}

# ============================================================
# config-server Roles
# ============================================================

resource "vault_database_secret_backend_role" "config_server_rw" {
  backend             = vault_mount.database.path
  name                = "config-server-rw"
  db_name             = vault_database_secret_backend_connection.postgres.name
  creation_statements = [
    "CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}';",
    "GRANT ALL ON ALL TABLES IN SCHEMA config TO \"{{name}}\";",
    "GRANT USAGE ON SCHEMA config TO \"{{name}}\";"
  ]
  # 動的クレデンシャルのリース失効時にロールと権限を確実に削除する
  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA config FROM \"{{name}}\";",
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA config FROM \"{{name}}\";",
    "REVOKE USAGE ON SCHEMA config FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
  default_ttl = var.credential_ttl
  max_ttl     = var.credential_max_ttl
}

resource "vault_database_secret_backend_role" "config_server_ro" {
  backend             = vault_mount.database.path
  name                = "config-server-ro"
  db_name             = vault_database_secret_backend_connection.postgres.name
  creation_statements = [
    "CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}';",
    "GRANT SELECT ON ALL TABLES IN SCHEMA config TO \"{{name}}\";",
    "GRANT USAGE ON SCHEMA config TO \"{{name}}\";"
  ]
  # 動的クレデンシャルのリース失効時にロールと権限を確実に削除する
  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA config FROM \"{{name}}\";",
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA config FROM \"{{name}}\";",
    "REVOKE USAGE ON SCHEMA config FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
  default_ttl = var.credential_ttl
  max_ttl     = var.credential_max_ttl
}

# ============================================================
# saga-server Roles
# ============================================================

resource "vault_database_secret_backend_role" "saga_server_rw" {
  backend             = vault_mount.database.path
  name                = "saga-server-rw"
  db_name             = vault_database_secret_backend_connection.postgres.name
  creation_statements = [
    "CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}';",
    "GRANT ALL ON ALL TABLES IN SCHEMA saga TO \"{{name}}\";",
    "GRANT USAGE ON SCHEMA saga TO \"{{name}}\";"
  ]
  # 動的クレデンシャルのリース失効時にロールと権限を確実に削除する
  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA saga FROM \"{{name}}\";",
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA saga FROM \"{{name}}\";",
    "REVOKE USAGE ON SCHEMA saga FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
  default_ttl = var.credential_ttl
  max_ttl     = var.credential_max_ttl
}

resource "vault_database_secret_backend_role" "saga_server_ro" {
  backend             = vault_mount.database.path
  name                = "saga-server-ro"
  db_name             = vault_database_secret_backend_connection.postgres.name
  creation_statements = [
    "CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}';",
    "GRANT SELECT ON ALL TABLES IN SCHEMA saga TO \"{{name}}\";",
    "GRANT USAGE ON SCHEMA saga TO \"{{name}}\";"
  ]
  # 動的クレデンシャルのリース失効時にロールと権限を確実に削除する
  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA saga FROM \"{{name}}\";",
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA saga FROM \"{{name}}\";",
    "REVOKE USAGE ON SCHEMA saga FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
  default_ttl = var.credential_ttl
  max_ttl     = var.credential_max_ttl
}

# ============================================================
# dlq-manager Roles
# ============================================================

resource "vault_database_secret_backend_role" "dlq_manager_rw" {
  backend             = vault_mount.database.path
  name                = "dlq-manager-rw"
  db_name             = vault_database_secret_backend_connection.postgres.name
  creation_statements = [
    "CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}';",
    "GRANT ALL ON ALL TABLES IN SCHEMA dlq TO \"{{name}}\";",
    "GRANT USAGE ON SCHEMA dlq TO \"{{name}}\";"
  ]
  # 動的クレデンシャルのリース失効時にロールと権限を確実に削除する
  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA dlq FROM \"{{name}}\";",
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA dlq FROM \"{{name}}\";",
    "REVOKE USAGE ON SCHEMA dlq FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
  default_ttl = var.credential_ttl
  max_ttl     = var.credential_max_ttl
}

resource "vault_database_secret_backend_role" "dlq_manager_ro" {
  backend             = vault_mount.database.path
  name                = "dlq-manager-ro"
  db_name             = vault_database_secret_backend_connection.postgres.name
  creation_statements = [
    "CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}';",
    "GRANT SELECT ON ALL TABLES IN SCHEMA dlq TO \"{{name}}\";",
    "GRANT USAGE ON SCHEMA dlq TO \"{{name}}\";"
  ]
  # 動的クレデンシャルのリース失効時にロールと権限を確実に削除する
  revocation_statements = [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA dlq FROM \"{{name}}\";",
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA dlq FROM \"{{name}}\";",
    "REVOKE USAGE ON SCHEMA dlq FROM \"{{name}}\";",
    "DROP ROLE IF EXISTS \"{{name}}\";"
  ]
  default_ttl = var.credential_ttl
  max_ttl     = var.credential_max_ttl
}
