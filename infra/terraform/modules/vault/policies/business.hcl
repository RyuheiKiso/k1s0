# business Tier Vault Policy
# Grants access to business-tier secrets, database credentials, and Kafka SASL.

# KV v2 - business tier static secrets
path "secret/data/k1s0/business/*" {
  capabilities = ["read", "list"]
}

# Database - business tier dynamic credentials
path "database/creds/business-*" {
  capabilities = ["read"]
}

# Kafka SASL credentials (cross-tier access)
path "secret/data/k1s0/system/kafka/sasl" {
  capabilities = ["read"]
}
