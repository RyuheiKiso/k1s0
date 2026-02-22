output "realm_id" {
  description = "Keycloak realm ID"
  value       = keycloak_realm.k1s0.id
}

output "react_spa_client_id" {
  description = "React SPA client ID"
  value       = keycloak_openid_client.react_spa.client_id
}

output "bff_client_id" {
  description = "BFF client ID"
  value       = keycloak_openid_client.k1s0_bff.client_id
}

output "bff_client_secret" {
  description = "BFF client secret"
  value       = keycloak_openid_client.k1s0_bff.client_secret
  sensitive   = true
}
