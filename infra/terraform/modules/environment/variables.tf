# 環境共通モジュールの変数定義
# 全環境（dev / staging / prod）で共有するモジュール群のパラメータを受け取る

variable "environment" {
  description = "環境名 (dev, staging, prod)"
  type        = string
}

# --- Kubernetes プロバイダー設定 ---
variable "kubeconfig_path" {
  description = "kubeconfig ファイルへのパス"
  type        = string
  default     = "~/.kube/config"
}

variable "kubeconfig_context" {
  description = "使用する Kubernetes コンテキスト名"
  type        = string
}

# --- Namespace / RBAC ---
variable "namespaces" {
  description = "Namespace とティア設定のマップ"
  type = map(object({
    tier               = string
    allowed_from_tiers = list(string)
  }))
}

variable "resource_quotas" {
  description = "Namespace ごとのリソースクォータ設定"
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
  description = "Ceph クラスタ ID"
  type        = string
}

variable "ceph_pool" {
  description = "Ceph RBD ブロックプール名"
  type        = string
}

variable "ceph_pool_fast" {
  description = "Ceph RBD SSD バックドプール名"
  type        = string
}

variable "ceph_filesystem_name" {
  description = "CephFS ファイルシステム名"
  type        = string
}

variable "reclaim_policy" {
  description = "StorageClass の reclaim ポリシー (Delete または Retain)"
  type        = string
  default     = "Delete"
}

# --- Observability ---
variable "prometheus_version" {
  description = "kube-prometheus-stack Helm チャートバージョン"
  type        = string
  default     = "51.0.0"
}

variable "loki_version" {
  description = "loki-stack Helm チャートバージョン"
  type        = string
  default     = "2.9.0"
}

variable "jaeger_version" {
  description = "Jaeger Helm チャートバージョン"
  type        = string
  default     = "0.71.0"
}

variable "otel_collector_version" {
  description = "OpenTelemetry Collector Helm チャートバージョン"
  type        = string
  default     = "0.90.0"
}

# --- Messaging (Kafka) ---
variable "strimzi_operator_version" {
  description = "Strimzi Kafka Operator Helm チャートバージョン"
  type        = string
  default     = "0.38.0"
}

variable "kafka_broker_replicas" {
  description = "Kafka ブローカーのレプリカ数"
  type        = number
  default     = 1
}

# M-19 監査対応: zookeeper_replicas 変数を削除。KRaft モード移行済み（ADR-0016 参照）。

variable "kafka_default_replication_factor" {
  description = "Kafka トピックのデフォルトレプリケーションファクター"
  type        = number
  default     = 1
}

variable "kafka_min_insync_replicas" {
  description = "Kafka の最小同期レプリカ数"
  type        = number
  default     = 1
}

# --- Database ---
variable "enable_postgresql" {
  description = "PostgreSQL をデプロイするか否か"
  type        = bool
  default     = true
}

variable "enable_mysql" {
  description = "MySQL をデプロイするか否か"
  type        = bool
  default     = false
}

variable "postgresql_chart_version" {
  description = "Bitnami PostgreSQL Helm チャートバージョン"
  type        = string
  default     = "13.0.0"
}

variable "mysql_chart_version" {
  description = "Bitnami MySQL Helm チャートバージョン"
  type        = string
  default     = "9.0.0"
}

variable "postgresql_version" {
  description = "PostgreSQL イメージバージョン"
  type        = string
  default     = "16"
}

variable "mysql_version" {
  description = "MySQL イメージバージョン"
  type        = string
  default     = "8.0"
}

# --- Harbor ---
variable "harbor_chart_version" {
  description = "Harbor Helm チャートバージョン"
  type        = string
  default     = "1.13.0"
}

variable "harbor_domain" {
  description = "Harbor の外部ドメイン"
  type        = string
  # プレースホルダー: 本番環境では適切な値に置換すること
  default     = "harbor.internal.example.com"
}

variable "harbor_s3_bucket" {
  description = "Harbor イメージストレージ用 Ceph S3 バケット名"
  type        = string
}

variable "ceph_s3_endpoint" {
  description = "Ceph S3 互換エンドポイント URL"
  type        = string
  # プレースホルダー: 本番環境では適切な値に置換すること
  default     = "http://ceph-rgw.internal.example.com:8080"
}

# --- Vault ---
variable "vault_address" {
  description = "Vault サーバーアドレス"
  type        = string
  default     = "https://vault.k1s0-system.svc.cluster.local:8200"
}

variable "kubernetes_host" {
  description = "Kubernetes API サーバーアドレス"
  type        = string
  default     = "https://kubernetes.default.svc"
}

variable "ldap_url" {
  description = "LDAP サーバー URL"
  type        = string
  # プレースホルダー: 本番環境では適切な値に置換すること
  default     = "ldaps://ldap.example.com:636"
}

variable "ldap_user_dn" {
  description = "LDAP ユーザー DN"
  type        = string
  # プレースホルダー: 本番環境では適切な値に置換すること
  default     = "ou=users,dc=example,dc=com"
}

variable "ldap_group_dn" {
  description = "LDAP グループ DN"
  type        = string
  # プレースホルダー: 本番環境では適切な値に置換すること
  default     = "ou=groups,dc=example,dc=com"
}

variable "ldap_bind_dn" {
  description = "LDAP バインド DN"
  type        = string
  # プレースホルダー: 本番環境では適切な値に置換すること
  default     = "cn=vault,ou=service-accounts,dc=example,dc=com"
}

variable "ldap_bind_password" {
  description = "LDAP バインドパスワード"
  type        = string
  sensitive   = true
  default     = ""
}

# --- Service Mesh (Istio) ---
variable "istio_version" {
  description = "Istio Helm チャートバージョン"
  type        = string
  default     = "1.20.0"
}

variable "kiali_version" {
  description = "Kiali Helm チャートバージョン"
  type        = string
  default     = "1.76.0"
}

variable "flagger_version" {
  description = "Flagger Helm チャートバージョン"
  type        = string
  default     = "1.35.0"
}

# --- Keycloak ---
variable "keycloak_url" {
  description = "Keycloak サーバー URL"
  type        = string
  default     = "https://keycloak.k1s0-system.svc.cluster.local:8443"
}

# Keycloak の React SPA / BFF リダイレクト先 URI — 環境ごとに異なる
variable "react_spa_redirect_uris" {
  description = "Keycloak React SPA クライアントの許可リダイレクト URI リスト"
  type        = list(string)
}

variable "react_spa_web_origins" {
  description = "Keycloak React SPA クライアントの許可 Web オリジン リスト"
  type        = list(string)
}

variable "bff_redirect_uris" {
  description = "Keycloak BFF クライアントの許可リダイレクト URI リスト"
  type        = list(string)
}

# --- Vault Database ---
variable "postgres_host" {
  description = "Vault データベースエンジン用 PostgreSQL ホスト名"
  type        = string
  default     = "postgresql.k1s0-system.svc.cluster.local"
}

variable "postgres_port" {
  description = "PostgreSQL ポート番号"
  type        = number
  default     = 5432
}

variable "postgres_ssl_mode" {
  description = "PostgreSQL SSL モード (disable, require, verify-full)"
  type        = string
  default     = "verify-full"
}

variable "vault_db_credential_ttl" {
  description = "動的 DB クレデンシャルのデフォルト TTL（秒）"
  type        = number
  default     = 86400
}

variable "vault_db_credential_max_ttl" {
  description = "動的 DB クレデンシャルの最大 TTL（秒）"
  type        = number
  default     = 172800
}

# --- Vault PKI ---
variable "vault_pki_system_cert_max_ttl" {
  description = "システム層 TLS 証明書の最大 TTL（秒）"
  type        = string
  default     = "2592000"
}

variable "vault_pki_business_cert_max_ttl" {
  description = "ビジネス層 TLS 証明書の最大 TTL（秒）"
  type        = string
  default     = "2592000"
}

variable "vault_pki_service_cert_max_ttl" {
  description = "サービス層 TLS 証明書の最大 TTL（秒）"
  type        = string
  default     = "2592000"
}

# --- Consul Backup ---
variable "consul_backup_namespace" {
  description = "Consul バックアップ CronJob の Kubernetes Namespace"
  type        = string
  default     = "k1s0-system"
}

variable "consul_backup_schedule" {
  description = "Consul ステートスナップショットの Cron スケジュール"
  type        = string
  default     = "0 0 * * *"
}

variable "consul_version" {
  description = "バックアップジョブ用 Consul イメージバージョン"
  type        = string
  default     = "1.17"
}

variable "consul_http_addr" {
  description = "スナップショット API 用 Consul HTTP アドレス"
  type        = string
  default     = "http://consul-server:8500"
}

variable "consul_token_secret_name" {
  description = "Consul ACL トークンを格納する Kubernetes Secret 名"
  type        = string
  default     = "consul-acl-token"
}

variable "consul_backup_pvc_name" {
  description = "Consul バックアップ用ローカルストレージの PVC 名"
  type        = string
  default     = "consul-backup-pvc"
}

variable "consul_backup_retention_count" {
  description = "保持する Consul バックアップスナップショットの件数"
  type        = number
  default     = 7
}
