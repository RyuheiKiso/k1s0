variable "kubeconfig_path" {
  description = "Path to the kubeconfig file"
  type        = string
  default     = "~/.kube/config"
}

variable "kubeconfig_context" {
  description = "Kubernetes context to use"
  type        = string
  default     = "k1s0-prod"
}

variable "environment" {
  description = "Environment name (dev, staging, prod)"
  type        = string
  default     = "prod"
}

# --- Namespace / RBAC ---
variable "namespaces" {
  description = "Map of namespaces with tier and allowed_from_tiers"
  type = map(object({
    tier               = string
    allowed_from_tiers = list(string)
  }))
}

# --- Ceph Storage ---
variable "ceph_cluster_id" {
  description = "Ceph cluster ID"
  type        = string
}

variable "ceph_pool" {
  description = "Ceph RBD pool name"
  type        = string
}

variable "ceph_pool_fast" {
  description = "Ceph RBD SSD-backed pool name"
  type        = string
  default     = "k8s-block-fast-prod"
}

variable "ceph_filesystem_name" {
  description = "CephFS filesystem name"
  type        = string
  default     = "k8s-cephfs-prod"
}

variable "reclaim_policy" {
  description = "StorageClass reclaim policy (Delete or Retain)"
  type        = string
  default     = "Retain"
}

# --- Observability ---
variable "prometheus_version" {
  description = "kube-prometheus-stack Helm chart version"
  type        = string
  default     = "51.0.0"
}

variable "loki_version" {
  description = "loki-stack Helm chart version"
  type        = string
  default     = "2.9.0"
}

variable "jaeger_version" {
  description = "Jaeger Helm chart version"
  type        = string
  default     = "0.71.0"
}

# --- Messaging (Kafka) ---
variable "strimzi_operator_version" {
  description = "Strimzi Kafka Operator Helm chart version"
  type        = string
  default     = "0.38.0"
}

variable "kafka_broker_replicas" {
  description = "Number of Kafka broker replicas"
  type        = number
  default     = 3
}

variable "zookeeper_replicas" {
  description = "Number of ZooKeeper replicas"
  type        = number
  default     = 3
}

variable "kafka_default_replication_factor" {
  description = "Default replication factor for Kafka topics"
  type        = number
  default     = 3
}

variable "kafka_min_insync_replicas" {
  description = "Minimum in-sync replicas for Kafka"
  type        = number
  default     = 2
}

# --- Database ---
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
  default     = "13.0.0"
}

variable "mysql_chart_version" {
  description = "Bitnami MySQL Helm chart version"
  type        = string
  default     = "9.0.0"
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
  default     = "k1s0-db-backup-prod"
}

# --- Harbor ---
variable "harbor_chart_version" {
  description = "Harbor Helm chart version"
  type        = string
  default     = "1.13.0"
}

variable "harbor_domain" {
  description = "Harbor external domain"
  type        = string
  default     = "harbor.internal.example.com"
}

variable "harbor_s3_bucket" {
  description = "Ceph S3 bucket for Harbor image storage"
  type        = string
  default     = "harbor-images-prod"
}

variable "ceph_s3_endpoint" {
  description = "Ceph S3-compatible endpoint URL"
  type        = string
  default     = "http://ceph-rgw.internal.example.com:8080"
}

# --- Service Mesh (Istio) ---
variable "istio_version" {
  description = "Istio Helm chart version"
  type        = string
  default     = "1.20.0"
}

variable "kiali_version" {
  description = "Kiali Helm chart version"
  type        = string
  default     = "1.76.0"
}

variable "flagger_version" {
  description = "Flagger Helm chart version"
  type        = string
  default     = "1.35.0"
}
