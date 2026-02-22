# Public SPA client (Authorization Code + PKCE)
resource "keycloak_openid_client" "react_spa" {
  realm_id  = keycloak_realm.k1s0.id
  client_id = "react-spa"
  name      = "React SPA"

  enabled                      = true
  access_type                  = "PUBLIC"
  standard_flow_enabled        = true
  implicit_flow_enabled        = false
  direct_access_grants_enabled = false

  valid_redirect_uris = var.react_spa_redirect_uris
  web_origins         = var.react_spa_web_origins

  pkce_code_challenge_method = "S256"
}

# Confidential BFF client
resource "keycloak_openid_client" "k1s0_bff" {
  realm_id  = keycloak_realm.k1s0.id
  client_id = "k1s0-bff"
  name      = "k1s0 BFF"

  enabled                      = true
  access_type                  = "CONFIDENTIAL"
  standard_flow_enabled        = true
  implicit_flow_enabled        = false
  direct_access_grants_enabled = false
  service_accounts_enabled     = true

  valid_redirect_uris = var.bff_redirect_uris
}

# Audience mapper for BFF client
resource "keycloak_openid_audience_protocol_mapper" "bff_audience" {
  realm_id  = keycloak_realm.k1s0.id
  client_id = keycloak_openid_client.k1s0_bff.id
  name      = "k1s0-api-audience"

  included_custom_audience = "k1s0-api"
}

# Audience mapper for SPA client
resource "keycloak_openid_audience_protocol_mapper" "spa_audience" {
  realm_id  = keycloak_realm.k1s0.id
  client_id = keycloak_openid_client.react_spa.id
  name      = "k1s0-api-audience"

  included_custom_audience = "k1s0-api"
}
