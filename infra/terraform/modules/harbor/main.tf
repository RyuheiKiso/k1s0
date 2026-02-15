resource "helm_release" "harbor" {
  name       = "harbor"
  namespace  = "harbor"
  repository = "https://helm.goharbor.io"
  chart      = "harbor"
  version    = var.harbor_chart_version

  create_namespace = true

  values = [file("${path.module}/values/harbor.yaml")]

  set {
    name  = "externalURL"
    value = "https://${var.harbor_domain}"
  }

  set {
    name  = "persistence.imageChartStorage.type"
    value = "s3"
  }

  set {
    name  = "persistence.imageChartStorage.s3.bucket"
    value = var.harbor_s3_bucket
  }

  set {
    name  = "persistence.imageChartStorage.s3.regionendpoint"
    value = var.ceph_s3_endpoint
  }
}

# --- Harbor Projects ---

resource "harbor_project" "k1s0" {
  for_each = toset(["k1s0-system", "k1s0-business", "k1s0-service", "k1s0-infra"])

  name   = each.key
  public = false

  depends_on = [helm_release.harbor]
}

# --- Harbor Robot Accounts (CI/CD) ---

resource "harbor_robot_account" "ci_push" {
  for_each = harbor_project.k1s0

  name        = "ci-push-${each.key}"
  description = "CI/CD push account for ${each.key}"
  level       = "project"

  permissions {
    access {
      action   = "push"
      resource = "repository"
    }
    access {
      action   = "pull"
      resource = "repository"
    }
    kind      = "project"
    namespace = each.value.name
  }

  depends_on = [harbor_project.k1s0]
}
