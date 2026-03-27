# prod 環境エントリポイント
# プロバイダー設定と Terraform バージョン要件のみを定義し、
# 実リソースはすべて modules/environment に委譲する。

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
    # Vault プロバイダー: modules/vault と統一（~> 4.0）
    vault = {
      source  = "hashicorp/vault"
      version = "~> 4.0"
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

# --- 環境共通リソース群 ---
# prod 固有の差分（Keycloak URL など）を変数として渡す
module "environment" {
  source = "../../modules/environment"

  environment        = var.environment
  kubeconfig_path    = var.kubeconfig_path
  kubeconfig_context = var.kubeconfig_context

  namespaces      = var.namespaces
  resource_quotas = var.resource_quotas

  ceph_cluster_id      = var.ceph_cluster_id
  ceph_pool            = var.ceph_pool
  ceph_pool_fast       = var.ceph_pool_fast
  ceph_filesystem_name = var.ceph_filesystem_name
  reclaim_policy       = var.reclaim_policy

  prometheus_version     = var.prometheus_version
  loki_version           = var.loki_version
  jaeger_version         = var.jaeger_version
  otel_collector_version = var.otel_collector_version

  strimzi_operator_version         = var.strimzi_operator_version
  kafka_broker_replicas            = var.kafka_broker_replicas
  # M-19 監査対応: ZooKeeper 変数を削除。KRaft モード移行済み（ADR-0016 参照）。
  kafka_default_replication_factor = var.kafka_default_replication_factor
  kafka_min_insync_replicas        = var.kafka_min_insync_replicas

  enable_postgresql        = var.enable_postgresql
  enable_mysql             = var.enable_mysql
  postgresql_chart_version = var.postgresql_chart_version
  mysql_chart_version      = var.mysql_chart_version
  postgresql_version       = var.postgresql_version
  mysql_version            = var.mysql_version

  harbor_chart_version = var.harbor_chart_version
  harbor_domain        = var.harbor_domain
  harbor_s3_bucket     = var.harbor_s3_bucket
  ceph_s3_endpoint     = var.ceph_s3_endpoint

  vault_address      = var.vault_address
  kubernetes_host    = var.kubernetes_host
  ldap_url           = var.ldap_url
  ldap_user_dn       = var.ldap_user_dn
  ldap_group_dn      = var.ldap_group_dn
  ldap_bind_dn       = var.ldap_bind_dn
  ldap_bind_password = var.ldap_bind_password

  postgres_host               = var.postgres_host
  postgres_port               = var.postgres_port
  postgres_ssl_mode           = var.postgres_ssl_mode
  vault_db_credential_ttl     = var.vault_db_credential_ttl
  vault_db_credential_max_ttl = var.vault_db_credential_max_ttl

  vault_pki_system_cert_max_ttl   = var.vault_pki_system_cert_max_ttl
  vault_pki_business_cert_max_ttl = var.vault_pki_business_cert_max_ttl
  vault_pki_service_cert_max_ttl  = var.vault_pki_service_cert_max_ttl

  consul_backup_namespace       = var.consul_backup_namespace
  consul_backup_schedule        = var.consul_backup_schedule
  consul_version                = var.consul_version
  consul_http_addr              = var.consul_http_addr
  consul_token_secret_name      = var.consul_token_secret_name
  consul_backup_pvc_name        = var.consul_backup_pvc_name
  consul_backup_retention_count = var.consul_backup_retention_count

  keycloak_url = var.keycloak_url

  # prod 環境: 本番ドメイン（example.com）ベースの URI を使用
  react_spa_redirect_uris = var.react_spa_redirect_uris
  react_spa_web_origins   = var.react_spa_web_origins
  bff_redirect_uris       = var.bff_redirect_uris

  istio_version   = var.istio_version
  kiali_version   = var.kiali_version
  flagger_version = var.flagger_version
}
