resource "helm_release" "kiali" {
  name       = "kiali"
  namespace  = "service-mesh"
  repository = "https://kiali.org/helm-charts"
  chart      = "kiali-server"
  version    = var.kiali_version

  values = [file("${path.module}/values/kiali.yaml")]

  depends_on = [helm_release.istiod]
}
