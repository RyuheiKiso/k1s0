variable "keycloak_url" {
  description = "Keycloak server URL"
  type        = string
}

variable "realm_name" {
  description = "Keycloak realm name"
  type        = string
  default     = "k1s0"
}

variable "react_spa_redirect_uris" {
  description = "Valid redirect URIs for the React SPA client"
  type        = list(string)
  default     = ["http://localhost:3000/*"]
}

variable "react_spa_web_origins" {
  description = "Allowed CORS origins for the React SPA client"
  type        = list(string)
  default     = ["http://localhost:3000"]
}

variable "bff_redirect_uris" {
  description = "Valid redirect URIs for the BFF client"
  type        = list(string)
  default     = ["http://localhost:8080/callback"]
}
