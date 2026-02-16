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

# ============================================================
# Kubernetes Auth Backend
# ============================================================

# Enable Kubernetes authentication method for pod-based access
resource "vault_auth_backend" "kubernetes" {
  type = "kubernetes"
  path = "kubernetes"
}

# System Tier role - bound to k1s0-system namespace
resource "vault_kubernetes_auth_backend_role" "system" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "system"
  bound_service_account_names      = ["*"]
  bound_service_account_namespaces = ["k1s0-system"]
  token_ttl                        = 3600
  token_policies                   = ["system-read"]
}

# Business Tier role - bound to k1s0-business namespace
resource "vault_kubernetes_auth_backend_role" "business" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "business"
  bound_service_account_names      = ["*"]
  bound_service_account_namespaces = ["k1s0-business"]
  token_ttl                        = 3600
  token_policies                   = ["business-read"]
}

# Service Tier role - bound to k1s0-service namespace
resource "vault_kubernetes_auth_backend_role" "service" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "service"
  bound_service_account_names      = ["*"]
  bound_service_account_namespaces = ["k1s0-service"]
  token_ttl                        = 3600
  token_policies                   = ["service-read"]
}
