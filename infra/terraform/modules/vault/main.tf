# Vault Module - Main Configuration
# Manages HashiCorp Vault setup for the k1s0 project.
# Includes secret engines, audit logging, and base configuration.

terraform {
  required_providers {
    vault = {
      source  = "hashicorp/vault"
      version = "~> 4.0"
    }
  }
}

# ============================================================
# Secret Engines
# ============================================================

# KV v2 Secret Engine - Static secrets (API keys, config values, etc.)
resource "vault_mount" "kv" {
  path        = "secret"
  type        = "kv-v2"
  description = "KV v2 secret engine for static secrets"
}

# Database Secret Engine - Dynamic database credential generation
resource "vault_mount" "database" {
  path        = "database"
  type        = "database"
  description = "Database secret engine for dynamic credential generation"
}

# PKI Secret Engine - Internal TLS certificate issuance
resource "vault_mount" "pki" {
  path                  = "pki"
  type                  = "pki"
  description           = "PKI secret engine for internal TLS certificates"
  max_lease_ttl_seconds = 315360000 # 10 years
}

# ============================================================
# Audit Configuration
# ============================================================

# Audit log - records all authentication attempts, secret reads,
# policy changes, and configuration changes.
# Secret values are masked (log_raw = false).
resource "vault_audit" "file" {
  type = "file"
  options = {
    file_path = "/vault/logs/audit.log"
    log_raw   = "false"
  }
}
