# Vault Database Module - Variables

variable "postgres_host" {
  description = "PostgreSQL server hostname"
  type        = string
  default     = "postgresql.k1s0-system.svc.cluster.local"
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
  # M-6 監査対応: TTL を 24h から 1h に短縮する。
  # 動的クレデンシャルの目的（侵害時の影響最小化）から 24h は過剰であり、1h が業界標準のベストプラクティス。
  default = 3600 # 1 hour
}

variable "credential_max_ttl" {
  description = "Maximum TTL for dynamic database credentials (seconds)"
  type        = number
  # M-6 監査対応: max_ttl を 48h から 4h に短縮する。
  # クレデンシャルが漏洩した場合の暴露時間を最小化するため、短い max_ttl を設定する。
  default = 14400 # 4 hours
}
