# k1s0 インフラの OpenTofu 設定ファイル（エントリーポイント）
# kind クラスタ / Kubernetes / Helm の 3 provider を宣言し
# 各リソースモジュールからインポートされるプロバイダ設定を提供する

# ============================================================
# terraform ブロック: 必要 provider バージョンを宣言する
# ============================================================
terraform {
  # OpenTofu バージョン制約（1.8 以上を要求する）
  required_version = ">= 1.8.0"

  required_providers {
    # kind provider: kind クラスタの宣言的プロビジョニングを担当する
    kind = {
      source  = "tehcyx/kind"
      version = "~> 0.7"
    }
    # Kubernetes provider: Namespace / ConfigMap 等の K8s リソースを管理する
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.35"
    }
    # Helm provider: Helm chart のデプロイを terraform から管理する
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.16"
    }
  }

  # state ファイルの保存先（ローカル開発は local backend を使用する）
  backend "local" {
    # state ファイルのパスを定義する
    path = "terraform.tfstate"
  }
}

# ============================================================
# provider 設定: 各 provider の接続設定を宣言する
# ============================================================
# kind provider の設定（特別な設定は不要。デフォルト設定を使用する）
provider "kind" {}

# Kubernetes provider の設定（kind クラスタの kubeconfig を参照する）
provider "kubernetes" {
  # kind クラスタの config_path を参照する
  config_path    = var.kubeconfig_path
  # 対象クラスタの context 名を指定する
  config_context = var.kube_context
}

# Helm provider の設定（Kubernetes provider と同じ認証情報を使用する）
provider "helm" {
  kubernetes {
    # Kubernetes provider と同一の kubeconfig を参照する
    config_path    = var.kubeconfig_path
    # 対象クラスタの context 名を指定する
    config_context = var.kube_context
  }
}

# ============================================================
# 変数定義: モジュール間で共有する入力変数を宣言する
# ============================================================
# kubeconfig ファイルのパス（デフォルトはホームディレクトリ）
variable "kubeconfig_path" {
  description = "kubeconfig ファイルのパス"
  type        = string
  default     = "~/.kube/config"
}

# 対象 Kubernetes クラスタの context 名
variable "kube_context" {
  description = "対象 Kubernetes クラスタの context 名"
  type        = string
  default     = "kind-k1s0-target"
}

# k1s0-target クラスタ名
variable "target_cluster_name" {
  description = "メインワークロードクラスタの名前"
  type        = string
  default     = "k1s0-target"
}

# k1s0-ops-edge クラスタ名
variable "ops_cluster_name" {
  description = "運用監視クラスタの名前"
  type        = string
  default     = "k1s0-ops-edge"
}

# k1s0 Helm chart バージョン
variable "helm_chart_version" {
  description = "k1s0 Helm chart のバージョン"
  type        = string
  default     = "0.1.0"
}

# 環境識別子（development / staging / production）
variable "environment" {
  description = "デプロイ環境の識別子"
  type        = string
  default     = "development"
  # 許可される値を validation で制約する
  validation {
    condition     = contains(["development", "staging", "production"], var.environment)
    error_message = "environment は development / staging / production のいずれかでなければならない。"
  }
}

# ============================================================
# ローカル値: 繰り返し使用する計算済み値を定義する
# ============================================================
locals {
  # k1s0 共通ラベルセットを定義する（全リソースに付与する）
  common_labels = {
    "k1s0.io/managed-by"  = "opentofu"
    "k1s0.io/axis"        = "infra"
    "k1s0.io/environment" = var.environment
  }

  # Helm values で使用する operator の有効/無効設定を定義する
  helm_values = {
    # 開発環境では ClickHouse を無効化する
    clickhouse_enabled = var.environment == "production" ? true : false
    # HA モードは production のみ有効化する
    openbao_ha_enabled = var.environment == "production" ? true : false
    # Harbor の Notary 検証は全環境で有効化する
    harbor_notary_enabled = true
  }
}

# ============================================================
# 出力値: apply 完了後に表示する情報を定義する
# ============================================================
# k1s0-target クラスタのエンドポイントを出力する
output "target_cluster_endpoint" {
  description = "k1s0-target クラスタの API エンドポイント"
  value       = module.clusters.target_cluster_endpoint
}

# k1s0-ops-edge クラスタのエンドポイントを出力する
output "ops_cluster_endpoint" {
  description = "k1s0-ops-edge クラスタの API エンドポイント"
  value       = module.clusters.ops_cluster_endpoint
}

# デプロイされた Helm release 一覧を出力する
output "deployed_helm_releases" {
  description = "デプロイされた Helm release の一覧"
  value       = module.helm_releases.release_names
}

# ============================================================
# モジュール参照: cluster.tf に定義したモジュールを呼び出す
# ============================================================
# クラスタプロビジョニングモジュールを参照する
module "clusters" {
  # cluster.tf に定義したモジュールのソースパス
  source = "./modules/clusters"
  # クラスタ名を渡す
  target_cluster_name = var.target_cluster_name
  ops_cluster_name    = var.ops_cluster_name
  # 共通ラベルを渡す
  common_labels       = local.common_labels
}

# Helm release デプロイモジュールを参照する
module "helm_releases" {
  # helm_releases に定義したモジュールのソースパス
  source = "./modules/helm_releases"
  # Helm chart バージョンを渡す
  chart_version   = var.helm_chart_version
  # Helm values を渡す
  helm_values     = local.helm_values
  # クラスタモジュールの完了に依存する（provisioner 後に実行する）
  depends_on      = [module.clusters]
}
