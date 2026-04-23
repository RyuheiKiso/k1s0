# 07. OpenTofu 配置

本ファイルは `deploy/opentofu/` 配下の配置を確定する。OpenTofu（Terraform fork）でベアメタル / IaaS プロビジョンを宣言的管理する。

## OpenTofu の採用理由

2023 年 8 月の HashiCorp Terraform ライセンス変更（BSL）を受け、Linux Foundation 配下で OSS fork が OpenTofu として立ち上がり、MPL-2.0 ライセンスで維持される保証を得た。k1s0 は OSS 原則に従い OpenTofu を採用する。

用途:

- **ベアメタルノードのブートストラップ**: PXE boot 設定、OS インストール、kubeadm / k3s 初期化
- **VPN Gateway / Firewall / DNS**: 社内ネットワーク境界の構成
- **クラウド IaaS**（Phase 2 以降）: マルチクラウド・マルチリージョン展開

## レイアウト

OpenTofu は **2 階層** に分割する。bootstrap 階層がアプリ階層を前提としない（state 保存先が自己の出力に依存しない）ことで循環依存を断つ。

```
deploy/opentofu/
├── README.md
├── bootstrap/                      # 第 1 階層：local state で動作する最小構成
│   ├── README.md
│   ├── modules/
│   │   ├── baremetal-k8s/          # PXE / kubeadm / k3s の初期化
│   │   ├── vpn-gateway/            # WireGuard Gateway
│   │   ├── dns/                    # 内部 DNS
│   │   └── external-minio/         # state 保存用 MinIO（クラスタ外、ベアメタル 1 台）
│   ├── environments/
│   │   ├── dev/
│   │   │   ├── main.tf
│   │   │   ├── terraform.tfvars.sops
│   │   │   └── backend.tf          # backend "local"（state は GitHub 別リポジトリへ commit）
│   │   ├── staging/
│   │   └── prod/
│   └── state-repo/                 # bootstrap state を commit する専用 Git リポジトリの参照設定
│       └── README.md               # state リポ URL / SOPS 暗号化方針
└── applications/                   # 第 2 階層：bootstrap 後に動く運用構成
    ├── README.md
    ├── modules/
    │   ├── harbor/                 # Harbor レジストリ（k8s 上）
    │   ├── backup-storage/         # クラスタ内 Longhorn バックアップ
    │   └── cloudflare/             # Phase 2：クラウド DNS / CDN
    ├── environments/
    │   ├── dev/
    │   │   ├── main.tf
    │   │   └── backend.tf          # backend "s3" — bootstrap で作った external-minio を参照
    │   ├── staging/
    │   └── prod/
    └── state/
        └── backend-config.tfvars   # bootstrap の external-minio エンドポイント
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

### external-minio/（bootstrap 階層）

OpenTofu state の保存先 S3 互換ストレージとしてクラスタ外に単独で立てる MinIO。ベアメタル 1 台の最小構成（docker-compose or systemd unit）。applications 階層の state backend になる。bootstrap 階層自身の state は後述のとおり Git 管理とし、MinIO に依存しない。

### harbor/ / backup-storage/（applications 階層）

k8s クラスタ内で運用する Harbor（コンテナレジストリ）とバックアップ Longhorn。これらは bootstrap 階層完了後にのみ provision される。

## environments/ の構造

### bootstrap/environments/prod/main.tf

```hcl
terraform {
  required_version = ">= 1.6"
  required_providers {
    null = {
      source  = "hashicorp/null"
      version = "~> 3.2"
    }
  }
  # bootstrap は local backend。tfstate は SOPS 暗号化して state-repo に commit
  backend "local" {
    path = "./terraform.tfstate"
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

module "external_minio" {
  source = "../../modules/external-minio"
}
```

### applications/environments/prod/main.tf

```hcl
terraform {
  required_version = ">= 1.6"
  backend "s3" {
    bucket                      = "k1s0-opentofu-state"
    key                         = "applications/prod/terraform.tfstate"
    endpoint                    = "https://minio-ext.k1s0.external:9000"
    region                      = "us-east-1"
    force_path_style            = true
    skip_credentials_validation = true
    skip_region_validation      = true
    skip_metadata_api_check     = true
  }
}

module "harbor" {
  source = "../../modules/harbor"
}

module "backup_storage" {
  source = "../../modules/backup-storage"
}
```

## state 管理

### bootstrap 階層の state

`backend "local"` で local 生成 → SOPS + AGE で暗号化 → **別 GitHub リポジトリ** `k1s0/k1s0-opentofu-state`（プライベート）に `git push`。プル時は `git pull` → SOPS decrypt → `tofu plan/apply`。state locking は state リポの branch protection + `git pull --ff-only` で代替する（2 名運用前提で実質的な競合は稀）。

### applications 階層の state

`backend "s3"` で bootstrap が立てた外部 MinIO（`minio-ext.k1s0.external`）に保存。これにより applications 階層は通常の S3 backend の操作感を得る。state locking は Phase 2 で PostgreSQL backend 導入を検討。

### 循環依存が断たれている根拠

1. bootstrap 階層の state → 外部 Git（OpenTofu とは独立した永続層）
2. bootstrap 階層 apply → external-minio が立つ
3. applications 階層の state → external-minio に保存
4. applications 階層 apply → Harbor 等が立つ

bootstrap 階層が applications 階層の成果物（例えば k8s 上の MinIO）に依存しないため、初回 provision でも chicken-and-egg が起きない。

## SOPS 統合

`environments/<env>/terraform.tfvars.sops` は SOPS + AGE で暗号化。tofu apply 前に sops decrypt で平文 tfvars を生成するラッパースクリプト（`ops/scripts/tofu-apply.sh`）を用意。

## Phase 導入タイミング

| Phase | 内容 |
|---|---|
| Phase 0 | 構造のみ |
| Phase 1a | bootstrap 階層（baremetal-k8s / dns / external-minio）。state リポ新設 |
| Phase 1b | bootstrap 追加（vpn-gateway）、applications 階層（harbor / backup-storage） |
| Phase 1c | environments/ 全環境（dev / staging / prod）の applications 階層 |
| Phase 2 | マルチリージョン / クラウド IaaS、PostgreSQL state lock |

## 対応 IMP-DIR ID

- IMP-DIR-OPS-097（OpenTofu 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-CICD-006（OpenTofu 採用）
- DX-CICD-\* / NFR-A-AVL-\*
