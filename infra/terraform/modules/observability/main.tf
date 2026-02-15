resource "helm_release" "prometheus" {
  name       = "prometheus"
  namespace  = "observability"
  repository = "https://prometheus-community.github.io/helm-charts"
  chart      = "kube-prometheus-stack"
  version    = var.prometheus_version

  values = [file("${path.module}/values/prometheus.yaml")]
}

resource "helm_release" "loki" {
  name       = "loki"
  namespace  = "observability"
  repository = "https://grafana.github.io/helm-charts"
  chart      = "loki-stack"
  version    = var.loki_version

  values = [file("${path.module}/values/loki.yaml")]
}

resource "helm_release" "jaeger" {
  name       = "jaeger"
  namespace  = "observability"
  repository = "https://jaegertracing.github.io/helm-charts"
  chart      = "jaeger"
  version    = var.jaeger_version

  values = [file("${path.module}/values/jaeger.yaml")]
}
