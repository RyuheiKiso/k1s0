resource "kubernetes_namespace" "tier" {
  for_each = var.namespaces

  metadata {
    name = each.key
    labels = {
      tier        = each.value.tier
      managed-by  = "terraform"
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
