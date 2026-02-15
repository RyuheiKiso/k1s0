# Vault Module - Variables

variable "vault_address" {
  description = "Vault server address"
  type        = string
  default     = "https://vault.k1s0-system.svc.cluster.local:8200"
}

variable "kubernetes_host" {
  description = "Kubernetes API server address"
  type        = string
  default     = "https://kubernetes.default.svc"
}

variable "ldap_url" {
  description = "LDAP server URL (LDAPS)"
  type        = string
  default     = "ldaps://ldap.example.com:636"
}

variable "ldap_user_dn" {
  description = "LDAP user DN for user search"
  type        = string
  default     = "ou=users,dc=example,dc=com"
}

variable "ldap_group_dn" {
  description = "LDAP group DN for group search"
  type        = string
  default     = "ou=groups,dc=example,dc=com"
}

variable "ldap_bind_dn" {
  description = "LDAP bind DN for Vault authentication"
  type        = string
  default     = "cn=vault,ou=service-accounts,dc=example,dc=com"
}

variable "ldap_bind_password" {
  description = "LDAP bind password for Vault authentication"
  type        = string
  sensitive   = true
}
