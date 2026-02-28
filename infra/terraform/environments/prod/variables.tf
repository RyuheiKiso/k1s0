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

variable "resource_quotas" {
  description = "Per-namespace resource quota overrides"
  type = map(object({
    requests_cpu    = string
    requests_memory = string
    limits_cpu      = string
    limits_memory   = string
    pods            = string
    pvcs            = string
  }))
  default = {}
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

variable "otel_collector_version" {
  description = "OpenTelemetry Collector Helm chart version"
  type        = string
  default     = "0.90.0"
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

# --- Vault ---
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

variable "ldap_url" {
  description = "LDAP server URL"
  type        = string
  default     = "ldaps://ldap.example.com:636"
}

variable "ldap_user_dn" {
  description = "LDAP user DN"
  type        = string
  default     = "ou=users,dc=example,dc=com"
}

variable "ldap_group_dn" {
  description = "LDAP group DN"
  type        = string
  default     = "ou=groups,dc=example,dc=com"
}

variable "ldap_bind_dn" {
  description = "LDAP bind DN"
  type        = string
  default     = "cn=vault,ou=service-accounts,dc=example,dc=com"
}

variable "ldap_bind_password" {
  description = "LDAP bind password"
  type        = string
  sensitive   = true
  default     = ""
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

# --- Keycloak ---
variable "keycloak_url" {
  description = "Keycloak server URL"
  type        = string
  default     = "https://keycloak.k1s0-system.svc.cluster.local:8443"
}

# --- Vault Database ---
variable "postgres_host" {
  description = "PostgreSQL server hostname for Vault database engine"
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

variable "vault_db_credential_ttl" {
  description = "Default TTL for dynamic database credentials (seconds)"
  type        = number
  default     = 86400
}

variable "vault_db_credential_max_ttl" {
  description = "Maximum TTL for dynamic database credentials (seconds)"
  type        = number
  default     = 172800
}

# --- Vault PKI ---
variable "vault_pki_system_cert_max_ttl" {
  description = "Maximum TTL for system tier TLS certificates (seconds)"
  type        = string
  default     = "2592000"
}

variable "vault_pki_business_cert_max_ttl" {
  description = "Maximum TTL for business tier TLS certificates (seconds)"
  type        = string
  default     = "2592000"
}

variable "vault_pki_service_cert_max_ttl" {
  description = "Maximum TTL for service tier TLS certificates (seconds)"
  type        = string
  default     = "2592000"
}

# --- Consul Backup ---
variable "consul_backup_namespace" {
  description = "Kubernetes namespace for the Consul backup CronJob"
  type        = string
  default     = "k1s0-system"
}

variable "consul_backup_schedule" {
  description = "Cron schedule for Consul state snapshot"
  type        = string
  default     = "0 0 * * *"
}

variable "consul_version" {
  description = "Consul image version for backup job"
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

variable "consul_backup_pvc_name" {
  description = "PersistentVolumeClaim name for local Consul backup storage"
  type        = string
  default     = "consul-backup-pvc"
}

variable "consul_backup_retention_count" {
  description = "Number of Consul backup snapshots to retain"
  type        = number
  default     = 90
}
