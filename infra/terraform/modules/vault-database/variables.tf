# Vault Database Module - Variables

variable "postgres_host" {
  description = "PostgreSQL server hostname"
  type        = string
  default     = "postgres.k1s0-system.svc.cluster.local"
}

variable "postgres_port" {
  description = "PostgreSQL server port"
  type        = number
  default     = 5432
}

variable "postgres_ssl_mode" {
  description = "PostgreSQL SSL mode (disable, require, verify-full)"
  type        = string
  default     = "verify-full"
}

variable "credential_ttl" {
  description = "Default TTL for dynamic database credentials (seconds)"
  type        = number
  default     = 86400 # 24 hours
}

variable "credential_max_ttl" {
  description = "Maximum TTL for dynamic database credentials (seconds)"
  type        = number
  default     = 172800 # 48 hours
}
