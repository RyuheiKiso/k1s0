output "harbor_url" {
  description = "Harbor external URL"
  value       = "https://${var.harbor_domain}"
}
