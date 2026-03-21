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
  # Kubernetes API サーバーのエンドポイントを変数から参照する
  kubernetes_host    = var.kubernetes_host
  kubernetes_ca_cert = file("/var/run/secrets/kubernetes.io/serviceaccount/ca.crt")
}

# system Tier role - サービス別SA名で最小権限を適用
resource "vault_kubernetes_auth_backend_role" "system" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "system"
  bound_service_account_names      = [
    "auth-rust", "config-rust", "dlq-manager", "event-store-rust",
    "featureflag-rust", "file-rust", "graphql-gateway", "master-maintenance",
    "navigation-rust", "notification-rust", "policy-rust", "quota-rust",
    "ratelimit-rust", "rule-engine-rust", "saga-rust", "scheduler-rust",
    "search-rust", "service-catalog", "session-rust", "tenant-rust",
    "vault-rust", "workflow-rust", "event-monitor-rust", "app-registry",
    "api-registry-rust",
  ]
  bound_service_account_namespaces = ["k1s0-system"]
  token_policies                   = ["system"]
  token_ttl                        = 3600
  # トークンの最大有効期限を24時間に制限する（m-03対応: token_max_ttl 未設定の修正）
  token_max_ttl                    = 86400
}

# business Tier role - サービス別SA名で最小権限を適用
resource "vault_kubernetes_auth_backend_role" "business" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "business"
  bound_service_account_names      = [
    "inventory-rust", "payment-rust", "domain-master-go",
  ]
  bound_service_account_namespaces = ["k1s0-business"]
  token_policies                   = ["business"]
  token_ttl                        = 3600
  # トークンの最大有効期限を24時間に制限する（m-03対応: token_max_ttl 未設定の修正）
  token_max_ttl                    = 86400
}

# service Tier role - サービス別SA名で最小権限を適用
resource "vault_kubernetes_auth_backend_role" "service" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "service"
  bound_service_account_names      = [
    "order-rust", "bff-proxy",
  ]
  bound_service_account_namespaces = ["k1s0-service"]
  token_policies                   = ["service"]
  token_ttl                        = 3600
  # トークンの最大有効期限を24時間に制限する（m-03対応: token_max_ttl 未設定の修正）
  token_max_ttl                    = 86400
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
  # token_ttl を 7200（2時間）に延長する。
  # CI/CD パイプラインが長時間のビルド・デプロイ中にトークン期限切れで失敗する問題を防ぐ。
  # token_max_ttl（上限）は token_ttl と同じ 7200 に合わせて整合性を保つ。
  token_ttl      = 7200
  token_max_ttl  = 7200
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
