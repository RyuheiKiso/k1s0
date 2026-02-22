# Vault Policy for saga-server
# Tier: system
# Description: Provides read access to saga-server specific secrets

# Saga server secrets
path "secret/data/k1s0/system/saga-server/*" {
  capabilities = ["read"]
}

# Saga server metadata
path "secret/metadata/k1s0/system/saga-server/*" {
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
