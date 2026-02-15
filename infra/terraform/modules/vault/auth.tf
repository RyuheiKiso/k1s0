# Vault Module - Authentication Configuration
# Configures Kubernetes Auth, AppRole, and LDAP auth backends.

# ============================================================
# Kubernetes Auth Backend
# ============================================================

resource "vault_auth_backend" "kubernetes" {
  type = "kubernetes"
}

resource "vault_kubernetes_auth_backend_config" "k8s" {
  backend            = vault_auth_backend.kubernetes.path
  kubernetes_host    = "https://kubernetes.default.svc"
  kubernetes_ca_cert = file("/var/run/secrets/kubernetes.io/serviceaccount/ca.crt")
}

# system Tier role - binds to k1s0-system namespace
resource "vault_kubernetes_auth_backend_role" "system" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "system"
  bound_service_account_names      = ["*"]
  bound_service_account_namespaces = ["k1s0-system"]
  token_policies                   = ["system"]
  token_ttl                        = 3600
}

# business Tier role - binds to k1s0-business namespace
resource "vault_kubernetes_auth_backend_role" "business" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "business"
  bound_service_account_names      = ["*"]
  bound_service_account_namespaces = ["k1s0-business"]
  token_policies                   = ["business"]
  token_ttl                        = 3600
}

# service Tier role - binds to k1s0-service namespace
resource "vault_kubernetes_auth_backend_role" "service" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "service"
  bound_service_account_names      = ["*"]
  bound_service_account_namespaces = ["k1s0-service"]
  token_policies                   = ["service"]
  token_ttl                        = 3600
}

# ============================================================
# AppRole Auth Backend - for CI/CD pipelines
# ============================================================

resource "vault_auth_backend" "approle" {
  type = "approle"
}

resource "vault_approle_auth_backend_role" "cicd" {
  backend        = vault_auth_backend.approle.path
  role_name      = "cicd"
  token_policies = ["cicd"]
  token_ttl      = 1800
  token_max_ttl  = 3600
}

# ============================================================
# LDAP Auth Backend - for human operator access
# ============================================================

resource "vault_auth_backend" "ldap" {
  type = "ldap"
}

resource "vault_ldap_auth_backend" "ldap" {
  path         = vault_auth_backend.ldap.path
  url          = var.ldap_url
  userdn       = var.ldap_user_dn
  groupdn      = var.ldap_group_dn
  binddn       = var.ldap_bind_dn
  bindpass     = var.ldap_bind_password
  insecure_tls = false
  starttls     = true
  userattr     = "sAMAccountName"
  groupattr    = "memberOf"
}
