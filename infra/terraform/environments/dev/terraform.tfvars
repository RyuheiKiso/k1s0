namespaces = {
  "k1s0-system" = {
    tier               = "system"
    allowed_from_tiers = ["system", "business", "service"]
  }
  "k1s0-business" = {
    tier               = "business"
    allowed_from_tiers = ["business", "service"]
  }
  "k1s0-service" = {
    tier               = "service"
    allowed_from_tiers = ["service"]
  }
  "observability" = {
    tier               = "infra"
    allowed_from_tiers = ["system", "business", "service"]
  }
  "messaging" = {
    tier               = "infra"
    allowed_from_tiers = ["system", "business", "service"]
  }
  "ingress" = {
    tier               = "infra"
    allowed_from_tiers = []
  }
  "service-mesh" = {
    tier               = "infra"
    allowed_from_tiers = []
  }
  "cert-manager" = {
    tier               = "infra"
    allowed_from_tiers = []
  }
  "harbor" = {
    tier               = "infra"
    allowed_from_tiers = []
  }
}

ceph_cluster_id  = "ceph-dev"
ceph_pool        = "k8s-block-dev"
reclaim_policy   = "Delete"
