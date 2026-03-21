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
  validation {
    # 本番環境ではプレースホルダードメインの使用を禁止する
    condition     = !can(regex("example\\.com", var.ldap_url))
    error_message = "本番環境では example.com プレースホルダーを使用できません。実際のドメインを指定してください。"
  }
}

variable "ldap_user_dn" {
  description = "LDAP user DN for user search"
  type        = string
  validation {
    # 本番環境ではプレースホルダードメインの使用を禁止する
    condition     = !can(regex("example\\.com", var.ldap_user_dn))
    error_message = "本番環境では example.com プレースホルダーを使用できません。実際のドメインを指定してください。"
  }
}

variable "ldap_group_dn" {
  description = "LDAP group DN for group search"
  type        = string
  validation {
    # 本番環境ではプレースホルダードメインの使用を禁止する
    condition     = !can(regex("example\\.com", var.ldap_group_dn))
    error_message = "本番環境では example.com プレースホルダーを使用できません。実際のドメインを指定してください。"
  }
}

variable "ldap_bind_dn" {
  description = "LDAP bind DN for Vault authentication"
  type        = string
  validation {
    # 本番環境ではプレースホルダードメインの使用を禁止する
    condition     = !can(regex("example\\.com", var.ldap_bind_dn))
    error_message = "本番環境では example.com プレースホルダーを使用できません。実際のドメインを指定してください。"
  }
}

variable "ldap_bind_password" {
  description = "LDAP bind password for Vault authentication"
  type        = string
  sensitive   = true
}
