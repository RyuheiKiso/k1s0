# Vault Policy for auth-server
# Tier: system
# Description: Provides access to auth-server specific secrets, database credentials, and PKI

# auth-server static secrets
path "secret/data/k1s0/system/auth-server/*" {
  capabilities = ["read"]
}

# auth-server metadata
path "secret/metadata/k1s0/system/auth-server/*" {
  capabilities = ["read", "list"]
}

# Shared database credentials (static)
path "secret/data/k1s0/system/database" {
  capabilities = ["read"]
}

# Dynamic database credentials (read-write)
path "database/creds/auth-server-rw" {
  capabilities = ["read"]
}

# Dynamic database credentials (read-only)
path "database/creds/auth-server-ro" {
  capabilities = ["read"]
}

# PKI 証明書発行（サービス固有ロールに制限）
# 最小権限の原則: system tier 共通ロールではなく、auth-server 専用ロールで発行する
path "pki_int/issue/auth-server" {
  capabilities = ["create", "update"]
}

# Shared Kafka credentials
path "secret/data/k1s0/system/kafka/*" {
  capabilities = ["read"]
}

# Keycloak integration secrets
path "secret/data/k1s0/system/keycloak/*" {
  capabilities = ["read"]
}
