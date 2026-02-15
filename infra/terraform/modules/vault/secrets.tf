# Vault Module - Secret Engine Configuration
# Configures database connections and roles for dynamic credential generation.

# ============================================================
# Database Secret Backend Connections
# ============================================================

# Order service database connection (service Tier)
resource "vault_database_secret_backend_connection" "order_db" {
  backend       = vault_mount.database.path
  name          = "service-order"
  allowed_roles = ["service-order-rw", "service-order-ro"]

  postgresql {
    connection_url = "postgresql://{{username}}:{{password}}@postgres.k1s0-service.svc.cluster.local:5432/order_db"
  }
}

# Auth service database connection (system Tier)
resource "vault_database_secret_backend_connection" "auth_db" {
  backend       = vault_mount.database.path
  name          = "system-auth"
  allowed_roles = ["system-auth-rw", "system-auth-ro"]

  postgresql {
    connection_url = "postgresql://{{username}}:{{password}}@postgres.k1s0-system.svc.cluster.local:5432/auth_db"
  }
}

# ============================================================
# Database Secret Backend Roles
# ============================================================

# Order service - Read/Write role
resource "vault_database_secret_backend_role" "order_rw" {
  backend             = vault_mount.database.path
  name                = "service-order-rw"
  db_name             = vault_database_secret_backend_connection.order_db.name
  creation_statements = ["CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT ALL ON ALL TABLES IN SCHEMA public TO \"{{name}}\";"]
  default_ttl         = 86400  # 24 hours
  max_ttl             = 172800 # 48 hours
}

# Order service - Read-Only role
resource "vault_database_secret_backend_role" "order_ro" {
  backend             = vault_mount.database.path
  name                = "service-order-ro"
  db_name             = vault_database_secret_backend_connection.order_db.name
  creation_statements = ["CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT SELECT ON ALL TABLES IN SCHEMA public TO \"{{name}}\";"]
  default_ttl         = 86400  # 24 hours
  max_ttl             = 172800 # 48 hours
}

# Auth service - Read/Write role
resource "vault_database_secret_backend_role" "auth_rw" {
  backend             = vault_mount.database.path
  name                = "system-auth-rw"
  db_name             = vault_database_secret_backend_connection.auth_db.name
  creation_statements = ["CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT ALL ON ALL TABLES IN SCHEMA public TO \"{{name}}\";"]
  default_ttl         = 86400  # 24 hours
  max_ttl             = 172800 # 48 hours
}

# Auth service - Read-Only role
resource "vault_database_secret_backend_role" "auth_ro" {
  backend             = vault_mount.database.path
  name                = "system-auth-ro"
  db_name             = vault_database_secret_backend_connection.auth_db.name
  creation_statements = ["CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT SELECT ON ALL TABLES IN SCHEMA public TO \"{{name}}\";"]
  default_ttl         = 86400  # 24 hours
  max_ttl             = 172800 # 48 hours
}
