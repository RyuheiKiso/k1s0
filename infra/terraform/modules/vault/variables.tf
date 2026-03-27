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

# H-5 監査対応: CA 証明書パスのハードコードを解消する。
# kubernetes_ca_cert が空文字列の場合は Pod 内のサービスアカウントパスから読み込む（Kubernetes Pod内実行時のデフォルト動作）。
# ローカル開発/CI 環境では data source や変数で証明書 PEM を渡してください。
variable "kubernetes_ca_cert" {
  description = "Kubernetes CA certificate PEM content. Empty = read from Pod service account path (/var/run/secrets/kubernetes.io/serviceaccount/ca.crt)"
  type        = string
  default     = ""
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
