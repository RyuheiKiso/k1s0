# Terraform 設計

オンプレミス環境における Terraform の構成とモジュール設計を定義する。

## 基本方針

- Terraform は Kubernetes リソースの宣言的管理に使用する
- 物理 / 仮想サーバーの構築・OS 設定は Ansible で行い、Terraform の管理対象外とする
- 環境（dev / staging / prod）ごとにワークスペースを分離する
- State はリモートバックエンド（Consul）で管理する

## 管理対象

| 管理対象                  | Provider            | 備考                           |
| ------------------------- | ------------------- | ------------------------------ |
| Kubernetes リソース       | hashicorp/kubernetes | Namespace, RBAC, StorageClass  |
| Helm リリース             | hashicorp/helm       | アプリケーションデプロイ       |
| Vault 設定                | hashicorp/vault      | ポリシー・シークレットエンジン |
| Harbor プロジェクト       | goharbor/harbor      | プロジェクト・ロボットアカウント |
| Keycloak Realm 設定      | mrparkers/keycloak   | Realm・クライアント・ロール      |

## ディレクトリ構成

```
infra/terraform/
├── environments/
│   ├── dev/
│   │   ├── main.tf              # モジュール呼び出し
│   │   ├── variables.tf         # 環境固有の変数定義
│   │   ├── terraform.tfvars     # 環境固有の値
│   │   ├── backend.tf           # リモート State 設定
│   │   └── outputs.tf
│   ├── staging/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── terraform.tfvars
│   │   ├── backend.tf
│   │   └── outputs.tf
│   └── prod/
│       ├── main.tf
│       ├── variables.tf
│       ├── terraform.tfvars
│       ├── backend.tf
│       └── outputs.tf
└── modules/
    ├── kubernetes-base/         # Namespace, RBAC, NetworkPolicy
    ├── kubernetes-storage/      # StorageClass, PV
    ├── observability/           # Prometheus, Grafana, Jaeger, Loki, OTel Collector
    ├── messaging/               # Strimzi Operator + Kafka CRD
    ├── database/                # PostgreSQL, MySQL（Helm 経由）
    ├── vault/                   # Vault 基本設定（KV v2・DB・PKI エンジン、Kubernetes Auth）
    ├── vault-database/          # Vault Database シークレットエンジン専用
    ├── vault-pki/               # Vault PKI / Internal CA 専用
    ├── consul-backup/           # Consul State バックアップ CronJob
    ├── harbor/                  # Harbor プロジェクト設定
    ├── keycloak/                # Keycloak Realm プロビジョニング
    └── service-mesh/            # Istio 設定
```

### 環境への統合方針

`vault-database`・`vault-pki`・`consul-backup` の 3 モジュールは、すべての環境（dev / staging / prod）の `environments/<env>/main.tf` から呼び出す。独立したワークスペースとして別管理するのではなく、既存モジュール（`vault`、`database` 等）と同一の apply コンテキストで管理することで、依存関係の明確化と運用の一貫性を確保する。

## State 管理

State バックエンドには Kubernetes クラスタ外の共有 Consul サービスを使用する。Consul は Ansible で構築・管理される独立したインフラであり、Terraform の管理対象外である（「Ansible との責務分担」セクションを参照）。

```hcl
# environments/dev/backend.tf
terraform {
  backend "consul" {
    address = "consul.internal.example.com:8500"
    scheme  = "https"
    path    = "terraform/k1s0/dev"
    lock    = true
  }
}
```

| 環境    | State パス                  |
| ------- | --------------------------- |
| dev     | `terraform/k1s0/dev`        |
| staging | `terraform/k1s0/staging`    |
| prod    | `terraform/k1s0/prod`       |

### Consul HA 構成

| 環境    | 構成                                    | 備考                                        |
| ------- | --------------------------------------- | ------------------------------------------- |
| prod    | 3 ノードクラスタ（Server モード）       | Raft コンセンサスプロトコルで冗長化          |
| staging | 1 ノード                                | 開発用途のため HA 不要                       |
| dev     | 1 ノード                                | ローカル開発用途のため HA 不要               |

### State バックアップ・リカバリ

- **バックアップ**: `consul snapshot save` を毎日 CronJob で実行し、Ceph オブジェクトストレージに 7 世代保持する
- **リカバリ**: `consul snapshot restore` で Terraform State を復元する
- **手順**: 障害発生時は直近のスナップショットから復元し、`terraform plan` で差分を確認してから運用を再開する

## モジュール詳細

### kubernetes-base

Namespace・RBAC・NetworkPolicy を管理する。

```hcl
# modules/kubernetes-base/main.tf

resource "kubernetes_namespace" "tier" {
  for_each = var.namespaces

  metadata {
    name = each.key
    labels = {
      tier        = each.value.tier
      managed-by  = "terraform"
    }
  }
}

resource "kubernetes_network_policy" "deny_cross_tier" {
  for_each = var.namespaces

  metadata {
    name      = "deny-cross-tier"
    namespace = each.key
  }

  spec {
    pod_selector {}
    policy_types = ["Ingress"]

    # 許可する Tier からのインバウンド（複数 Tier を指定可能）
    dynamic "ingress" {
      for_each = length(each.value.allowed_from_tiers) > 0 ? [1] : []
      content {
        dynamic "from" {
          for_each = each.value.allowed_from_tiers
          content {
            namespace_selector {
              match_labels = {
                tier = from.value
              }
            }
          }
        }
        # 同一 Tier 内の通信も許可
        from {
          namespace_selector {
            match_labels = {
              tier = each.value.tier
            }
          }
        }
      }
    }
  }
}
```

### kubernetes-storage

```hcl
# modules/kubernetes-storage/main.tf

resource "kubernetes_storage_class" "ceph_block" {
  metadata {
    name = "ceph-block"
    annotations = {
      "storageclass.kubernetes.io/is-default-class" = "true"
    }
  }

  storage_provisioner    = "rbd.csi.ceph.com"
  reclaim_policy         = var.reclaim_policy   # dev: Delete, prod: Retain
  allow_volume_expansion = true

  parameters = {
    clusterID = var.ceph_cluster_id
    pool      = var.ceph_pool
  }
}

resource "kubernetes_storage_class" "ceph_filesystem" {
  metadata {
    name = "ceph-filesystem"
  }

  storage_provisioner    = "cephfs.csi.ceph.com"
  reclaim_policy         = var.reclaim_policy
  allow_volume_expansion = true

  parameters = {
    clusterID = var.ceph_cluster_id
    fsName    = var.ceph_filesystem_name
  }
}

resource "kubernetes_storage_class" "ceph_block_fast" {
  metadata {
    name = "ceph-block-fast"
  }

  storage_provisioner    = "rbd.csi.ceph.com"
  reclaim_policy         = var.reclaim_policy
  allow_volume_expansion = true

  parameters = {
    clusterID = var.ceph_cluster_id
    pool      = var.ceph_pool_fast   # SSD-backed pool
  }
}
```

### observability

Helm Provider 経由で可観測性スタックをデプロイする。主要リソースは `prometheus`（kube-prometheus-stack）、`loki`（loki-stack）、`jaeger`、`otel_collector`（opentelemetry-collector）の 4 つの `helm_release` で構成される。`otel_collector` は Jaeger デプロイ後に `depends_on` で順序制御される。

```hcl
# modules/observability/main.tf

resource "helm_release" "prometheus" {
  name       = "prometheus"
  namespace  = "observability"
  repository = "https://prometheus-community.github.io/helm-charts"
  chart      = "kube-prometheus-stack"
  version    = var.prometheus_version

  values = [file("${path.module}/values/prometheus.yaml")]
}

resource "helm_release" "loki" {
  name       = "loki"
  namespace  = "observability"
  repository = "https://grafana.github.io/helm-charts"
  chart      = "loki-stack"
  version    = var.loki_version

  values = [file("${path.module}/values/loki.yaml")]
}

resource "helm_release" "jaeger" {
  name       = "jaeger"
  namespace  = "observability"
  repository = "https://jaegertracing.github.io/helm-charts"
  chart      = "jaeger"
  version    = var.jaeger_version

  values = [file("${path.module}/values/jaeger.yaml")]
}

resource "helm_release" "otel_collector" {
  name       = "otel-collector"
  namespace  = "observability"
  repository = "https://open-telemetry.github.io/opentelemetry-helm-charts"
  chart      = "opentelemetry-collector"
  version    = var.otel_collector_version

  values = [file("${path.module}/values/otel-collector.yaml")]

  depends_on = [helm_release.jaeger]
}
```

### messaging

Strimzi Operator を Helm Chart 経由でデプロイし、Kafka クラスタを Strimzi CRD（`Kafka` カスタムリソース）として管理する。Strimzi Operator がクラスタのライフサイクルを監視・制御する。

主要リソース:
- `helm_release.strimzi_operator`: Strimzi Kafka Operator の Helm デプロイ（`strimzi.io/charts/` リポジトリ）
- `kubernetes_manifest.kafka_cluster`: `kafka.strimzi.io/v1beta2` の `Kafka` CRD でクラスタ定義（`k1s0-kafka`）

`kubernetes_manifest.kafka_cluster` は `depends_on` で `helm_release.strimzi_operator` の後にデプロイされる。

```hcl
# modules/messaging/main.tf

resource "helm_release" "strimzi_operator" {
  name       = "strimzi-kafka-operator"
  namespace  = "messaging"
  repository = "https://strimzi.io/charts/"
  chart      = "strimzi-kafka-operator"
  version    = var.strimzi_operator_version

  create_namespace = true
}

resource "kubernetes_manifest" "kafka_cluster" {
  manifest = {
    apiVersion = "kafka.strimzi.io/v1beta2"
    kind       = "Kafka"
    metadata = {
      name      = "k1s0-kafka"
      namespace = "messaging"
    }
    spec = {
      kafka = {
        version  = "3.6.1"
        replicas = var.kafka_broker_replicas
        # ...（listeners, config, storage, resources）
      }
      zookeeper = {
        replicas = var.zookeeper_replicas
        # ...（storage, resources）
      }
      entityOperator = {
        topicOperator = {}
        userOperator  = {}
      }
    }
  }

  depends_on = [helm_release.strimzi_operator]
}
```

### service-mesh

Istio サービスメッシュと関連 Addon をデプロイする。

```
modules/service-mesh/
├── main.tf          # Istio Helm Chart デプロイ
├── variables.tf     # Istio・Flagger バージョン、メッシュ設定
├── outputs.tf       # IngressGateway の IP 等
├── kiali.tf         # Kiali ダッシュボードのデプロイ
└── flagger.tf       # Flagger カナリアデプロイコントローラーのデプロイ
```

- Istio は `istio/istio` Helm Chart を使用してインストールする
- IstioOperator ではなく Helm ベースのインストールを採用する（管理の簡素化のため）
- Kiali・Jaeger の Addon も同モジュールでデプロイする
- Flagger は `flagger/flagger` Helm Chart を使用し、Istio 連携のカナリアデプロイコントローラーとしてデプロイする（詳細は [サービスメッシュ設計.md](../service-mesh/サービスメッシュ設計.md) の「カナリアリリースの段階的ロールアウト」を参照）

```hcl
# modules/service-mesh/main.tf

resource "helm_release" "istio_base" {
  name       = "istio-base"
  namespace  = "service-mesh"
  repository = "https://istio-release.storage.googleapis.com/charts"
  chart      = "base"
  version    = var.istio_version

  create_namespace = true
}

resource "helm_release" "istiod" {
  name       = "istiod"
  namespace  = "service-mesh"
  repository = "https://istio-release.storage.googleapis.com/charts"
  chart      = "istiod"
  version    = var.istio_version

  values = [file("${path.module}/values/istiod.yaml")]

  depends_on = [helm_release.istio_base]
}

resource "helm_release" "istio_ingress" {
  name       = "istio-ingress"
  namespace  = "service-mesh"
  repository = "https://istio-release.storage.googleapis.com/charts"
  chart      = "gateway"
  version    = var.istio_version

  values = [file("${path.module}/values/gateway.yaml")]

  depends_on = [helm_release.istiod]
}
```

```hcl
# modules/service-mesh/kiali.tf

resource "helm_release" "kiali" {
  name       = "kiali"
  namespace  = "service-mesh"
  repository = "https://kiali.org/helm-charts"
  chart      = "kiali-server"
  version    = var.kiali_version

  values = [file("${path.module}/values/kiali.yaml")]

  depends_on = [helm_release.istiod]
}
```

```hcl
# modules/service-mesh/flagger.tf

resource "helm_release" "flagger" {
  name       = "flagger"
  namespace  = "service-mesh"
  repository = "https://flagger.app"
  chart      = "flagger"
  version    = var.flagger_version

  set {
    name  = "meshProvider"
    value = "istio"
  }

  set {
    name  = "metricsServer"
    value = "http://prometheus.observability.svc.cluster.local:9090"
  }

  values = [file("${path.module}/values/flagger.yaml")]

  depends_on = [helm_release.istiod]
}
```

### database

PostgreSQL・MySQL を Helm Chart 経由でデプロイする。

```
modules/database/
├── main.tf          # PostgreSQL/MySQL Helm Chart デプロイ
├── variables.tf     # DB バージョン、ストレージサイズ、レプリカ数
├── outputs.tf       # 接続文字列、ポート情報
└── backup.tf        # バックアップ CronJob 定義
```

- PostgreSQL: Bitnami Helm Chart を使用する（dev/staging: `postgresql` Chart、prod: `postgresql-ha` Chart で HA 構成。Kong 用 PostgreSQL HA の詳細は [APIゲートウェイ設計.md](../../architecture/api/APIゲートウェイ設計.md) を参照）
- MySQL: Bitnami `mysql` Helm Chart を使用する
- バックアップ: CronJob で `pg_dump` / `mysqldump` を実行し、Ceph オブジェクトストレージに保存する
- 環境別設定: `variables.tf` で prod / staging / dev の構成（レプリカ数、ストレージサイズ等）を切り替える

```hcl
# modules/database/main.tf

resource "helm_release" "postgresql" {
  count      = var.enable_postgresql ? 1 : 0
  name       = "postgresql"
  namespace  = var.database_namespace
  repository = "https://charts.bitnami.com/bitnami"
  chart      = "postgresql"
  version    = var.postgresql_chart_version

  values = [file("${path.module}/values/postgresql-${var.environment}.yaml")]
}

resource "helm_release" "mysql" {
  count      = var.enable_mysql ? 1 : 0
  name       = "mysql"
  namespace  = var.database_namespace
  repository = "https://charts.bitnami.com/bitnami"
  chart      = "mysql"
  version    = var.mysql_chart_version

  values = [file("${path.module}/values/mysql-${var.environment}.yaml")]
}
```

```hcl
# modules/database/backup.tf

resource "kubernetes_cron_job_v1" "postgresql_backup" {
  count = var.enable_postgresql ? 1 : 0

  metadata {
    name      = "postgresql-backup"
    namespace = var.database_namespace
  }

  spec {
    schedule = "0 2 * * *"   # 毎日 02:00 JST

    job_template {
      spec {
        template {
          spec {
            container {
              name    = "pg-backup"
              image   = "bitnami/postgresql:${var.postgresql_version}"
              command = ["/bin/sh", "-c"]
              args    = [
                "pg_dump -h postgresql -U $PGUSER -d $PGDATABASE | gzip > /backup/pg-$(date +%Y%m%d).sql.gz && s3cmd put /backup/pg-$(date +%Y%m%d).sql.gz s3://${var.backup_bucket}/postgresql/"
              ]

              env_from {
                secret_ref {
                  name = "postgresql-credentials"
                }
              }
            }

            restart_policy = "OnFailure"
          }
        }
      }
    }
  }
}

resource "kubernetes_cron_job_v1" "mysql_backup" {
  count = var.enable_mysql ? 1 : 0

  metadata {
    name      = "mysql-backup"
    namespace = var.database_namespace
  }

  spec {
    schedule = "0 2 * * *"   # 毎日 02:00 JST

    job_template {
      spec {
        template {
          spec {
            container {
              name    = "mysql-backup"
              image   = "bitnami/mysql:${var.mysql_version}"
              command = ["/bin/sh", "-c"]
              args    = [
                "mysqldump -h mysql -u $MYSQL_USER -p$MYSQL_PASSWORD --all-databases | gzip > /backup/mysql-$(date +%Y%m%d).sql.gz && s3cmd put /backup/mysql-$(date +%Y%m%d).sql.gz s3://${var.backup_bucket}/mysql/"
              ]

              env_from {
                secret_ref {
                  name = "mysql-credentials"
                }
              }
            }

            restart_policy = "OnFailure"
          }
        }
      }
    }
  }
}
```

### harbor

Harbor コンテナレジストリをデプロイし、プロジェクト・ロボットアカウントを管理する。

```
modules/harbor/
├── main.tf          # Harbor Helm Chart デプロイ
├── variables.tf     # Harbor ドメイン、ストレージ設定
├── outputs.tf       # Harbor URL、管理者情報
└── projects.tf      # Harbor プロジェクト・ロボットアカウント定義
```

- Harbor Helm Chart を使用してデプロイする
- プロジェクト自動作成: `k1s0-system`, `k1s0-business`, `k1s0-service`, `k1s0-infra` の 4 プロジェクト
- ロボットアカウント: CI/CD 用のプッシュ権限付きアカウントを自動作成する
- ストレージバックエンド: Ceph S3 互換ストレージを使用する

```hcl
# modules/harbor/main.tf

resource "helm_release" "harbor" {
  name       = "harbor"
  namespace  = "harbor"
  repository = "https://helm.goharbor.io"
  chart      = "harbor"
  version    = var.harbor_chart_version

  create_namespace = true

  values = [file("${path.module}/values/harbor.yaml")]

  set {
    name  = "externalURL"
    value = "https://${var.harbor_domain}"
  }

  set {
    name  = "persistence.imageChartStorage.type"
    value = "s3"
  }

  set {
    name  = "persistence.imageChartStorage.s3.bucket"
    value = var.harbor_s3_bucket
  }

  set {
    name  = "persistence.imageChartStorage.s3.regionendpoint"
    value = var.ceph_s3_endpoint
  }
}
```

```hcl
# modules/harbor/projects.tf

resource "harbor_project" "k1s0" {
  for_each = toset(["k1s0-system", "k1s0-business", "k1s0-service", "k1s0-infra"])

  name   = each.key
  public = false

  depends_on = [helm_release.harbor]
}

resource "harbor_robot_account" "ci_push" {
  for_each = harbor_project.k1s0

  name        = "ci-push-${each.key}"
  description = "CI/CD push account for ${each.key}"
  level       = "project"

  permissions {
    access {
      action   = "push"
      resource = "repository"
    }
    access {
      action   = "pull"
      resource = "repository"
    }
    kind      = "project"
    namespace = each.value.name
  }

  depends_on = [harbor_project.k1s0]
}
```

### keycloak

[mrparkers/keycloak](https://registry.terraform.io/providers/mrparkers/keycloak/latest) Terraform Provider を使用し、Keycloak の Realm・クライアント・ロールを宣言的に管理する。

```
modules/keycloak/
├── main.tf          # Keycloak Provider 設定・Realm 定義
├── variables.tf     # Keycloak URL、Realm 名、クライアント設定
├── outputs.tf       # Realm 名、クライアント ID
├── clients.tf       # OIDC クライアント定義
└── roles.tf         # Realm ロール・クライアントロール定義
```

- Realm 定義のベースは `infra/docker/keycloak/k1s0-realm.json` に対応する（ローカル開発環境では JSON インポート、staging/prod では Terraform で管理）
- OIDC クライアント（`react-spa`, `flutter-app`, `k1s0-bff`, `auth-server`）を Terraform で作成する
- Realm ロール（`sys_admin`, `sys_operator`, `sys_auditor`, `user` 等）を Terraform で定義する
- クライアントシークレットは Vault に格納し、Terraform の `vault_generic_secret` data source で参照する

```hcl
# modules/keycloak/main.tf

terraform {
  required_providers {
    keycloak = {
      source  = "mrparkers/keycloak"
      version = ">= 4.0.0"
    }
  }
}

provider "keycloak" {
  client_id = "admin-cli"
  url       = var.keycloak_url
  username  = var.keycloak_admin_user
  password  = var.keycloak_admin_password
}

resource "keycloak_realm" "k1s0" {
  realm   = var.realm_name
  enabled = true

  login_theme   = "keycloak"
  account_theme = "keycloak.v2"

  access_token_lifespan = "5m"

  internationalization {
    supported_locales = ["en", "ja"]
    default_locale    = "ja"
  }
}
```

```hcl
# modules/keycloak/clients.tf

resource "keycloak_openid_client" "react_spa" {
  realm_id              = keycloak_realm.k1s0.id
  client_id             = "react-spa"
  name                  = "React SPA"
  access_type           = "PUBLIC"
  standard_flow_enabled = true

  valid_redirect_uris = var.react_spa_redirect_uris
  web_origins         = var.react_spa_web_origins
}

resource "keycloak_openid_client" "bff" {
  realm_id              = keycloak_realm.k1s0.id
  client_id             = "k1s0-bff"
  name                  = "BFF Server"
  access_type           = "CONFIDENTIAL"
  standard_flow_enabled = true

  valid_redirect_uris = var.bff_redirect_uris
}
```

```hcl
# modules/keycloak/roles.tf

resource "keycloak_role" "realm_roles" {
  for_each = toset(["sys_admin", "sys_operator", "sys_auditor", "user"])

  realm_id = keycloak_realm.k1s0.id
  name     = each.key
}
```

### vault

Vault の基本設定を管理する。シークレットエンジンのマウント、監査ログ設定、Kubernetes 認証バックエンドを一括でプロビジョニングする。

主要リソース:
- `vault_mount.kv`: KV v2 シークレットエンジン（パス: `secret`）— API キー・設定値等の静的シークレット管理
- `vault_mount.database`: Database シークレットエンジン（パス: `database`）— 動的データベース認証情報生成
- `vault_mount.pki`: PKI シークレットエンジン（パス: `pki`）— 内部 TLS 証明書発行（max lease: 10 年）
- `vault_audit.file`: ファイル監査ログ（`/vault/logs/audit.log`、シークレット値はマスク）
- `vault_auth_backend.kubernetes`: Kubernetes 認証メソッド（Pod ベースのアクセス制御）
- `vault_kubernetes_auth_backend_role.system/business/service`: 各 Tier の Namespace にバインドされた認証ロール（TTL: 1 時間）

```hcl
# modules/vault/main.tf

resource "vault_mount" "kv" {
  path        = "secret"
  type        = "kv-v2"
  description = "KV v2 secret engine for static secrets"
}

resource "vault_mount" "database" {
  path        = "database"
  type        = "database"
  description = "Database secret engine for dynamic credential generation"
}

resource "vault_mount" "pki" {
  path                  = "pki"
  type                  = "pki"
  description           = "PKI secret engine for internal TLS certificates"
  max_lease_ttl_seconds = 315360000  # 10 years
}

resource "vault_audit" "file" {
  type = "file"
  options = {
    file_path = "/vault/logs/audit.log"
    log_raw   = "false"
  }
}

resource "vault_auth_backend" "kubernetes" {
  type = "kubernetes"
  path = "kubernetes"
}

resource "vault_kubernetes_auth_backend_role" "system" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "system"
  bound_service_account_names      = ["*"]
  bound_service_account_namespaces = ["k1s0-system"]
  token_ttl                        = 3600
  token_policies                   = ["system-read"]
}

resource "vault_kubernetes_auth_backend_role" "business" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "business"
  bound_service_account_names      = ["*"]
  bound_service_account_namespaces = ["k1s0-business"]
  token_ttl                        = 3600
  token_policies                   = ["business-read"]
}

resource "vault_kubernetes_auth_backend_role" "service" {
  backend                          = vault_auth_backend.kubernetes.path
  role_name                        = "service"
  bound_service_account_names      = ["*"]
  bound_service_account_namespaces = ["k1s0-service"]
  token_ttl                        = 3600
  token_policies                   = ["service-read"]
}
```

### vault-database

Vault Database シークレットエンジンの詳細設定を専用モジュールとして管理する。各サービスの PostgreSQL 動的認証情報ロールを定義する。

主要リソース:
- `vault_mount.database`: Database エンジンのマウント（パス: `database`）
- `vault_database_secret_backend_connection.postgres`: PostgreSQL 接続設定（`k1s0-postgres`）
- `vault_database_secret_backend_role.*_rw / *_ro`: 各サービス（`auth-server`, `config-server`, `saga-server`, `dlq-manager`）の読み書き・読み取り専用ロール

各ロールは `CREATE ROLE` 文でスキーマ別の権限（例: `GRANT ALL ON ALL TABLES IN SCHEMA auth`）を付与し、TTL による認証情報の自動失効を管理する。environments/main.tf から呼び出す。

```hcl
# modules/vault-database/main.tf（抜粋）

resource "vault_mount" "database" {
  path        = "database"
  type        = "database"
  description = "Database secret engine for dynamic credential generation"
}

resource "vault_database_secret_backend_connection" "postgres" {
  backend       = vault_mount.database.path
  name          = "k1s0-postgres"
  allowed_roles = ["auth-server-rw", "auth-server-ro", "config-server-rw", "config-server-ro",
                   "saga-server-rw", "saga-server-ro", "dlq-manager-rw", "dlq-manager-ro"]

  postgresql {
    connection_url = "postgresql://{{username}}:{{password}}@${var.postgres_host}:${var.postgres_port}/postgres?sslmode=${var.postgres_ssl_mode}"
  }
}

resource "vault_database_secret_backend_role" "auth_server_rw" {
  backend             = vault_mount.database.path
  name                = "auth-server-rw"
  db_name             = vault_database_secret_backend_connection.postgres.name
  creation_statements = [
    "CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}';",
    "GRANT ALL ON ALL TABLES IN SCHEMA auth TO \"{{name}}\";",
    "GRANT USAGE ON SCHEMA auth TO \"{{name}}\";"
  ]
  default_ttl = var.credential_ttl
  max_ttl     = var.credential_max_ttl
}
# ...（auth-server-ro, config-server-rw/ro, saga-server-rw/ro, dlq-manager-rw/ro も同様に定義）
```

### vault-pki

Vault PKI シークレットエンジンを専用モジュールとして管理する。Root CA・Intermediate CA の構成と、Tier 別の証明書発行ポリシー（ロール）を定義する。

主要リソース:
- `vault_mount.pki`: Root CA マウント（パス: `pki`、max lease: 10 年）
- `vault_pki_secret_backend_root_cert.root`: 自己署名 Root CA（`k1s0 Internal CA`、RSA 4096）
- `vault_mount.pki_int`: Intermediate CA マウント（パス: `pki_int`、max lease: 5 年）
- `vault_pki_secret_backend_intermediate_cert_request.intermediate`: 中間 CA CSR 生成
- `vault_pki_secret_backend_root_sign_intermediate.intermediate`: Root CA による中間 CA 署名
- `vault_pki_secret_backend_intermediate_set_signed.intermediate`: 署名済み証明書の設定
- `vault_pki_secret_backend_role.system/business/service`: Tier 別証明書発行ポリシー（各 `svc.cluster.local` ドメインのサブドメインを許可、RSA 2048）

environments/main.tf から呼び出す。

```hcl
# modules/vault-pki/main.tf（抜粋）

resource "vault_mount" "pki" {
  path                      = "pki"
  type                      = "pki"
  description               = "PKI secret engine for internal TLS certificates"
  default_lease_ttl_seconds = 86400      # 24 hours
  max_lease_ttl_seconds     = 315360000  # 10 years
}

resource "vault_pki_secret_backend_root_cert" "root" {
  backend     = vault_mount.pki.path
  type        = "internal"
  common_name = "k1s0 Internal CA"
  ttl         = "315360000"
  key_type    = "rsa"
  key_bits    = 4096
}

resource "vault_pki_secret_backend_role" "system" {
  backend          = vault_mount.pki_int.path
  name             = "system"
  allowed_domains  = ["k1s0-system.svc.cluster.local"]
  allow_subdomains = true
  max_ttl          = var.system_cert_max_ttl
  key_type         = "rsa"
  key_bits         = 2048
}
# ...（business, service ロールも同様に定義）
```

### consul-backup

Consul State のスナップショットを毎日取得し Ceph オブジェクトストレージに保存する CronJob を管理する専用モジュール。

主要リソース:
- `kubernetes_cron_job_v1.consul_backup`: `consul snapshot save` を実行する CronJob（スケジュールは `var.schedule` で設定）

動作:
1. `consul snapshot save` でスナップショットを PVC に保存
2. `s3cmd put` で `s3://${var.backup_bucket}/consul/` にアップロード
3. ローカルの古いスナップショットを削除（`var.retention_count` 世代保持、デフォルト 7）

`CONSUL_HTTP_TOKEN` は `var.consul_token_secret_name` が参照する Kubernetes Secret から注入する。environments/main.tf から呼び出す。

```hcl
# modules/consul-backup/main.tf（抜粋）

resource "kubernetes_cron_job_v1" "consul_backup" {
  metadata {
    name      = "consul-backup"
    namespace = var.namespace
    labels = {
      "app.kubernetes.io/name"      = "consul-backup"
      "app.kubernetes.io/component" = "backup"
      "app.kubernetes.io/part-of"   = "k1s0"
    }
  }

  spec {
    schedule                      = var.schedule
    successful_jobs_history_limit = var.retention_count
    failed_jobs_history_limit     = 3
    concurrency_policy            = "Forbid"
    # ...（job_template: consul snapshot save → s3cmd put）
  }
}
```

## 環境別の変数例

```hcl
# environments/dev/terraform.tfvars

namespaces = {
  "k1s0-system" = {
    tier               = "system"
    allowed_from_tiers = ["system", "business", "service"]   # 全 Tier からのアクセスを許可（認証・config 等の共通基盤）
  }
  "k1s0-business" = {
    tier               = "business"
    allowed_from_tiers = ["business", "service"]             # service および同一 Tier から許可
  }
  "k1s0-service" = {
    tier               = "service"
    allowed_from_tiers = ["service"]                         # 同一 Tier のみ。ingress Namespace からのアクセスは kubernetes設計.md の NetworkPolicy で別途許可している
  }
  "observability" = {
    tier               = "infra"
    allowed_from_tiers = ["system", "business", "service"]   # 全 Tier からトレース送信・メトリクス取得が必要
  }
  "messaging" = {
    tier               = "infra"
    allowed_from_tiers = ["system", "business", "service"]   # 全 Tier から Kafka クラスタへのアクセスを許可
  }
  "ingress" = {
    tier               = "infra"
    allowed_from_tiers = []                                  # 外部トラフィックのみ受け付け
  }
  "service-mesh" = {
    tier               = "infra"
    allowed_from_tiers = []                                  # Istio Control Plane（istiod が各 Namespace の Sidecar と通信）
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

ceph_cluster_id  = "ceph-dev"
ceph_pool        = "k8s-block-dev"
reclaim_policy   = "Delete"
```

```hcl
# environments/prod/terraform.tfvars

reclaim_policy   = "Retain"
ceph_pool        = "k8s-block-prod"
# 実環境の Ceph クラスタ ID に合わせて変更すること
ceph_cluster_id  = "prod-ceph-cluster-001"
```

## Ansible との責務分担

Terraform と Ansible はインフラ構築の異なるレイヤーを担当する。責務を明確に分離し、二重管理を防止する。

### Terraform の責務

- Kubernetes クラスタ構成（Namespace, RBAC, NetworkPolicy, StorageClass）
- Helm Chart デプロイ（アプリケーション、ミドルウェア）
- Kubernetes リソース管理（ConfigMap, Secret, CronJob 等）
- Ceph CSI 設定（StorageClass, PersistentVolume）
- Vault / Harbor の設定管理

### Ansible の責務

- 物理 / 仮想サーバーの OS 初期設定（パッケージ、ユーザー、SSH）
- Kubernetes ノードのセットアップ（kubeadm によるクラスタ構築）
- ネットワーク設定（Calico CNI、MetalLB ロードバランサー）
- Ceph クラスタ構築（OSD、MON、MGR のデプロイ）
- Consul サーバーのインストールと初期設定

### 実行順序

```
1. Ansible（インフラ基盤構築）
   └─ OS 初期設定 → Kubernetes クラスタ構築 → Ceph クラスタ構築 → Consul セットアップ

2. Terraform（Kubernetes 上のリソース管理）
   └─ Namespace / RBAC → StorageClass → Helm Chart デプロイ → アプリケーション設定
```

> **注記**: Ansible プレイブック設計は別途 `docs/ansible設計.md` で定義する。

## 運用ルール

- `terraform plan` の結果を PR に添付し、レビューを受けてから `terraform apply` を実行する
- prod 環境への apply は 2 名以上の承認を必須とする
- State のロックを確認してから操作を行う
- `terraform import` は既存リソースの取り込み時のみ使用し、手動変更は禁止する

## 関連ドキュメント

- [kubernetes設計](../kubernetes/kubernetes設計.md)
- [helm設計](../kubernetes/helm設計.md)
- [インフラ設計](../overview/インフラ設計.md)
- [サービスメッシュ設計](../service-mesh/サービスメッシュ設計.md)
- [可観測性設計](../../architecture/observability/可観測性設計.md)
- [認証認可設計](../../architecture/auth/認証認可設計.md)
- [APIゲートウェイ設計](../../architecture/api/APIゲートウェイ設計.md)
