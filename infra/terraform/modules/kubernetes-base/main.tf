resource "kubernetes_namespace" "tier" {
  for_each = var.namespaces

  metadata {
    name = each.key
    labels = {
      tier       = each.value.tier
      managed-by = "terraform"
    }
  }
}

resource "kubernetes_network_policy" "deny_cross_tier" {
  for_each = var.namespaces

  metadata {
    name      = "deny-cross-tier"
    namespace = each.key
  }

  spec {
    pod_selector {}
    policy_types = ["Ingress"]

    # 許可する Tier からのインバウンド（複数 Tier を指定可能）
    dynamic "ingress" {
      for_each = length(each.value.allowed_from_tiers) > 0 ? [1] : []
      content {
        dynamic "from" {
          for_each = each.value.allowed_from_tiers
          content {
            namespace_selector {
              match_labels = {
                tier = from.value
              }
            }
          }
        }
        # 同一 Tier 内の通信も許可
        from {
          namespace_selector {
            match_labels = {
              tier = each.value.tier
            }
          }
        }
      }
    }
  }

  depends_on = [kubernetes_namespace.tier]
}

# ============================================================
# RBAC - ClusterRoles
# ============================================================

resource "kubernetes_cluster_role" "k1s0_admin" {
  metadata {
    name = "k1s0-admin"
    labels = {
      managed-by = "terraform"
    }
  }

  rule {
    api_groups = ["*"]
    resources  = ["*"]
    verbs      = ["*"]
  }
}

resource "kubernetes_cluster_role" "k1s0_operator" {
  metadata {
    name = "k1s0-operator"
    labels = {
      managed-by = "terraform"
    }
  }

  rule {
    api_groups = [""]
    resources  = ["pods", "services", "configmaps", "secrets"]
    verbs      = ["get", "list", "watch", "create", "update", "delete"]
  }

  rule {
    api_groups = ["apps"]
    resources  = ["deployments", "statefulsets"]
    verbs      = ["get", "list", "watch", "create", "update", "patch", "delete"]
  }
}

resource "kubernetes_cluster_role" "k1s0_developer" {
  metadata {
    name = "k1s0-developer"
    labels = {
      managed-by = "terraform"
    }
  }

  rule {
    api_groups = [""]
    resources  = ["pods", "services", "configmaps"]
    verbs      = ["get", "list", "watch"]
  }

  rule {
    api_groups = ["apps"]
    resources  = ["deployments"]
    verbs      = ["get", "list", "watch"]
  }
}

resource "kubernetes_cluster_role" "readonly" {
  metadata {
    name = "readonly"
    labels = {
      managed-by = "terraform"
    }
  }

  rule {
    api_groups = ["*"]
    resources  = ["*"]
    verbs      = ["get", "list", "watch"]
  }
}

# ============================================================
# RBAC - ClusterRoleBindings
# ============================================================

resource "kubernetes_cluster_role_binding" "k1s0_admin" {
  metadata {
    name = "k1s0-admin-binding"
    labels = {
      managed-by = "terraform"
    }
  }

  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = kubernetes_cluster_role.k1s0_admin.metadata[0].name
  }

  subject {
    kind = "Group"
    name = "k1s0-admin"
  }
}

resource "kubernetes_cluster_role_binding" "k1s0_operator" {
  metadata {
    name = "k1s0-operator-binding"
    labels = {
      managed-by = "terraform"
    }
  }

  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = kubernetes_cluster_role.k1s0_operator.metadata[0].name
  }

  subject {
    kind = "Group"
    name = "k1s0-operator"
  }
}

resource "kubernetes_cluster_role_binding" "k1s0_developer" {
  metadata {
    name = "k1s0-developer-binding"
    labels = {
      managed-by = "terraform"
    }
  }

  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = kubernetes_cluster_role.k1s0_developer.metadata[0].name
  }

  subject {
    kind = "Group"
    name = "k1s0-developer"
  }
}

resource "kubernetes_cluster_role_binding" "readonly" {
  metadata {
    name = "readonly-binding"
    labels = {
      managed-by = "terraform"
    }
  }

  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = kubernetes_cluster_role.readonly.metadata[0].name
  }

  subject {
    kind = "Group"
    name = "readonly"
  }
}

# ============================================================
# LimitRange - Default resource limits per Namespace
# ============================================================

resource "kubernetes_limit_range" "default_limits" {
  for_each = {
    for k, v in var.namespaces : k => v
    if contains(["system", "business", "service"], v.tier)
  }

  metadata {
    name      = "default-limits"
    namespace = each.key
  }

  spec {
    limit {
      type = "Container"

      default = {
        cpu    = "1"
        memory = "1Gi"
      }

      default_request = {
        cpu    = "250m"
        memory = "256Mi"
      }
    }
  }

  depends_on = [kubernetes_namespace.tier]
}

# ============================================================
# ResourceQuota - Namespace-level resource caps
# ============================================================

resource "kubernetes_resource_quota" "namespace_quota" {
  for_each = {
    for k, v in var.namespaces : k => v
    if contains(["system", "business", "service"], v.tier)
  }

  metadata {
    name      = "namespace-quota"
    namespace = each.key
  }

  spec {
    hard = {
      "requests.cpu"            = lookup(var.resource_quotas, each.key, var.default_resource_quota).requests_cpu
      "requests.memory"         = lookup(var.resource_quotas, each.key, var.default_resource_quota).requests_memory
      "limits.cpu"              = lookup(var.resource_quotas, each.key, var.default_resource_quota).limits_cpu
      "limits.memory"           = lookup(var.resource_quotas, each.key, var.default_resource_quota).limits_memory
      "pods"                    = lookup(var.resource_quotas, each.key, var.default_resource_quota).pods
      "persistentvolumeclaims"  = lookup(var.resource_quotas, each.key, var.default_resource_quota).pvcs
    }
  }

  depends_on = [kubernetes_namespace.tier]
}
