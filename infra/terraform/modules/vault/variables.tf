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

# L-13 監査対応: LDAP 接続時の TLS 証明書検証に使用する CA 証明書（PEM 形式）。
# 空文字列の場合はシステムのデフォルト CA バンドルを使用する。
# 自己署名証明書や企業内 CA を使用する環境では必ず PEM 内容を設定すること。
variable "ldap_ca_cert" {
  description = "LDAP サーバーの TLS 証明書を検証するための CA 証明書（PEM 形式）。空文字列の場合はシステムデフォルトの CA バンドルを使用する。"
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
  # K8S-TF-001 監査対応: 平文 LDAP 通信を禁止し、TLS 必須の ldaps:// スキームを強制する。
  # ldap:// を許容すると通信が暗号化されず認証情報が平文で流れるリスクがある。
  validation {
    condition     = can(regex("^ldaps://", var.ldap_url))
    error_message = "LDAP URL は ldaps:// スキームを使用する必要があります。平文 ldap:// は許可されません。"
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
  validation {
    # LDAP バインドパスワードは空文字と短すぎるパスワードを拒否する（H-14 監査対応）
    condition     = length(var.ldap_bind_password) >= 8
    error_message = "ldap_bind_password は8文字以上である必要があります。"
  }
}

# H-02 / L-14 監査対応: サービス個別 Vault ロールで使用する Kubernetes namespace。
# system tier サービスがデプロイされる namespace を指定する。
# デフォルト値は k1s0-system（既存インフラとの互換性維持）。
variable "k8s_namespace" {
  description = "サービス個別 Vault ロールをバインドする Kubernetes namespace（system tier）"
  type        = string
  default     = "k1s0-system"
}
