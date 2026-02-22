# Vault Policy for dlq-manager
# Tier: system
# Description: Provides read access to dlq-manager specific secrets

# DLQ manager secrets
path "secret/data/k1s0/system/dlq-manager/*" {
  capabilities = ["read"]
}

# DLQ manager metadata
path "secret/metadata/k1s0/system/dlq-manager/*" {
  capabilities = ["read", "list"]
}

# Shared database credentials
path "secret/data/k1s0/system/database" {
  capabilities = ["read"]
}

# Shared Kafka credentials
path "secret/data/k1s0/system/kafka/*" {
  capabilities = ["read"]
}
