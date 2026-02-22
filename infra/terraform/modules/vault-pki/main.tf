# Vault PKI Secret Engine Module
# Manages internal TLS certificate issuance for k1s0 services.

terraform {
  required_providers {
    vault = {
      source  = "hashicorp/vault"
      version = "~> 4.0"
    }
  }
}

# ============================================================
# Root CA
# ============================================================

resource "vault_mount" "pki" {
  path                      = "pki"
  type                      = "pki"
  description               = "PKI secret engine for internal TLS certificates"
  default_lease_ttl_seconds = 86400      # 24 hours
  max_lease_ttl_seconds     = 315360000  # 10 years
}

resource "vault_pki_secret_backend_root_cert" "root" {
  backend              = vault_mount.pki.path
  type                 = "internal"
  common_name          = "k1s0 Internal CA"
  ttl                  = "315360000"
  format               = "pem"
  private_key_format   = "der"
  key_type             = "rsa"
  key_bits             = 4096
  exclude_cn_from_sans = true
  ou                   = "k1s0 Platform"
  organization         = "k1s0"
}

# ============================================================
# Intermediate CA
# ============================================================

resource "vault_mount" "pki_int" {
  path                      = "pki_int"
  type                      = "pki"
  description               = "PKI intermediate CA for service certificate issuance"
  default_lease_ttl_seconds = 86400     # 24 hours
  max_lease_ttl_seconds     = 157680000 # 5 years
}

resource "vault_pki_secret_backend_intermediate_cert_request" "intermediate" {
  backend     = vault_mount.pki_int.path
  type        = "internal"
  common_name = "k1s0 Intermediate CA"
  key_type    = "rsa"
  key_bits    = 4096
}

resource "vault_pki_secret_backend_root_sign_intermediate" "intermediate" {
  backend              = vault_mount.pki.path
  csr                  = vault_pki_secret_backend_intermediate_cert_request.intermediate.csr
  common_name          = "k1s0 Intermediate CA"
  exclude_cn_from_sans = true
  ou                   = "k1s0 Platform"
  organization         = "k1s0"
  ttl                  = "157680000"
}

resource "vault_pki_secret_backend_intermediate_set_signed" "intermediate" {
  backend     = vault_mount.pki_int.path
  certificate = vault_pki_secret_backend_root_sign_intermediate.intermediate.certificate
}

# ============================================================
# PKI Roles (Certificate Issuance Policies)
# ============================================================

# system Tier services
resource "vault_pki_secret_backend_role" "system" {
  backend          = vault_mount.pki_int.path
  name             = "system"
  allowed_domains  = ["k1s0-system.svc.cluster.local"]
  allow_subdomains = true
  max_ttl          = var.system_cert_max_ttl
  key_type         = "rsa"
  key_bits         = 2048
  require_cn       = true
  generate_lease   = true
}

# business Tier services
resource "vault_pki_secret_backend_role" "business" {
  backend          = vault_mount.pki_int.path
  name             = "business"
  allowed_domains  = ["k1s0-business.svc.cluster.local"]
  allow_subdomains = true
  max_ttl          = var.business_cert_max_ttl
  key_type         = "rsa"
  key_bits         = 2048
  require_cn       = true
  generate_lease   = true
}

# service Tier services
resource "vault_pki_secret_backend_role" "service" {
  backend          = vault_mount.pki_int.path
  name             = "service"
  allowed_domains  = ["k1s0-service.svc.cluster.local"]
  allow_subdomains = true
  max_ttl          = var.service_cert_max_ttl
  key_type         = "rsa"
  key_bits         = 2048
  require_cn       = true
  generate_lease   = true
}
