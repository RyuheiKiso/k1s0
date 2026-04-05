# Vault Module - Authentication Configuration
# Configures Kubernetes Auth, AppRole, and LDAP auth backends.
#
# INFRA-001 整合性確認済み（2026-04-03）:
# 本ファイルの全 bound_service_account_names は以下と完全に一致していることを確認済み。
# SA 名の変更は不要。
#   1. infra/kubernetes/rbac/service-accounts.yaml の ServiceAccount.metadata.name
#   2. 各サービスの infra/helm/services/**/values.yaml の serviceAccount.name
# bound_service_account_names の命名規約混在（-rust / -sa / suffix なし）は意図的設計。
# 特に bff-proxy-sa は H-1 監査対応で確定済みであり変更不可。

# ============================================================
# Kubernetes Auth Backend
# ============================================================

resource "vault_auth_backend" "kubernetes" {
  type = "kubernetes"
}

resource "vault_kubernetes_auth_backend_config" "k8s" {
  backend            = vault_auth_backend.kubernetes.path
  # Kubernetes API サーバーのエンドポイントを変数から参照する
  kubernetes_host    = var.kubernetes_host
  # H-5 監査対応: CA 証明書パスのハードコードを解消。
  # var.kubernetes_ca_cert が空文字列の場合は Kubernetes Pod 内のサービスアカウントマウントパスから読み込む。
  # ローカル/CI 環境からの terraform apply 時は kubernetes_ca_cert 変数で PEM 内容を渡すこと。
  kubernetes_ca_cert = var.kubernetes_ca_cert != "" ? var.kubernetes_ca_cert : file("/var/run/secrets/kubernetes.io/serviceaccount/ca.crt")
}

# system Tier role - サービス別SA名で最小権限を適用
# H-1 監査対応: bff-proxy の Vault 認証を system ロールに統一する。
#   - service-accounts.yaml の SA 名は "bff-proxy-sa"（namespace: k1s0-system）
#   - values.yaml の vault.role は "system" に設定済み
#   → bound_service_account_names に "bff-proxy-sa" を追加し、service ロールから削除する
#
# L-14 監査対応（Phase 5 実装予定）: ADR-0045 で定義した「サービス個別 Vault ロール」への移行が必要。
# 現在は全 system tier サービス（27 SA）が単一 "system" ロールに集約されているため、
# 1サービスが侵害された場合に同 tier 内の全シークレットにアクセス可能なリスクがある。
# 個別ポリシーファイル（infra/vault/policies/{service}.hcl）は既に用意済みであり、
# Phase 5 で以下の対応を実施すること:
#   1. 各サービス用 vault_kubernetes_auth_backend_role リソースをサービス数分作成する
#      例: vault_kubernetes_auth_backend_role.auth_rust, .bff_proxy, .config_rust, ...
#   2. 各ロールの token_policies を専用 HCL ポリシー名に変更する
#      例: token_policies = ["auth-rust"] (auth.hcl を参照)
#   3. 現在の "system" ロールは全移行完了後に削除する
#   参照: docs/architecture/adr/0045-vault-per-service-roles.md
# H-04 監査対応: 旧モノリシック "system" ロールを削除済み（2026-03-30）。
# 全26サービスの Helm values.yaml で vault.role が個別ロール名に移行完了したため削除した。
# いずれかの SA が侵害された場合の爆発半径を最小化するため、全サービスを個別ロールで管理する。
# 参照: ADR-0045（docs/architecture/adr/0045-vault-per-service-roles.md）

# INFRA-HIGH-002 対応: business/service ティアも system ティア（ADR-0045）と同様に
# サービス個別 Vault ロールに分離する（ADR-0077 参照）。
# 共有ロールでは1サービス侵害で同ティア全シークレットが漏洩するリスクがあるため廃止する。

# AI ゲートウェイサービス（ai-gateway）個別 Vault ロール（MED-005 監査対応）
# HIGH-006 監査対応: 共通ポリシー "system" を削除し、ai-gateway 専用ポリシーのみを付与する
# ai-gateway の Helm values.yaml の vault.role が "ai-gateway" と一致させる
resource "vault_kubernetes_auth_backend_role" "ai_gateway" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "ai-gateway"
  bound_service_account_names      = ["ai-gateway"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["ai-gateway"]
}

# AI エージェントサービス（ai-agent）個別 Vault ロール（MED-005 監査対応）
# HIGH-006 監査対応: 共通ポリシー "system" を削除し、ai-agent 専用ポリシーのみを付与する
# ai-agent の Helm values.yaml の vault.role が "ai-agent" と一致させる
resource "vault_kubernetes_auth_backend_role" "ai_agent" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "ai-agent"
  bound_service_account_names      = ["ai-agent"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["ai-agent"]
}

# project-master-rust 個別 Vault ロール（business ティア）
# ADR-0077: 旧共有ロール "business" を廃止し、project-master 専用ロールに移行する
# HIGH-006 監査対応: 共通ポリシー "business" を削除し、project-master 専用ポリシーのみを付与する
resource "vault_kubernetes_auth_backend_role" "project_master_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "project-master-rust"
  bound_service_account_names      = ["project-master-rust"]
  bound_service_account_namespaces = ["k1s0-business"]
  # project-master 専用ポリシーのみを付与する（ADR-0045 / ADR-0077 準拠）
  token_policies                   = ["project-master"]
  token_ttl                        = 3600
  # M-18 監査対応: token_max_ttl を 24h(86400)から 4h(14400)に短縮してセッション乗っ取りリスクを低減する
  token_max_ttl                    = 14400
}

# task-rust 個別 Vault ロール（service ティア）
# ADR-0077: 旧共有ロール "service" を廃止し、task 専用ロールに移行する
# HIGH-006 監査対応: 共通ポリシー "service" を削除し、task 専用ポリシーのみを付与する
resource "vault_kubernetes_auth_backend_role" "task_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "task-rust"
  bound_service_account_names      = ["task-rust"]
  bound_service_account_namespaces = ["k1s0-service"]
  # task 専用ポリシーのみを付与する（ADR-0045 / ADR-0077 準拠）
  token_policies                   = ["task"]
  token_ttl                        = 3600
  # M-18 監査対応: token_max_ttl を 24h(86400)から 4h(14400)に短縮してセッション乗っ取りリスクを低減する
  token_max_ttl                    = 14400
}

# board-rust 個別 Vault ロール（service ティア）
# ADR-0077: 旧共有ロール "service" を廃止し、board 専用ロールに移行する
# HIGH-006 監査対応: 共通ポリシー "service" を削除し、board 専用ポリシーのみを付与する
resource "vault_kubernetes_auth_backend_role" "board_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "board-rust"
  bound_service_account_names      = ["board-rust"]
  bound_service_account_namespaces = ["k1s0-service"]
  # board 専用ポリシーのみを付与する（ADR-0045 / ADR-0077 準拠）
  token_policies                   = ["board"]
  token_ttl                        = 3600
  # M-18 監査対応: token_max_ttl を 24h(86400)から 4h(14400)に短縮してセッション乗っ取りリスクを低減する
  token_max_ttl                    = 14400
}

# activity-rust 個別 Vault ロール（service ティア）
# ADR-0077: 旧共有ロール "service" を廃止し、activity 専用ロールに移行する
# HIGH-006 監査対応: 共通ポリシー "service" を削除し、activity 専用ポリシーのみを付与する
resource "vault_kubernetes_auth_backend_role" "activity_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "activity-rust"
  bound_service_account_names      = ["activity-rust"]
  bound_service_account_namespaces = ["k1s0-service"]
  # activity 専用ポリシーのみを付与する（ADR-0045 / ADR-0077 準拠）
  token_policies                   = ["activity"]
  token_ttl                        = 3600
  # M-18 監査対応: token_max_ttl を 24h(86400)から 4h(14400)に短縮してセッション乗っ取りリスクを低減する
  token_max_ttl                    = 14400
}

# ============================================================
# サービス個別 Vault ロール（H-02 / L-14 監査対応）
# 最小権限の原則に従い、各サービスが自サービスのシークレットにのみアクセスできるよう
# 個別の Kubernetes auth ロールを作成する。
# - "system" ポリシー: 共通シークレット（Kafka, Redis 等）へのアクセス
# - "<service>" ポリシー: サービス固有シークレットへのアクセス
# 移行完了後、上部の単一 "system" ロールを削除する（ADR-0045 参照）。
# ============================================================

# 認証サービス（auth-rust）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、auth 専用ポリシーのみを付与する
# auth サービスは JWT 署名・Transit・auth DB のみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "auth_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "auth-rust"
  bound_service_account_names      = ["auth-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["auth"]
}

# BFF プロキシサービス（bff-proxy-sa）個別 Vault ロール
# HIGH-006 監査対応: 共通ポリシー "system" を削除し、bff-proxy 専用ポリシーのみを付与する
resource "vault_kubernetes_auth_backend_role" "bff_proxy" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "bff-proxy"
  bound_service_account_names      = ["bff-proxy-sa"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["bff-proxy"]
}

# 設定サービス（config-rust）個別 Vault ロール
# HIGH-006 監査対応: 共通ポリシー "system" を削除し、config-server 専用ポリシーのみを付与する
resource "vault_kubernetes_auth_backend_role" "config_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "config-rust"
  bound_service_account_names      = ["config-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["config-server"]
}

# DLQ マネージャーサービス（dlq-manager）個別 Vault ロール
# HIGH-006 監査対応: 共通ポリシー "system" を削除し、dlq-manager 専用ポリシーのみを付与する
resource "vault_kubernetes_auth_backend_role" "dlq_manager" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "dlq-manager"
  bound_service_account_names      = ["dlq-manager"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["dlq-manager"]
}

# イベントストアサービス（event-store-rust）個別 Vault ロール
# HIGH-006 監査対応: 共通ポリシー "system" を削除し、event-store 専用ポリシーのみを付与する
resource "vault_kubernetes_auth_backend_role" "event_store_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "event-store-rust"
  bound_service_account_names      = ["event-store-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["event-store"]
}

# フィーチャーフラグサービス（featureflag-rust）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、featureflag 専用ポリシーのみを付与する
# featureflag サービスは featureflag DB のみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "featureflag_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "featureflag-rust"
  bound_service_account_names      = ["featureflag-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["featureflag"]
}

# ファイル管理サービス（file-rust）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、file 専用ポリシーのみを付与する
# file サービスは file DB・S3 クレデンシャルのみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "file_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "file-rust"
  bound_service_account_names      = ["file-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["file"]
}

# GraphQL ゲートウェイサービス（graphql-gateway）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、graphql-gateway 専用ポリシーのみを付与する
# graphql-gateway はゲートウェイ設定・PKI のみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "graphql_gateway" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "graphql-gateway"
  bound_service_account_names      = ["graphql-gateway"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["graphql-gateway"]
}

# マスターメンテナンスサービス（master-maintenance）個別 Vault ロール
# HIGH-006 監査対応: 共通ポリシー "system" を削除し、master-maintenance 専用ポリシーのみを付与する
resource "vault_kubernetes_auth_backend_role" "master_maintenance" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "master-maintenance"
  bound_service_account_names      = ["master-maintenance"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["master-maintenance"]
}

# ナビゲーションサービス（navigation-rust）個別 Vault ロール
# HIGH-006 監査対応: 共通ポリシー "system" を削除し、navigation 専用ポリシーのみを付与する
resource "vault_kubernetes_auth_backend_role" "navigation_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "navigation-rust"
  bound_service_account_names      = ["navigation-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["navigation"]
}

# 通知サービス（notification-rust）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、notification 専用ポリシーのみを付与する
# notification サービスは notification DB・Kafka・外部通知クレデンシャルのみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "notification_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "notification-rust"
  bound_service_account_names      = ["notification-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["notification"]
}

# ポリシー管理サービス（policy-rust）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、policy 専用ポリシーのみを付与する
# policy サービスは policy DB のみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "policy_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "policy-rust"
  bound_service_account_names      = ["policy-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["policy"]
}

# クォータ管理サービス（quota-rust）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、quota 専用ポリシーのみを付与する
# quota サービスは quota DB・Redis のみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "quota_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "quota-rust"
  bound_service_account_names      = ["quota-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["quota"]
}

# レートリミットサービス（ratelimit-rust）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、ratelimit 専用ポリシーのみを付与する
# ratelimit サービスは ratelimit DB・Redis のみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "ratelimit_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "ratelimit-rust"
  bound_service_account_names      = ["ratelimit-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["ratelimit"]
}

# ルールエンジンサービス（rule-engine-rust）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、rule-engine 専用ポリシーのみを付与する
# rule-engine サービスは rule-engine DB・Kafka のみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "rule_engine_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "rule-engine-rust"
  bound_service_account_names      = ["rule-engine-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["rule-engine"]
}

# Saga オーケストレーションサービス（saga-rust）個別 Vault ロール
# HIGH-006 監査対応: 共通ポリシー "system" を削除し、saga-server 専用ポリシーのみを付与する
resource "vault_kubernetes_auth_backend_role" "saga_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "saga-rust"
  bound_service_account_names      = ["saga-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["saga-server"]
}

# スケジューラーサービス（scheduler-rust）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、scheduler 専用ポリシーのみを付与する
# scheduler サービスは scheduler DB・Kafka のみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "scheduler_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "scheduler-rust"
  bound_service_account_names      = ["scheduler-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["scheduler"]
}

# 検索サービス（search-rust）個別 Vault ロール
# HIGH-006 監査対応: 共通ポリシー "system" を削除し、search 専用ポリシーのみを付与する
resource "vault_kubernetes_auth_backend_role" "search_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "search-rust"
  bound_service_account_names      = ["search-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["search"]
}

# サービスカタログサービス（service-catalog）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、service-catalog 専用ポリシーのみを付与する
# service-catalog サービスは service-catalog DB のみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "service_catalog" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "service-catalog"
  bound_service_account_names      = ["service-catalog"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["service-catalog"]
}

# セッション管理サービス（session-rust）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、session 専用ポリシーのみを付与する
# session サービスは session DB・Redis のみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "session_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "session-rust"
  bound_service_account_names      = ["session-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["session"]
}

# テナント管理サービス（tenant-rust）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、tenant 専用ポリシーのみを付与する
# tenant サービスは tenant DB・Kafka のみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "tenant_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "tenant-rust"
  bound_service_account_names      = ["tenant-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["tenant"]
}

# Vault シークレット管理サービス（vault-rust）個別 Vault ロール
# HIGH-006 監査対応: 共通ポリシー "system" を削除し、vault-server 専用ポリシーのみを付与する
resource "vault_kubernetes_auth_backend_role" "vault_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "vault-rust"
  bound_service_account_names      = ["vault-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["vault-server"]
}

# ワークフローサービス（workflow-rust）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、workflow 専用ポリシーのみを付与する
# workflow サービスは workflow DB・Kafka のみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "workflow_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "workflow-rust"
  bound_service_account_names      = ["workflow-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["workflow"]
}

# イベント監視サービス（event-monitor-rust）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、event-monitor 専用ポリシーのみを付与する
# event-monitor サービスは event-monitor DB・Kafka のみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "event_monitor_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "event-monitor-rust"
  bound_service_account_names      = ["event-monitor-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["event-monitor"]
}

# アプリケーションレジストリサービス（app-registry）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、app-registry 専用ポリシーのみを付与する
# app-registry サービスは app-registry DB のみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "app_registry" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "app-registry"
  bound_service_account_names      = ["app-registry"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["app-registry"]
}

# API レジストリサービス（api-registry-rust）個別 Vault ロール
# H-010 監査対応: "system" 共通ポリシーを削除し、api-registry 専用ポリシーのみを付与する
# api-registry サービスは api-registry DB のみアクセス可能とする
resource "vault_kubernetes_auth_backend_role" "api_registry_rust" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "api-registry-rust"
  bound_service_account_names      = ["api-registry-rust"]
  bound_service_account_namespaces = [var.k8s_namespace]
  token_ttl                        = 3600
  token_max_ttl                    = 14400
  token_policies                   = ["api-registry"]
}

# ============================================================
# AppRole Auth Backend - for CI/CD pipelines
# ============================================================

resource "vault_auth_backend" "approle" {
  type = "approle"
}

resource "vault_approle_auth_backend_role" "cicd" {
  backend        = vault_auth_backend.approle.path
  role_name      = "cicd"
  token_policies = ["cicd"]
  # token_ttl を 7200（2時間）に延長する。
  # CI/CD パイプラインが長時間のビルド・デプロイ中にトークン期限切れで失敗する問題を防ぐ。
  # token_max_ttl（上限）は token_ttl と同じ 7200 に合わせて整合性を保つ。
  token_ttl      = 7200
  token_max_ttl  = 7200
}

# ============================================================
# LDAP Auth Backend - for human operator access
# ============================================================

resource "vault_auth_backend" "ldap" {
  type = "ldap"
}

resource "vault_ldap_auth_backend" "ldap" {
  path         = vault_auth_backend.ldap.path
  url          = var.ldap_url
  userdn       = var.ldap_user_dn
  groupdn      = var.ldap_group_dn
  binddn       = var.ldap_bind_dn
  bindpass     = var.ldap_bind_password
  insecure_tls = false
  starttls     = true
  userattr     = "sAMAccountName"
  groupattr    = "memberOf"
  # L-13 監査対応: LDAP TLS 接続の証明書検証に使用する CA 証明書を設定する。
  # 空文字列の場合は Vault のデフォルト CA バンドルを使用する。
  # 自己署名証明書や企業内 CA 環境では ldap_ca_cert 変数に PEM 内容を指定すること。
  certificate  = var.ldap_ca_cert != "" ? var.ldap_ca_cert : null
}
