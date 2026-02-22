variable "prometheus_version" {
  description = "kube-prometheus-stack Helm chart version"
  type        = string
}

variable "loki_version" {
  description = "loki-stack Helm chart version"
  type        = string
}

variable "jaeger_version" {
  description = "Jaeger Helm chart version"
  type        = string
}

variable "otel_collector_version" {
  description = "OpenTelemetry Collector Helm chart version"
  type        = string
}
