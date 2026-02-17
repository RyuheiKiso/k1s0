# Vault Policy for k1s0 system tier
# Tier: system
# Description: Provides read access to all system tier secrets under secret/data/k1s0/system/*

# System tier secrets (wildcard read access)
path "secret/data/k1s0/system/*" {
  capabilities = ["read"]
}

# System tier metadata (for listing and checking secret existence)
path "secret/metadata/k1s0/system/*" {
  capabilities = ["read", "list"]
}
