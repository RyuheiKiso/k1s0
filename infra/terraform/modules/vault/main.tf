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
# NOTE: database mount と pki mount はサブモジュール (vault-database, vault-pki) が canonical owner
# NOTE: kubernetes auth backend と roles は auth.tf が canonical owner
resource "vault_mount" "kv" {
  path        = "secret"
  type        = "kv-v2"
  description = "KV v2 secret engine for static secrets"
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
