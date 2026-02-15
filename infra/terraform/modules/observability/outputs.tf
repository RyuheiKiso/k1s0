output "prometheus_status" {
  description = "Status of the Prometheus Helm release"
  value       = helm_release.prometheus.status
}

output "loki_status" {
  description = "Status of the Loki Helm release"
  value       = helm_release.loki.status
}

output "jaeger_status" {
  description = "Status of the Jaeger Helm release"
  value       = helm_release.jaeger.status
}
