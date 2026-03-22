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

# --- Harbor ---
variable "harbor_chart_version" {
  description = "Harbor Helm chart version"
  type        = string
  default     = "1.13.0"
}

variable "harbor_domain" {
  description = "Harbor external domain"
  type        = string
  validation {
    # 本番環境ではプレースホルダードメインの使用を禁止する
    condition     = !can(regex("example\\.com", var.harbor_domain))
    error_message = "本番環境では example.com プレースホルダーを使用できません。実際のドメインを指定してください。"
  }
}

variable "harbor_s3_bucket" {
  description = "Ceph S3 bucket for Harbor image storage"
  type        = string
  default     = "harbor-images-prod"
}

variable "ceph_s3_endpoint" {
  description = "Ceph S3-compatible endpoint URL"
  type        = string
  validation {
    # 本番環境ではプレースホルダードメインの使用を禁止する
    condition     = !can(regex("example\\.com", var.ceph_s3_endpoint))
    error_message = "本番環境では example.com プレースホルダーを使用できません。実際のドメインを指定してください。"
  }
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
  validation {
    # 本番環境ではプレースホルダードメインの使用を禁止する
    condition     = !can(regex("example\\.com", var.ldap_url))
    error_message = "本番環境では example.com プレースホルダーを使用できません。実際のドメインを指定してください。"
  }
}

variable "ldap_user_dn" {
  description = "LDAP user DN"
  type        = string
  validation {
    # 本番環境ではプレースホルダードメインの使用を禁止する
    condition     = !can(regex("example\\.com", var.ldap_user_dn))
    error_message = "本番環境では example.com プレースホルダーを使用できません。実際のドメインを指定してください。"
  }
}

variable "ldap_group_dn" {
  description = "LDAP group DN"
  type        = string
  validation {
    # 本番環境ではプレースホルダードメインの使用を禁止する
    condition     = !can(regex("example\\.com", var.ldap_group_dn))
    error_message = "本番環境では example.com プレースホルダーを使用できません。実際のドメインを指定してください。"
  }
}

variable "ldap_bind_dn" {
  description = "LDAP bind DN"
  type        = string
  validation {
    # 本番環境ではプレースホルダードメインの使用を禁止する
    condition     = !can(regex("example\\.com", var.ldap_bind_dn))
    error_message = "本番環境では example.com プレースホルダーを使用できません。実際のドメインを指定してください。"
  }
}

variable "ldap_bind_password" {
  description = "LDAP bind password"
  type        = string
  sensitive   = true
  validation {
    # 本番環境では空の LDAP バインドパスワードを禁止する
    condition     = length(var.ldap_bind_password) > 0
    error_message = "本番環境では ldap_bind_password に有効な値を設定してください。"
  }
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
  description = "Keycloak サーバー URL"
  type        = string
  default     = "https://keycloak.k1s0-system.svc.cluster.local:8443"
}

# prod 環境: 本番ドメイン（example.com プレースホルダー不可）のリダイレクト URI
variable "react_spa_redirect_uris" {
  description = "Keycloak React SPA クライアントの許可リダイレクト URI リスト"
  type        = list(string)
  validation {
    # 本番環境ではプレースホルダードメインの使用を禁止する
    condition     = !anytrue([for uri in var.react_spa_redirect_uris : can(regex("example\\.com", uri))])
    error_message = "本番環境では example.com プレースホルダーを使用できません。実際のドメインを指定してください。"
  }
}

variable "react_spa_web_origins" {
  description = "Keycloak React SPA クライアントの許可 Web オリジン リスト"
  type        = list(string)
  validation {
    # 本番環境ではプレースホルダードメインの使用を禁止する
    condition     = !anytrue([for origin in var.react_spa_web_origins : can(regex("example\\.com", origin))])
    error_message = "本番環境では example.com プレースホルダーを使用できません。実際のドメインを指定してください。"
  }
}

variable "bff_redirect_uris" {
  description = "Keycloak BFF クライアントの許可リダイレクト URI リスト"
  type        = list(string)
  validation {
    # 本番環境ではプレースホルダードメインの使用を禁止する
    condition     = !anytrue([for uri in var.bff_redirect_uris : can(regex("example\\.com", uri))])
    error_message = "本番環境では example.com プレースホルダーを使用できません。実際のドメインを指定してください。"
  }
}

# --- Vault Database ---
variable "postgres_host" {
  description = "PostgreSQL server hostname for Vault database engine"
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
