# Vault Module - Policy Configuration
# Loads tier-based access policies from HCL files.

# system Tier policy - access to secret/data/k1s0/system/*, database/creds/system-*, pki/issue/system
resource "vault_policy" "system" {
  name   = "system"
  policy = file("${path.module}/policies/system.hcl")
}

# business Tier policy - access to secret/data/k1s0/business/*, database/creds/business-*, kafka SASL
resource "vault_policy" "business" {
  name   = "business"
  policy = file("${path.module}/policies/business.hcl")
}

# service Tier policy - access to secret/data/k1s0/service/*, database/creds/service-*, kafka SASL
resource "vault_policy" "service" {
  name   = "service"
  policy = file("${path.module}/policies/service.hcl")
}

# CI/CD pipeline policy - limited access for AppRole auth
resource "vault_policy" "cicd" {
  name = "cicd"
  policy = <<-EOT
    # CI/CD pipeline policy
    # Read-only access to secrets needed for deployment

    path "secret/data/k1s0/*" {
      capabilities = ["read", "list"]
    }

    path "pki/issue/*" {
      capabilities = ["create", "update"]
    }
  EOT
}
