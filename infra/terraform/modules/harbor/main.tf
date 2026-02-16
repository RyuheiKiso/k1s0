terraform {
  required_providers {
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.11"
    }
    harbor = {
      source  = "goharbor/harbor"
      version = "~> 3.10"
    }
  }
}

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
