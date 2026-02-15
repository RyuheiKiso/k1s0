resource "helm_release" "istio_base" {
  name       = "istio-base"
  namespace  = "service-mesh"
  repository = "https://istio-release.storage.googleapis.com/charts"
  chart      = "base"
  version    = var.istio_version

  create_namespace = true
}

resource "helm_release" "istiod" {
  name       = "istiod"
  namespace  = "service-mesh"
  repository = "https://istio-release.storage.googleapis.com/charts"
  chart      = "istiod"
  version    = var.istio_version

  values = [file("${path.module}/values/istiod.yaml")]

  depends_on = [helm_release.istio_base]
}

resource "helm_release" "istio_ingress" {
  name       = "istio-ingress"
  namespace  = "service-mesh"
  repository = "https://istio-release.storage.googleapis.com/charts"
  chart      = "gateway"
  version    = var.istio_version

  values = [file("${path.module}/values/gateway.yaml")]

  depends_on = [helm_release.istiod]
}

# --- Kiali Dashboard ---

resource "helm_release" "kiali" {
  name       = "kiali"
  namespace  = "service-mesh"
  repository = "https://kiali.org/helm-charts"
  chart      = "kiali-server"
  version    = var.kiali_version

  values = [file("${path.module}/values/kiali.yaml")]

  depends_on = [helm_release.istiod]
}

# --- Flagger (Canary Deployment Controller) ---

resource "helm_release" "flagger" {
  name       = "flagger"
  namespace  = "service-mesh"
  repository = "https://flagger.app"
  chart      = "flagger"
  version    = var.flagger_version

  set {
    name  = "meshProvider"
    value = "istio"
  }

  set {
    name  = "metricsServer"
    value = "http://prometheus.observability.svc.cluster.local:9090"
  }

  values = [file("${path.module}/values/flagger.yaml")]

  depends_on = [helm_release.istiod]
}
