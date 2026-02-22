variable "namespace" {
  description = "Kubernetes namespace for the backup CronJob"
  type        = string
  default     = "k1s0-system"
}

variable "schedule" {
  description = "Cron schedule for the backup job"
  type        = string
  default     = "0 0 * * *"
}

variable "retention_count" {
  description = "Number of backup snapshots to retain"
  type        = number
  default     = 7
}

variable "consul_version" {
  description = "Consul image version"
  type        = string
  default     = "1.17"
}

variable "consul_http_addr" {
  description = "Consul HTTP address for snapshot API"
  type        = string
  default     = "http://consul-server:8500"
}

variable "consul_token_secret_name" {
  description = "Kubernetes Secret name containing the Consul ACL token"
  type        = string
  default     = "consul-acl-token"
}

variable "backup_bucket" {
  description = "S3 bucket name for storing Consul snapshots"
  type        = string
}

variable "backup_pvc_name" {
  description = "PersistentVolumeClaim name for local backup storage"
  type        = string
  default     = "consul-backup-pvc"
}
