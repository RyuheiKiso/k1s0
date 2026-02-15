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

variable "backup_bucket" {
  description = "Ceph S3 bucket name for database backups"
  type        = string
}
