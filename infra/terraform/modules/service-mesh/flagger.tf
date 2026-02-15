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
