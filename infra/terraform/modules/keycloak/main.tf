terraform {
  required_providers {
    keycloak = {
      source  = "mrparkers/keycloak"
      version = ">= 4.0.0"
    }
  }
}

provider "keycloak" {
  client_id = "admin-cli"
  url       = var.keycloak_url
}

resource "keycloak_realm" "k1s0" {
  realm   = var.realm_name
  enabled = true

  display_name = "k1s0"

  login_theme   = "keycloak"
  account_theme = "keycloak.v2"

  access_token_lifespan = "5m"
  sso_session_idle_timeout = "30m"
  sso_session_max_lifespan = "10h"

  internationalization {
    supported_locales = ["en", "ja"]
    default_locale    = "ja"
  }
}
