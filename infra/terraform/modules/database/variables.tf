variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
}

variable "database_namespace" {
  description = "Kubernetes namespace for database deployments"
  type        = string
  default     = "k1s0-system"
}

variable "enable_postgresql" {
  description = "Enable PostgreSQL deployment"
  type        = bool
  default     = true
}

variable "enable_mysql" {
  description = "Enable MySQL deployment"
  type        = bool
  default     = false
}

variable "postgresql_chart_version" {
  description = "Bitnami PostgreSQL Helm chart version"
  type        = string
}

variable "mysql_chart_version" {
  description = "Bitnami MySQL Helm chart version"
  type        = string
}

variable "postgresql_version" {
  description = "PostgreSQL image version"
  type        = string
  default     = "16"
}

variable "mysql_version" {
  description = "MySQL image version"
  type        = string
  default     = "8.0"
}

# ロール管理に使用するデータベース名
# postgresql_grant リソースが参照するデータベースを指定する
variable "database_name" {
  description = "Name of the PostgreSQL database for role and grant management"
  type        = string
  default     = "k1s0"
}

# マイグレーション専用ロールのパスワード（DDL権限を持つ高権限ロール）
# 本番環境では Vault や外部シークレット管理から注入すること
variable "migration_password" {
  description = "Password for the k1s0_migration role (DDL privileges). Must be injected from secret management in production."
  type        = string
  sensitive   = true
}

# サービス別ロールのパスワードマッピング
# キー: サービス名（auth, config, saga 等）、値: 各ロールのパスワード
# 本番環境では Vault や外部シークレット管理から注入すること
variable "service_passwords" {
  description = "Map of service name to password for each service-specific DB role (e.g. { auth = \"...\" }). Must be injected from secret management in production."
  type        = map(string)
  sensitive   = true
}
