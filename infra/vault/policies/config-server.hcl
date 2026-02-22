# Vault Policy for config-server
# Tier: system
# Description: Provides access to config-server specific secrets and database credentials

# config-server static secrets
path "secret/data/k1s0/system/config-server/*" {
  capabilities = ["read"]
}

# config-server metadata
path "secret/metadata/k1s0/system/config-server/*" {
  capabilities = ["read", "list"]
}

# Shared database credentials (static)
path "secret/data/k1s0/system/database" {
  capabilities = ["read"]
}

# Dynamic database credentials (read-write)
path "database/creds/config-server-rw" {
  capabilities = ["read"]
}

# Dynamic database credentials (read-only)
path "database/creds/config-server-ro" {
  capabilities = ["read"]
}

# PKI certificate issuance (system tier)
path "pki_int/issue/system" {
  capabilities = ["create", "update"]
}

# Shared Kafka credentials
path "secret/data/k1s0/system/kafka/*" {
  capabilities = ["read"]
}
