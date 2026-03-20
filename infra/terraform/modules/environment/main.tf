# 環境共通モジュール
# dev / staging / prod の全環境が共有するインフラリソース群をまとめて定義する。
# 環境固有の差分は呼び出し側（environments/*/main.tf）から変数として渡す。

# --- Kubernetes Base (Namespace, RBAC, NetworkPolicy, LimitRange, ResourceQuota) ---
module "kubernetes_base" {
  source          = "../kubernetes-base"
  namespaces      = var.namespaces
  resource_quotas = var.resource_quotas
}

# --- Kubernetes Storage (StorageClass, PV) ---
module "kubernetes_storage" {
  source               = "../kubernetes-storage"
  ceph_cluster_id      = var.ceph_cluster_id
  ceph_pool            = var.ceph_pool
  ceph_pool_fast       = var.ceph_pool_fast
  ceph_filesystem_name = var.ceph_filesystem_name
  reclaim_policy       = var.reclaim_policy
}

# --- Observability (Prometheus, Grafana, Jaeger, Loki) ---
module "observability" {
  source                 = "../observability"
  prometheus_version     = var.prometheus_version
  loki_version           = var.loki_version
  jaeger_version         = var.jaeger_version
  otel_collector_version = var.otel_collector_version

  depends_on = [module.kubernetes_base]
}

# --- Messaging (Kafka) ---
module "messaging" {
  source                           = "../messaging"
  strimzi_operator_version         = var.strimzi_operator_version
  kafka_broker_replicas            = var.kafka_broker_replicas
  zookeeper_replicas               = var.zookeeper_replicas
  kafka_default_replication_factor = var.kafka_default_replication_factor
  kafka_min_insync_replicas        = var.kafka_min_insync_replicas

  depends_on = [module.kubernetes_base]
}

# --- Database (PostgreSQL, MySQL) ---
module "database" {
  source                   = "../database"
  environment              = var.environment
  database_namespace       = "k1s0-system"
  enable_postgresql        = var.enable_postgresql
  enable_mysql             = var.enable_mysql
  postgresql_chart_version = var.postgresql_chart_version
  mysql_chart_version      = var.mysql_chart_version
  postgresql_version       = var.postgresql_version
  mysql_version            = var.mysql_version
  backup_bucket            = var.backup_bucket

  depends_on = [module.kubernetes_base, module.kubernetes_storage]
}

# --- Harbor (Container Registry) ---
module "harbor" {
  source               = "../harbor"
  harbor_chart_version = var.harbor_chart_version
  harbor_domain        = var.harbor_domain
  harbor_s3_bucket     = var.harbor_s3_bucket
  ceph_s3_endpoint     = var.ceph_s3_endpoint

  depends_on = [module.kubernetes_base, module.kubernetes_storage]
}

# --- Vault ---
module "vault" {
  source             = "../vault"
  vault_address      = var.vault_address
  kubernetes_host    = var.kubernetes_host
  ldap_url           = var.ldap_url
  ldap_user_dn       = var.ldap_user_dn
  ldap_group_dn      = var.ldap_group_dn
  ldap_bind_dn       = var.ldap_bind_dn
  ldap_bind_password = var.ldap_bind_password
}

# --- Vault Database (動的クレデンシャル生成) ---
module "vault_database" {
  source             = "../vault-database"
  postgres_host      = var.postgres_host
  postgres_port      = var.postgres_port
  postgres_ssl_mode  = var.postgres_ssl_mode
  credential_ttl     = var.vault_db_credential_ttl
  credential_max_ttl = var.vault_db_credential_max_ttl

  depends_on = [module.vault, module.database]
}

# --- Vault PKI (内部 CA / TLS 証明書発行) ---
module "vault_pki" {
  source                = "../vault-pki"
  system_cert_max_ttl   = var.vault_pki_system_cert_max_ttl
  business_cert_max_ttl = var.vault_pki_business_cert_max_ttl
  service_cert_max_ttl  = var.vault_pki_service_cert_max_ttl

  depends_on = [module.vault]
}

# --- Consul Backup (ステートスナップショット CronJob) ---
module "consul_backup" {
  source                   = "../consul-backup"
  namespace                = var.consul_backup_namespace
  schedule                 = var.consul_backup_schedule
  consul_version           = var.consul_version
  consul_http_addr         = var.consul_http_addr
  consul_token_secret_name = var.consul_token_secret_name
  backup_bucket            = var.backup_bucket
  backup_pvc_name          = var.consul_backup_pvc_name
  retention_count          = var.consul_backup_retention_count

  depends_on = [module.kubernetes_base]
}

# --- Keycloak (Identity Provider) ---
# react_spa_redirect_uris / react_spa_web_origins / bff_redirect_uris は
# 環境ごとに異なる URL を持つため、呼び出し側から変数として渡す
module "keycloak" {
  source       = "../keycloak"
  keycloak_url = var.keycloak_url
  realm_name   = "k1s0"

  react_spa_redirect_uris = var.react_spa_redirect_uris
  react_spa_web_origins   = var.react_spa_web_origins
  bff_redirect_uris       = var.bff_redirect_uris
}

# --- Service Mesh (Istio) ---
module "service_mesh" {
  source          = "../service-mesh"
  istio_version   = var.istio_version
  kiali_version   = var.kiali_version
  flagger_version = var.flagger_version

  depends_on = [module.kubernetes_base]
}
