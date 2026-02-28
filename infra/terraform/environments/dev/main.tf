terraform {
  required_version = ">= 1.5.0"

  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.23"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.11"
    }
    vault = {
      source  = "hashicorp/vault"
      version = "~> 3.20"
    }
    harbor = {
      source  = "goharbor/harbor"
      version = "~> 3.10"
    }
  }
}

provider "kubernetes" {
  config_path    = var.kubeconfig_path
  config_context = var.kubeconfig_context
}

provider "helm" {
  kubernetes {
    config_path    = var.kubeconfig_path
    config_context = var.kubeconfig_context
  }
}

# --- Kubernetes Base (Namespace, RBAC, NetworkPolicy, LimitRange, ResourceQuota) ---
module "kubernetes_base" {
  source          = "../../modules/kubernetes-base"
  namespaces      = var.namespaces
  resource_quotas = var.resource_quotas
}

# --- Kubernetes Storage (StorageClass, PV) ---
module "kubernetes_storage" {
  source              = "../../modules/kubernetes-storage"
  ceph_cluster_id     = var.ceph_cluster_id
  ceph_pool           = var.ceph_pool
  ceph_pool_fast      = var.ceph_pool_fast
  ceph_filesystem_name = var.ceph_filesystem_name
  reclaim_policy      = var.reclaim_policy
}

# --- Observability (Prometheus, Grafana, Jaeger, Loki) ---
module "observability" {
  source                 = "../../modules/observability"
  prometheus_version     = var.prometheus_version
  loki_version           = var.loki_version
  jaeger_version         = var.jaeger_version
  otel_collector_version = var.otel_collector_version

  depends_on = [module.kubernetes_base]
}

# --- Messaging (Kafka) ---
module "messaging" {
  source                      = "../../modules/messaging"
  strimzi_operator_version    = var.strimzi_operator_version
  kafka_broker_replicas       = var.kafka_broker_replicas
  zookeeper_replicas          = var.zookeeper_replicas
  kafka_default_replication_factor = var.kafka_default_replication_factor
  kafka_min_insync_replicas   = var.kafka_min_insync_replicas

  depends_on = [module.kubernetes_base]
}

# --- Database (PostgreSQL, MySQL) ---
module "database" {
  source                   = "../../modules/database"
  environment              = var.environment
  database_namespace       = "k1s0-system"
  enable_postgresql         = var.enable_postgresql
  enable_mysql              = var.enable_mysql
  postgresql_chart_version = var.postgresql_chart_version
  mysql_chart_version      = var.mysql_chart_version
  postgresql_version       = var.postgresql_version
  mysql_version            = var.mysql_version
  backup_bucket            = var.backup_bucket

  depends_on = [module.kubernetes_base, module.kubernetes_storage]
}

# --- Harbor (Container Registry) ---
module "harbor" {
  source               = "../../modules/harbor"
  harbor_chart_version = var.harbor_chart_version
  harbor_domain        = var.harbor_domain
  harbor_s3_bucket     = var.harbor_s3_bucket
  ceph_s3_endpoint     = var.ceph_s3_endpoint

  depends_on = [module.kubernetes_base, module.kubernetes_storage]
}

# --- Vault ---
module "vault" {
  source             = "../../modules/vault"
  vault_address      = var.vault_address
  kubernetes_host    = var.kubernetes_host
  ldap_url           = var.ldap_url
  ldap_user_dn       = var.ldap_user_dn
  ldap_group_dn      = var.ldap_group_dn
  ldap_bind_dn       = var.ldap_bind_dn
  ldap_bind_password = var.ldap_bind_password
}

# --- Vault Database (Dynamic Credential Generation) ---
module "vault_database" {
  source               = "../../modules/vault-database"
  postgres_host        = var.postgres_host
  postgres_port        = var.postgres_port
  postgres_ssl_mode    = var.postgres_ssl_mode
  credential_ttl       = var.vault_db_credential_ttl
  credential_max_ttl   = var.vault_db_credential_max_ttl

  depends_on = [module.vault, module.database]
}

# --- Vault PKI (Internal CA / TLS Certificate Issuance) ---
module "vault_pki" {
  source                = "../../modules/vault-pki"
  system_cert_max_ttl   = var.vault_pki_system_cert_max_ttl
  business_cert_max_ttl = var.vault_pki_business_cert_max_ttl
  service_cert_max_ttl  = var.vault_pki_service_cert_max_ttl

  depends_on = [module.vault]
}

# --- Consul Backup (State Snapshot CronJob) ---
module "consul_backup" {
  source                    = "../../modules/consul-backup"
  namespace                 = var.consul_backup_namespace
  schedule                  = var.consul_backup_schedule
  consul_version            = var.consul_version
  consul_http_addr          = var.consul_http_addr
  consul_token_secret_name  = var.consul_token_secret_name
  backup_bucket             = var.backup_bucket
  backup_pvc_name           = var.consul_backup_pvc_name
  retention_count           = var.consul_backup_retention_count

  depends_on = [module.kubernetes_base]
}

# --- Keycloak (Identity Provider) ---
module "keycloak" {
  source       = "../../modules/keycloak"
  keycloak_url = var.keycloak_url
  realm_name   = "k1s0"

  react_spa_redirect_uris = ["http://localhost:3000/*"]
  react_spa_web_origins   = ["http://localhost:3000"]
  bff_redirect_uris       = ["http://localhost:8080/callback"]
}

# --- Service Mesh (Istio) ---
module "service_mesh" {
  source          = "../../modules/service-mesh"
  istio_version   = var.istio_version
  kiali_version   = var.kiali_version
  flagger_version = var.flagger_version

  depends_on = [module.kubernetes_base]
}
