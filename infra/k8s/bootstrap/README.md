# infra/k8s/bootstrap — Kubernetes クラスタ Bootstrap

ADR-INFRA-001 / IMP-DEV-POL-006 に従い、k1s0 のオンプレ / マネージド K8s クラスタの
ブートストラップ方式を定義する。本ディレクトリは **production** クラスタ用、
ローカル開発は `tools/local-stack/kind-cluster.yaml` を参照する。

## ファイル

| ファイル | 内容 |
|---|---|
| `cluster-api.yaml` | Cluster API（CAPI）ベースの Cluster + KubeadmControlPlane（prod 推奨） |
| `kubeadm-init-config.yaml` | kubeadm init 用 ClusterConfiguration（CAPI を使わない場合の代替） |

## ブートストラップ方式

prod では以下のいずれかから選択する（環境別 overlay で切替）:

1. **Cluster API ベース**（推奨、`infra/environments/prod/k8s-overlay/` で provider 別に上書き）
   - bootstrap provider: kubeadm
   - control-plane provider: KubeadmControlPlane
   - infrastructure provider: 環境別（vSphere / AWS / GCP / Azure / OpenStack 等）
2. **kubeadm 直接実行**（小規模オンプレで CAPI を採用しない場合）

## ローカル開発との差分

| 観点 | dev（kind） | prod |
|---|---|---|
| node 数 | 1 control-plane + 0 worker | 3 control-plane（HA） + 3+ worker |
| etcd | スタックド（control-plane と同居） | 外部 etcd cluster（オプション） |
| CNI | kindnet | Cilium eBPF（ADR-NET-001） |
| storage | hostPath | StorageClass（CSI provisioner） |
| LB | MetalLB（dev 用 IPAddressPool） | 環境別 LB（CCM / cloud LB） |

## 関連設計

- IMP-DEV-POL-006 — ローカル開発と production の同等性
- ADR-NET-001 — Cilium CNI
