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
    ├── observability/           # Prometheus, Grafana, Jaeger, Loki
    ├── messaging/               # Kafka クラスタ
    ├── database/                # PostgreSQL, MySQL（Helm 経由）
    ├── vault/                   # Vault 設定
    ├── harbor/                  # Harbor プロジェクト設定
    └── service-mesh/            # Istio 設定
```

## State 管理

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

    ingress {
      from {
        namespace_selector {
          match_labels = {
            tier = each.value.allowed_from_tier
          }
        }
      }
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
```

### observability

Helm Provider 経由で可観測性スタックをデプロイする。

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
```

## 環境別の変数例

```hcl
# environments/dev/terraform.tfvars

namespaces = {
  "k1s0-system" = {
    tier              = "system"
    allowed_from_tier = "business"
  }
  "k1s0-business" = {
    tier              = "business"
    allowed_from_tier = "service"
  }
  "k1s0-service" = {
    tier              = "service"
    allowed_from_tier = ""
  }
  "observability" = {
    tier              = "infra"
    allowed_from_tier = ""
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
```

## 運用ルール

- `terraform plan` の結果を PR に添付し、レビューを受けてから `terraform apply` を実行する
- prod 環境への apply は 2 名以上の承認を必須とする
- State のロックを確認してから操作を行う
- `terraform import` は既存リソースの取り込み時のみ使用し、手動変更は禁止する
