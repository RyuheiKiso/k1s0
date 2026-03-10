variable "ceph_s3_endpoint" {
  description = "Ceph RGW S3-compatible endpoint URL"
  type        = string
}

variable "ceph_access_key" {
  description = "Ceph RGW access key"
  type        = string
  sensitive   = true
}

variable "ceph_secret_key" {
  description = "Ceph RGW secret key"
  type        = string
  sensitive   = true
}

variable "bucket_prefix" {
  description = "Prefix for bucket names"
  type        = string
  default     = "k1s0-apps"
}

variable "environments" {
  description = "List of environments to create buckets for"
  type        = list(string)
  default     = ["dev", "stg", "prod"]
}

variable "cors_allowed_origins" {
  description = "CORS allowed origins"
  type        = list(string)
  default     = ["*"]
}

variable "lifecycle_expiration_days" {
  description = "Days before old non-current versions expire"
  type        = number
  default     = 90
}

variable "tags" {
  description = "Tags to apply to resources"
  type        = map(string)
  default     = {}
}
