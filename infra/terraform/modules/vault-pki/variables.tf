# Vault PKI Module - Variables

variable "system_cert_max_ttl" {
  description = "Maximum TTL for system tier certificates (seconds)"
  type        = string
  default     = "2592000" # 30 days
}

variable "business_cert_max_ttl" {
  description = "Maximum TTL for business tier certificates (seconds)"
  type        = string
  default     = "2592000" # 30 days
}

variable "service_cert_max_ttl" {
  description = "Maximum TTL for service tier certificates (seconds)"
  type        = string
  default     = "2592000" # 30 days
}
