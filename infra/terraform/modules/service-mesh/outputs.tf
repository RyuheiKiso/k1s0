output "ingress_gateway_ip" {
  description = "Istio Ingress Gateway external IP"
  value       = helm_release.istio_ingress.status
}
