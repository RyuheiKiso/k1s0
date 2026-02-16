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

reclaim_policy   = "Retain"
ceph_pool        = "k8s-block-prod"
# 実環境の Ceph クラスタ ID に合わせて変更すること
ceph_cluster_id  = "prod-ceph-cluster-001"

resource_quotas = {
  "k1s0-system" = {
    requests_cpu    = "8"
    requests_memory = "16Gi"
    limits_cpu      = "16"
    limits_memory   = "32Gi"
    pods            = "50"
    pvcs            = "20"
  }
  "k1s0-business" = {
    requests_cpu    = "16"
    requests_memory = "32Gi"
    limits_cpu      = "32"
    limits_memory   = "64Gi"
    pods            = "100"
    pvcs            = "40"
  }
  "k1s0-service" = {
    requests_cpu    = "8"
    requests_memory = "16Gi"
    limits_cpu      = "16"
    limits_memory   = "32Gi"
    pods            = "50"
    pvcs            = "20"
  }
}
