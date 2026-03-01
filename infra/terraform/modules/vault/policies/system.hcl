# system Tier Vault Policy
# Grants access to system-tier secrets, database credentials, and PKI.

# KV v2 - system tier static secrets
path "secret/data/k1s0/system/*" {
  capabilities = ["read", "list"]
}

# Database - system tier dynamic credentials
path "database/creds/system-*" {
  capabilities = ["read"]
}

# PKI - system tier certificate issuance (Intermediate CA)
path "pki_int/issue/system" {
  capabilities = ["create", "update"]
}
