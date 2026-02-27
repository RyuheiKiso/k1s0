# テンプレート仕様 — Terraform

## 概要

本ドキュメントは、k1s0 CLI の「ひな形生成」機能で生成される **Terraform 環境ファイル** のテンプレート仕様を定義する。オンプレミス環境における Kubernetes リソースの宣言的管理に必要なファイル群（main.tf, variables.tf, terraform.tfvars, backend.tf, outputs.tf）を、環境（dev / staging / prod）ごとに自動生成する。

Terraform の全体設計は [terraform設計](../../infrastructure/terraform/terraform設計.md) を参照。

## 生成対象

Terraform テンプレートは環境セットアップ時に生成される。サービスの `kind` や `language` には依存せず、インフラ環境の初期構築時に使用する。

| 生成対象             | 説明                                                         |
| -------------------- | ------------------------------------------------------------ |
| 環境ディレクトリ     | `environments/{environment}/` 配下にファイル一式を生成       |
| モジュール呼び出し   | `main.tf` で共有モジュールを参照                             |
| 変数定義             | `variables.tf` でサービス固有の変数を定義                    |
| 環境固有値           | `terraform.tfvars` で環境ごとの値を設定                      |
| State 設定           | `backend.tf` で Consul バックエンドを設定                    |
| 出力定義             | `outputs.tf` で他環境・モジュールへの公開値を定義            |

## 配置パス

生成される Terraform ファイルは `infra/terraform/environments/` 配下に環境別のパスで配置される。

| 環境    | 配置パス                                         |
| ------- | ------------------------------------------------ |
| dev     | `infra/terraform/environments/dev/`              |
| staging | `infra/terraform/environments/staging/`          |
| prod    | `infra/terraform/environments/prod/`             |

## テンプレートファイル一覧

テンプレートファイルは `CLI/templates/terraform/` 配下に配置する。

| テンプレートファイル       | 生成先                                                      | 説明                         |
| -------------------------- | ----------------------------------------------------------- | ---------------------------- |
| `main.tf.tera`             | `infra/terraform/environments/{environment}/main.tf`        | モジュール呼び出し           |
| `variables.tf.tera`        | `infra/terraform/environments/{environment}/variables.tf`   | 変数定義                     |
| `terraform.tfvars.tera`    | `infra/terraform/environments/{environment}/terraform.tfvars` | 環境固有値                 |
| `backend.tf.tera`          | `infra/terraform/environments/{environment}/backend.tf`     | リモート State 設定          |
| `outputs.tf.tera`          | `infra/terraform/environments/{environment}/outputs.tf`     | 出力定義                     |

### ディレクトリ構成

```
CLI/
└── templates/
    └── terraform/
        ├── main.tf.tera
        ├── variables.tf.tera
        ├── terraform.tfvars.tera
        ├── backend.tf.tera
        └── outputs.tf.tera
```

## 使用するテンプレート変数

Terraform テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) を参照。

| 変数名               | 型     | 用途                                               |
| -------------------- | ------ | -------------------------------------------------- |
| `environment`        | String | 環境識別子（`dev` / `staging` / `prod`）           |
| `service_name`       | String | サービス名（State パス、リソースタグ等）           |
| `tier`               | String | 配置階層（Namespace 名の導出等）                   |
| `enable_postgresql`  | bool   | PostgreSQL モジュールの有効化                      |
| `enable_mysql`       | bool   | MySQL モジュールの有効化                           |
| `enable_kafka`       | bool   | Kafka モジュールの有効化                           |
| `enable_observability` | bool | 可観測性スタックモジュールの有効化                 |
| `enable_service_mesh`  | bool | サービスメッシュモジュールの有効化                 |
| `enable_vault`       | bool   | Vault 設定モジュールの有効化                       |
| `enable_harbor`      | bool   | Harbor プロジェクト管理モジュールの有効化          |

---

## 各テンプレートの内容

### main.tf.tera

共有モジュールを呼び出し、環境に応じたリソースを構成する。条件変数（`enable_*`）により、必要なモジュールのみが有効化される。

```tera
# main.tf — {{ environment }} 環境
# Terraform モジュール呼び出し

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.25"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.12"
    }
{% if enable_vault %}
    vault = {
      source  = "hashicorp/vault"
      version = "~> 3.24"
    }
{% endif %}
{% if enable_harbor %}
    harbor = {
      source  = "goharbor/harbor"
      version = "~> 3.10"
    }
{% endif %}
  }
}

provider "kubernetes" {
  config_path = "~/.kube/config"
}

provider "helm" {
  kubernetes {
    config_path = "~/.kube/config"
  }
}

{% if enable_vault %}
provider "vault" {
  address = var.vault_address
}
{% endif %}

{% if enable_harbor %}
provider "harbor" {
  url      = var.harbor_url
  username = var.harbor_admin_username
  password = var.harbor_admin_password
}
{% endif %}

# -------------------------------------------
# Kubernetes 基盤モジュール
# -------------------------------------------

module "kubernetes_base" {
  source = "../../modules/kubernetes-base"

  namespaces = var.namespaces
}

module "kubernetes_storage" {
  source = "../../modules/kubernetes-storage"

  ceph_cluster_id     = var.ceph_cluster_id
  ceph_pool           = var.ceph_pool
  ceph_pool_fast      = var.ceph_pool_fast
  ceph_filesystem_name = var.ceph_filesystem_name
  reclaim_policy      = var.reclaim_policy
}

# -------------------------------------------
# 可観測性スタック
# -------------------------------------------

{% if enable_observability %}
module "observability" {
  source = "../../modules/observability"

  prometheus_version = var.prometheus_version
  loki_version       = var.loki_version
  jaeger_version     = var.jaeger_version

  depends_on = [module.kubernetes_base]
}
{% endif %}

# -------------------------------------------
# サービスメッシュ
# -------------------------------------------

{% if enable_service_mesh %}
module "service_mesh" {
  source = "../../modules/service-mesh"

  istio_version   = var.istio_version
  kiali_version   = var.kiali_version
  flagger_version = var.flagger_version

  depends_on = [module.kubernetes_base]
}
{% endif %}

# -------------------------------------------
# データベース
# -------------------------------------------

{% if enable_postgresql or enable_mysql %}
module "database" {
  source = "../../modules/database"

  environment             = var.environment
  database_namespace      = var.database_namespace
  enable_postgresql       = var.enable_postgresql
  enable_mysql            = var.enable_mysql
  postgresql_chart_version = var.postgresql_chart_version
  mysql_chart_version     = var.mysql_chart_version
  backup_bucket           = var.backup_bucket

  depends_on = [module.kubernetes_base, module.kubernetes_storage]
}
{% endif %}

# -------------------------------------------
# メッセージング
# -------------------------------------------

{% if enable_kafka %}
module "messaging" {
  source = "../../modules/messaging"

  kafka_version     = var.kafka_version
  kafka_namespace   = var.kafka_namespace

  depends_on = [module.kubernetes_base, module.kubernetes_storage]
}
{% endif %}

# -------------------------------------------
# Vault 設定
# -------------------------------------------

{% if enable_vault %}
module "vault" {
  source = "../../modules/vault"

  environment    = var.environment
  vault_policies = var.vault_policies
  secret_engines = var.secret_engines
}
{% endif %}

# -------------------------------------------
# Harbor プロジェクト管理
# -------------------------------------------

{% if enable_harbor %}
module "harbor" {
  source = "../../modules/harbor"

  harbor_domain        = var.harbor_domain
  harbor_chart_version = var.harbor_chart_version
  harbor_s3_bucket     = var.harbor_s3_bucket
  ceph_s3_endpoint     = var.ceph_s3_endpoint

  depends_on = [module.kubernetes_base, module.kubernetes_storage]
}
{% endif %}
```

### variables.tf.tera

環境共通の変数定義。モジュールで使用する入力変数を宣言する。

```tera
# variables.tf — {{ environment }} 環境
# 変数定義

# -------------------------------------------
# 共通
# -------------------------------------------

variable "environment" {
  description = "環境識別子（dev / staging / prod）"
  type        = string
}

variable "namespaces" {
  description = "Kubernetes Namespace 定義（Tier・アクセス制御付き）"
  type = map(object({
    tier               = string
    allowed_from_tiers = list(string)
  }))
}

# -------------------------------------------
# Kubernetes Storage
# -------------------------------------------

variable "ceph_cluster_id" {
  description = "Ceph クラスタ ID"
  type        = string
}

variable "ceph_pool" {
  description = "Ceph ブロックストレージプール名"
  type        = string
}

variable "ceph_pool_fast" {
  description = "Ceph 高速ブロックストレージプール名（SSD-backed）"
  type        = string
  default     = ""
}

variable "ceph_filesystem_name" {
  description = "CephFS ファイルシステム名"
  type        = string
  default     = ""
}

variable "reclaim_policy" {
  description = "StorageClass の Reclaim Policy（dev: Delete, prod: Retain）"
  type        = string
  default     = "Delete"
}

# -------------------------------------------
# 可観測性
# -------------------------------------------

{% if enable_observability %}
variable "prometheus_version" {
  description = "kube-prometheus-stack Helm Chart バージョン"
  type        = string
}

variable "loki_version" {
  description = "loki-stack Helm Chart バージョン"
  type        = string
}

variable "jaeger_version" {
  description = "Jaeger Helm Chart バージョン"
  type        = string
}
{% endif %}

# -------------------------------------------
# サービスメッシュ
# -------------------------------------------

{% if enable_service_mesh %}
variable "istio_version" {
  description = "Istio Helm Chart バージョン"
  type        = string
}

variable "kiali_version" {
  description = "Kiali Helm Chart バージョン"
  type        = string
}

variable "flagger_version" {
  description = "Flagger Helm Chart バージョン"
  type        = string
}
{% endif %}

# -------------------------------------------
# データベース
# -------------------------------------------

{% if enable_postgresql or enable_mysql %}
variable "database_namespace" {
  description = "データベース用 Namespace"
  type        = string
  default     = "database"
}

variable "backup_bucket" {
  description = "バックアップ用 Ceph S3 バケット名"
  type        = string
}
{% endif %}

{% if enable_postgresql %}
variable "enable_postgresql" {
  description = "PostgreSQL デプロイの有効化"
  type        = bool
  default     = false
}

variable "postgresql_chart_version" {
  description = "Bitnami PostgreSQL Helm Chart バージョン"
  type        = string
}
{% endif %}

{% if enable_mysql %}
variable "enable_mysql" {
  description = "MySQL デプロイの有効化"
  type        = bool
  default     = false
}

variable "mysql_chart_version" {
  description = "Bitnami MySQL Helm Chart バージョン"
  type        = string
}
{% endif %}

# -------------------------------------------
# メッセージング
# -------------------------------------------

{% if enable_kafka %}
variable "kafka_version" {
  description = "Kafka Helm Chart バージョン"
  type        = string
}

variable "kafka_namespace" {
  description = "Kafka 用 Namespace"
  type        = string
  default     = "messaging"
}
{% endif %}

# -------------------------------------------
# Vault
# -------------------------------------------

{% if enable_vault %}
variable "vault_address" {
  description = "Vault サーバーアドレス"
  type        = string
}

variable "vault_policies" {
  description = "Vault ポリシー定義"
  type = map(object({
    path         = string
    capabilities = list(string)
  }))
  default = {}
}

variable "secret_engines" {
  description = "Vault シークレットエンジン定義"
  type = map(object({
    type = string
    path = string
  }))
  default = {}
}
{% endif %}

# -------------------------------------------
# Harbor
# -------------------------------------------

{% if enable_harbor %}
variable "harbor_url" {
  description = "Harbor レジストリ URL"
  type        = string
}

variable "harbor_admin_username" {
  description = "Harbor 管理者ユーザー名"
  type        = string
  sensitive   = true
}

variable "harbor_admin_password" {
  description = "Harbor 管理者パスワード"
  type        = string
  sensitive   = true
}

variable "harbor_domain" {
  description = "Harbor ドメイン名"
  type        = string
}

variable "harbor_chart_version" {
  description = "Harbor Helm Chart バージョン"
  type        = string
}

variable "harbor_s3_bucket" {
  description = "Harbor ストレージ用 S3 バケット名"
  type        = string
}

variable "ceph_s3_endpoint" {
  description = "Ceph S3 互換エンドポイント"
  type        = string
}
{% endif %}
```

### terraform.tfvars.tera

環境固有の値を設定する。環境（`environment`）に応じてリソースのサイジングやポリシーが変化する。

```tera
# terraform.tfvars — {{ environment }} 環境
# 環境固有の設定値

environment = "{{ environment }}"

# -------------------------------------------
# Namespace 定義
# -------------------------------------------

namespaces = {
  "k1s0-system" = {
    tier               = "system"
    allowed_from_tiers = ["system", "business", "service"]
  }
  "k1s0-business" = {
    tier               = "business"
    allowed_from_tiers = ["business", "service"]
  }
  "k1s0-service" = {
    tier               = "service"
    allowed_from_tiers = ["service"]
  }
  "observability" = {
    tier               = "infra"
    allowed_from_tiers = ["system", "business", "service"]
  }
  "messaging" = {
    tier               = "infra"
    allowed_from_tiers = ["system", "business", "service"]
  }
  "ingress" = {
    tier               = "infra"
    allowed_from_tiers = []
  }
  "service-mesh" = {
    tier               = "infra"
    allowed_from_tiers = []
  }
  "cert-manager" = {
    tier               = "infra"
    allowed_from_tiers = []
  }
  "harbor" = {
    tier               = "infra"
    allowed_from_tiers = []
  }
}

# -------------------------------------------
# Kubernetes Storage
# -------------------------------------------

{% if environment == "prod" %}
ceph_cluster_id  = "prod-ceph-cluster-001"
ceph_pool        = "k8s-block-prod"
ceph_pool_fast   = "k8s-block-fast-prod"
reclaim_policy   = "Retain"
{% elif environment == "staging" %}
ceph_cluster_id  = "ceph-staging"
ceph_pool        = "k8s-block-staging"
ceph_pool_fast   = "k8s-block-fast-staging"
reclaim_policy   = "Delete"
{% else %}
ceph_cluster_id  = "ceph-dev"
ceph_pool        = "k8s-block-dev"
ceph_pool_fast   = "k8s-block-fast-dev"
reclaim_policy   = "Delete"
{% endif %}

# -------------------------------------------
# 可観測性
# -------------------------------------------

{% if enable_observability %}
prometheus_version = "56.6.0"
loki_version       = "2.10.0"
jaeger_version     = "0.73.0"
{% endif %}

# -------------------------------------------
# サービスメッシュ
# -------------------------------------------

{% if enable_service_mesh %}
istio_version   = "1.20.0"
kiali_version   = "1.77.0"
flagger_version = "1.36.0"
{% endif %}

# -------------------------------------------
# データベース
# -------------------------------------------

{% if enable_postgresql or enable_mysql %}
database_namespace = "database"
backup_bucket      = "k1s0-backup-{{ environment }}"
{% endif %}

{% if enable_postgresql %}
enable_postgresql        = true
postgresql_chart_version = "14.0.0"
{% endif %}

{% if enable_mysql %}
enable_mysql        = true
mysql_chart_version = "9.0.0"
{% endif %}

# -------------------------------------------
# メッセージング
# -------------------------------------------

{% if enable_kafka %}
kafka_version   = "26.0.0"
kafka_namespace = "messaging"
{% endif %}

# -------------------------------------------
# Vault
# -------------------------------------------

{% if enable_vault %}
{% if environment == "prod" %}
vault_address = "https://vault.internal.example.com:8200"
{% elif environment == "staging" %}
vault_address = "https://vault-staging.internal.example.com:8200"
{% else %}
vault_address = "https://vault-dev.internal.example.com:8200"
{% endif %}

vault_policies = {
  "k1s0-system" = {
    path         = "secret/data/k1s0/system/*"
    capabilities = ["read", "list"]
  }
  "k1s0-business" = {
    path         = "secret/data/k1s0/business/*"
    capabilities = ["read", "list"]
  }
  "k1s0-service" = {
    path         = "secret/data/k1s0/service/*"
    capabilities = ["read", "list"]
  }
}

secret_engines = {
  "k1s0-kv" = {
    type = "kv-v2"
    path = "secret/k1s0"
  }
}
{% endif %}

# -------------------------------------------
# Harbor
# -------------------------------------------

{% if enable_harbor %}
{% if environment == "prod" %}
harbor_url           = "https://harbor.internal.example.com"
harbor_domain        = "harbor.internal.example.com"
{% elif environment == "staging" %}
harbor_url           = "https://harbor-staging.internal.example.com"
harbor_domain        = "harbor-staging.internal.example.com"
{% else %}
harbor_url           = "https://harbor-dev.internal.example.com"
harbor_domain        = "harbor-dev.internal.example.com"
{% endif %}
harbor_chart_version = "1.14.0"
harbor_s3_bucket     = "harbor-{{ environment }}"
ceph_s3_endpoint     = "https://s3.internal.example.com"
{% endif %}
```

### backend.tf.tera

Consul バックエンドによるリモート State 管理を設定する。環境ごとに State パスが異なる。

```tera
# backend.tf — {{ environment }} 環境
# リモート State 設定（Consul バックエンド）

terraform {
  backend "consul" {
    address = "consul.internal.example.com:8500"
    scheme  = "https"
    path    = "terraform/k1s0/{{ environment }}"
    lock    = true
  }
}
```

### outputs.tf.tera

他のモジュールや環境から参照可能な出力値を定義する。

```tera
# outputs.tf — {{ environment }} 環境
# 出力定義

# -------------------------------------------
# Kubernetes 基盤
# -------------------------------------------

output "namespace_names" {
  description = "作成された Namespace 名の一覧"
  value       = module.kubernetes_base.namespace_names
}

output "storage_class_names" {
  description = "作成された StorageClass 名の一覧"
  value       = module.kubernetes_storage.storage_class_names
}

# -------------------------------------------
# 可観測性
# -------------------------------------------

{% if enable_observability %}
output "prometheus_endpoint" {
  description = "Prometheus サーバーの内部エンドポイント"
  value       = module.observability.prometheus_endpoint
}

output "grafana_endpoint" {
  description = "Grafana ダッシュボードの内部エンドポイント"
  value       = module.observability.grafana_endpoint
}

output "jaeger_endpoint" {
  description = "Jaeger UI の内部エンドポイント"
  value       = module.observability.jaeger_endpoint
}
{% endif %}

# -------------------------------------------
# サービスメッシュ
# -------------------------------------------

{% if enable_service_mesh %}
output "istio_ingress_ip" {
  description = "Istio IngressGateway の IP アドレス"
  value       = module.service_mesh.ingress_gateway_ip
}
{% endif %}

# -------------------------------------------
# データベース
# -------------------------------------------

{% if enable_postgresql %}
output "postgresql_endpoint" {
  description = "PostgreSQL サービスの内部エンドポイント"
  value       = module.database.postgresql_endpoint
}
{% endif %}

{% if enable_mysql %}
output "mysql_endpoint" {
  description = "MySQL サービスの内部エンドポイント"
  value       = module.database.mysql_endpoint
}
{% endif %}

# -------------------------------------------
# Harbor
# -------------------------------------------

{% if enable_harbor %}
output "harbor_url" {
  description = "Harbor レジストリの URL"
  value       = module.harbor.harbor_url
}

output "harbor_robot_accounts" {
  description = "CI/CD 用ロボットアカウント情報"
  value       = module.harbor.robot_accounts
  sensitive   = true
}
{% endif %}
```

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、`main.tf` 内のモジュール呼び出しと `variables.tf` の変数定義が変わる。

| 条件                          | 選択肢          | main.tf への影響                                        |
| ----------------------------- | --------------- | ------------------------------------------------------- |
| `enable_postgresql`           | `true`          | `module.database` に PostgreSQL 関連設定を追加          |
| `enable_mysql`                | `true`          | `module.database` に MySQL 関連設定を追加               |
| `enable_kafka`                | `true`          | `module.messaging` を有効化                             |
| `enable_observability`        | `true`          | `module.observability` を有効化                         |
| `enable_service_mesh`         | `true`          | `module.service_mesh` を有効化                          |
| `enable_vault`                | `true`          | `module.vault` と Vault Provider を有効化               |
| `enable_harbor`               | `true`          | `module.harbor` と Harbor Provider を有効化             |
| `environment`                 | `dev` / `staging` / `prod` | `terraform.tfvars` の値（Reclaim Policy、Vault アドレス等）を切り替え |

### 環境別の差分

| 設定項目          | dev          | staging      | prod              |
| ----------------- | ------------ | ------------ | ----------------- |
| `reclaim_policy`  | `Delete`     | `Delete`     | `Retain`          |
| Consul State パス | `terraform/k1s0/dev` | `terraform/k1s0/staging` | `terraform/k1s0/prod` |
| Vault アドレス    | `vault-dev`  | `vault-staging` | `vault`          |
| Harbor ドメイン   | `harbor-dev` | `harbor-staging` | `harbor`         |

### テンプレートファイルの生成条件

| ファイル               | 生成条件                                          |
| ---------------------- | ------------------------------------------------- |
| `main.tf`              | 常に生成                                          |
| `variables.tf`         | 常に生成                                          |
| `terraform.tfvars`     | 常に生成                                          |
| `backend.tf`           | 常に生成                                          |
| `outputs.tf`           | 常に生成                                          |

全ファイルが常に生成される。モジュールの有効・無効は `enable_*` 変数による条件分岐で制御する。

---

## 生成例

### 全モジュール有効（prod 環境）の場合

入力:
```json
{
  "environment": "prod",
  "service_name": "k1s0",
  "tier": "system",
  "enable_postgresql": true,
  "enable_mysql": false,
  "enable_kafka": true,
  "enable_observability": true,
  "enable_service_mesh": true,
  "enable_vault": true,
  "enable_harbor": true
}
```

生成されるファイル:
- `infra/terraform/environments/prod/main.tf` --- kubernetes-base, kubernetes-storage, observability, service-mesh, database（PostgreSQL）, messaging, vault, harbor の全モジュール呼び出し
- `infra/terraform/environments/prod/variables.tf` --- 全モジュールの変数定義
- `infra/terraform/environments/prod/terraform.tfvars` --- prod 固有値（`reclaim_policy: Retain`、本番 Vault/Harbor アドレス）
- `infra/terraform/environments/prod/backend.tf` --- Consul State パス `terraform/k1s0/prod`
- `infra/terraform/environments/prod/outputs.tf` --- 全モジュールの出力定義

生成後のディレクトリ構成:

```
infra/terraform/environments/prod/
├── main.tf
├── variables.tf
├── terraform.tfvars
├── backend.tf
└── outputs.tf
```

### 最小構成（dev 環境）の場合

入力:
```json
{
  "environment": "dev",
  "service_name": "k1s0",
  "tier": "system",
  "enable_postgresql": false,
  "enable_mysql": false,
  "enable_kafka": false,
  "enable_observability": false,
  "enable_service_mesh": false,
  "enable_vault": false,
  "enable_harbor": false
}
```

生成されるファイル:
- `infra/terraform/environments/dev/main.tf` --- kubernetes-base, kubernetes-storage のみ
- `infra/terraform/environments/dev/variables.tf` --- 共通変数のみ
- `infra/terraform/environments/dev/terraform.tfvars` --- dev 固有値（`reclaim_policy: Delete`）
- `infra/terraform/environments/dev/backend.tf` --- Consul State パス `terraform/k1s0/dev`
- `infra/terraform/environments/dev/outputs.tf` --- 基盤モジュールの出力のみ

### DB + 可観測性（staging 環境）の場合

入力:
```json
{
  "environment": "staging",
  "service_name": "k1s0",
  "tier": "system",
  "enable_postgresql": true,
  "enable_mysql": true,
  "enable_kafka": false,
  "enable_observability": true,
  "enable_service_mesh": false,
  "enable_vault": true,
  "enable_harbor": false
}
```

生成されるファイル:
- `infra/terraform/environments/staging/main.tf` --- kubernetes-base, kubernetes-storage, observability, database（PostgreSQL + MySQL）, vault
- `infra/terraform/environments/staging/variables.tf` --- 可観測性・DB・Vault の変数定義を含む
- `infra/terraform/environments/staging/terraform.tfvars` --- staging 固有値
- `infra/terraform/environments/staging/backend.tf` --- Consul State パス `terraform/k1s0/staging`
- `infra/terraform/environments/staging/outputs.tf` --- 有効化モジュールの出力定義

---

## 関連ドキュメント

- [terraform設計](../../infrastructure/terraform/terraform設計.md) --- Terraform のモジュール設計・管理対象・運用ルール
- [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) --- テンプレート変数・条件分岐・フィルタの仕様
- [テンプレート仕様-Helm](Helm.md) --- Helm Chart テンプレート仕様
- [テンプレート仕様-CICD](CICD.md) --- CI/CD ワークフローテンプレート仕様
- [kubernetes設計](../../infrastructure/kubernetes/kubernetes設計.md) --- Kubernetes クラスタ設計
- [可観測性設計](../../observability/overview/可観測性設計.md) --- Prometheus / Grafana / Jaeger / Loki の設計
- [サービスメッシュ設計](../../infrastructure/service-mesh/サービスメッシュ設計.md) --- Istio / Kiali / Flagger の設計
- [認証認可設計](../../auth/design/認証認可設計.md) --- Vault シークレット管理
- [インフラ設計](../../infrastructure/overview/インフラ設計.md) --- オンプレミスインフラの全体設計
