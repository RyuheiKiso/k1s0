# service Tier Vault Policy
# Grants access to service-tier secrets, database credentials, and Kafka SASL.

# KV v2 - service tier static secrets
path "secret/data/k1s0/service/*" {
  capabilities = ["read", "list"]
}

# Database - service tier dynamic credentials
path "database/creds/service-*" {
  capabilities = ["read"]
}

# Kafka SASL credentials (cross-tier access)
path "secret/data/k1s0/system/kafka/sasl" {
  capabilities = ["read"]
}
