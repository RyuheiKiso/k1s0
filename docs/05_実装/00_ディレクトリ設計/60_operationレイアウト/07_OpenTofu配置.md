# 07. OpenTofu 配置

本ファイルは `deploy/opentofu/` 配下の配置を確定する。OpenTofu（Terraform fork）でベアメタル / IaaS プロビジョンを宣言的管理する。

## OpenTofu の採用理由

2023 年 8 月の HashiCorp Terraform ライセンス変更（BSL）を受け、Linux Foundation 配下で OSS fork が OpenTofu として立ち上がり、MPL-2.0 ライセンスで維持される保証を得た。k1s0 は OSS 原則に従い OpenTofu を採用する。

用途:

- **ベアメタルノードのブートストラップ**: PXE boot 設定、OS インストール、kubeadm / k3s 初期化
- **VPN Gateway / Firewall / DNS**: 社内ネットワーク境界の構成
- **クラウド IaaS**（Phase 2 以降）: マルチクラウド・マルチリージョン展開

## レイアウト

```
deploy/opentofu/
├── README.md
├── modules/                        # 再利用可能モジュール
│   ├── baremetal-k8s/
│   │   ├── README.md
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   ├── versions.tf
│   │   └── templates/
│   │       ├── kubeadm-init.sh.tmpl
│   │       └── cloud-init.yaml.tmpl
│   ├── vpn-gateway/
│   │   ├── main.tf                 # WireGuard Gateway
│   │   └── ...
│   ├── dns/
│   │   ├── main.tf                 # CoreDNS forward / Route53 代替
│   │   └── ...
│   ├── harbor/
│   │   └── main.tf                 # Harbor レジストリ外部デプロイ
│   └── backup-storage/
│       └── main.tf                 # S3/MinIO バックアップ bucket
├── environments/
│   ├── dev/
│   │   ├── main.tf                 # modules/ を組合せて dev 環境定義
│   │   ├── terraform.tfvars.sops   # SOPS 暗号化 vars
│   │   └── backend.tf
│   ├── staging/
│   │   └── ...
│   └── prod/
│       └── ...
└── state/
    └── backend-config.tfvars       # backend state 保存先（MinIO S3）
```

## modules/ の構造

### baremetal-k8s/

ベアメタルノードの K8s 初期化を宣言的に記述。実装は PXE server / Ansible playbook を呼び出す形式で、tofu apply で以下が実行される。

1. PXE boot 設定を DHCP サーバに配信
2. Node が PXE boot、Ubuntu 22.04 Server をインストール
3. cloud-init で kubeadm インストール、Control Plane / Worker join
4. `infra/k8s/bootstrap/` を `kubectl apply -k` で展開

```hcl
# modules/baremetal-k8s/main.tf
variable "control_plane_count" {
  type    = number
  default = 3
}

variable "worker_count" {
  type    = number
  default = 3
}

variable "mac_addresses" {
  type = map(string)
}

resource "local_file" "dhcp_config" {
  filename = "${path.module}/dhcp.conf"
  content  = templatefile("${path.module}/templates/dhcp.conf.tmpl", {
    macs = var.mac_addresses
  })
}

resource "local_file" "cloud_init_control_plane" {
  count    = var.control_plane_count
  filename = "${path.module}/cloud-init-cp-${count.index}.yaml"
  content  = templatefile("${path.module}/templates/cloud-init.yaml.tmpl", {
    role  = "control-plane"
    index = count.index
  })
}

# ansible を local-exec で実行
resource "null_resource" "kubeadm_init" {
  provisioner "local-exec" {
    command = "ansible-playbook -i inventory.yaml kubeadm-init.yaml"
  }
  depends_on = [
    local_file.cloud_init_control_plane,
  ]
}
```

### vpn-gateway/ / dns/

社内ネットワーク境界の WireGuard Gateway と、内部 DNS（CoreDNS）の管理。

### harbor/ / backup-storage/

外部 Harbor（コンテナレジストリ）とバックアップ用 MinIO（クラスタ外に置き cluster 全損時の避難先）。

## environments/ の構造

環境ごとに modules/ を組合せて具体化する。

```hcl
# environments/prod/main.tf
terraform {
  required_version = ">= 1.6"
  required_providers {
    null = {
      source  = "hashicorp/null"
      version = "~> 3.2"
    }
  }
  backend "s3" {
    bucket   = "k1s0-opentofu-state"
    key      = "prod/terraform.tfstate"
    endpoint = "https://minio-backup.k1s0.external:9000"
    region   = "us-east-1"  # MinIO dummy region
  }
}

module "baremetal_k8s" {
  source              = "../../modules/baremetal-k8s"
  control_plane_count = 3
  worker_count        = 5
  mac_addresses       = var.node_macs
}

module "vpn_gateway" {
  source = "../../modules/vpn-gateway"
}

module "dns" {
  source = "../../modules/dns"
}

module "backup_storage" {
  source = "../../modules/backup-storage"
}
```

## state 管理

OpenTofu state は `backend "s3"` で MinIO に保存する。MinIO は `modules/backup-storage/` で cluster 外に独立配置（bootstrap 時は手動構築）。state locking は DynamoDB 代替として PostgreSQL backend を Phase 2 で導入検討。

## SOPS 統合

`environments/<env>/terraform.tfvars.sops` は SOPS + AGE で暗号化。tofu apply 前に sops decrypt で平文 tfvars を生成するラッパースクリプト（`ops/scripts/tofu-apply.sh`）を用意。

## Phase 導入タイミング

| Phase | 内容 |
|---|---|
| Phase 0 | 構造のみ |
| Phase 1a | baremetal-k8s / dns（最小） |
| Phase 1b | vpn-gateway / harbor / backup-storage |
| Phase 1c | environments/ 全環境 |
| Phase 2 | マルチリージョン / クラウド IaaS |

## 対応 IMP-DIR ID

- IMP-DIR-OPS-097（OpenTofu 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-CICD-006（OpenTofu 採用）
- DX-CICD-\* / NFR-A-AVL-\*
