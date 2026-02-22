# Vault Policy for bff-proxy
# Tier: system
# Description: Provides access to bff-proxy specific secrets and PKI certificates

# bff-proxy static secrets
path "secret/data/k1s0/system/bff-proxy/*" {
  capabilities = ["read"]
}

# bff-proxy metadata
path "secret/metadata/k1s0/system/bff-proxy/*" {
  capabilities = ["read", "list"]
}

# Session store credentials (Redis)
path "secret/data/k1s0/system/redis/*" {
  capabilities = ["read"]
}

# Keycloak OIDC client secrets
path "secret/data/k1s0/system/keycloak/bff-proxy" {
  capabilities = ["read"]
}

# PKI certificate issuance (system tier)
path "pki_int/issue/system" {
  capabilities = ["create", "update"]
}

# Upstream service auth secrets
path "secret/data/k1s0/system/service-auth/*" {
  capabilities = ["read"]
}
