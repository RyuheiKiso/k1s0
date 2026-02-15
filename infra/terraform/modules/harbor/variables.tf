variable "harbor_chart_version" {
  description = "Harbor Helm chart version"
  type        = string
}

variable "harbor_domain" {
  description = "Harbor external domain"
  type        = string
}

variable "harbor_s3_bucket" {
  description = "Ceph S3 bucket for Harbor image storage"
  type        = string
}

variable "ceph_s3_endpoint" {
  description = "Ceph S3-compatible endpoint URL"
  type        = string
}
