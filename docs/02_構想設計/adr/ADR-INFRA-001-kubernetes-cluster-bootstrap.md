# ADR-INFRA-001: Kubernetes クラスタを kubeadm + Cluster API で構築する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / インフラ運用チーム / 採用検討組織

## コンテキスト

k1s0 はオンプレミス完結を要件とする 採用側組織向け PaaS であり（NFR-F-SYS-001）、クラウド事業者のマネージド Kubernetes（EKS / GKE / AKS）に依存する選択肢は採れない。一方で Kubernetes クラスタを「素手で構築する」と運用ノウハウが属人化し、HA control-plane の構成・etcd バックアップ・kubelet バージョン整合・CNI 選定・OS パッチ追従といった作業を採用組織の小規模運用チーム（NFR-C-NOP-001）が自前で抱えることになる。

採用検討組織は「オンプレ K8s を 10 年保守する」前提で k1s0 を評価するため、ブートストラップ方式の選定は以下を満たす必要がある。

- **完全 OSS**（採用組織のコスト構造に合致、BC-COST-003）
- **vanilla K8s 派生でない**（CNCF Conformance 互換、ADR-CNCF-001 と整合）
- **HA 3 control-plane を標準**（NFR-A-CONT-001 の RTO 4 時間達成）
- **宣言的管理**（GitOps と整合、Argo CD で cluster lifecycle まで版管理可能）
- **環境差分の低コスト切替**（vSphere / AWS / GCP / OpenStack / ベアメタル）
- **採用組織のスキル流用性**（運用エンジニアが世間で標準的に学ぶスキルで回せる）

クラスタ ブートストラップの選択は **one-way door** に近い決定で、後から方式を入れ替えるとデータ移行・ネットワーク再設計・GitOps tree の総書き換えが発生する。リリース時点で方式を確定させ、採用検討組織が世代交代しても保守できる構造を残す。

## 決定

**production の Kubernetes クラスタ ブートストラップは Cluster API（CAPI）+ kubeadm（KubeadmControlPlane）を標準とし、Bootstrap Provider に kubeadm、Infrastructure Provider に環境別（vSphere / AWS / GCP / Azure / OpenStack / Bare Metal）を選択する構成で運用する。**

- 採用 Kubernetes バージョン: 1.31 LTS、四半期パッチ自動追従
- control-plane HA: 3 ノード（stacked etcd または external etcd は環境別 overlay で選択）
- worker: 最小 3 ノード、`autoscaling`（KEDA、ADR-SCALE-001）と整合する node pool 構成
- 小規模オンプレ（CAPI を採用しない単一クラスタ）向けに kubeadm 直接実行も維持
- ローカル開発は kind（`tools/local-stack/kind-cluster.yaml`、ADR-POL-002）で 1 control-plane + 任意 worker

`infra/k8s/bootstrap/` 配下に `cluster-api.yaml`（推奨）と `kubeadm-init-config.yaml`（代替）の 2 系統を併置し、環境別 overlay (`infra/environments/{dev,staging,prod}/`) で切り替える。

## 検討した選択肢

### 選択肢 A: Cluster API + kubeadm（採用）

- 概要: SIG Cluster Lifecycle 製の K8s ネイティブ プロビジョニング フレームワーク。Cluster / KubeadmControlPlane / MachineDeployment 等の CRD で宣言的にクラスタを管理する
- メリット:
  - K8s 公式（SIG Cluster Lifecycle）、CNCF Graduated（Provider Ecosystem は CNCF Sandbox 含む）
  - Apache 2.0、完全 OSS
  - vanilla K8s（CNCF Conformance 互換）
  - vSphere / AWS / GCP / Azure / OpenStack / MAAS / Metal3 等の Infrastructure Provider が公式・コミュニティから提供されており、環境差分が overlay で吸収できる
  - cluster lifecycle（upgrade / scale / delete）が宣言的、Argo CD と統合可能
  - kubeadm を内部で呼ぶため運用エンジニアが普通に学ぶスキルがそのまま流用できる
- デメリット:
  - 学習曲線がある（Provider 体系の理解が必要）
  - bootstrap cluster（CAPI 自身を動かす管理用 K8s）が別途必要

### 選択肢 B: kubeadm 直接実行（オンプレ小規模で代替維持）

- 概要: kubeadm init / kubeadm join を手動 or Ansible 等で実行
- メリット:
  - シンプル、追加 component なし
  - 公式 documented procedure
- デメリット:
  - cluster lifecycle が手続き的、宣言管理が困難
  - HA control-plane の構築・upgrade を手動で組む必要
  - 大規模化・マルチクラスタ化で運用が破綻

### 選択肢 C: Rancher RKE2

- 概要: SUSE 社の Rancher 系 K8s ディストリビューション
- メリット:
  - インストールが平易、HA / etcd 同居が標準
  - Rancher UI で管理可能
- デメリット:
  - vanilla K8s 派生（CNCF Conformance は通るが SUSE 独自 component を含む）
  - SUSE 商用サポート前提の features（cluster autoscaler の挙動等）に部分的にロックイン
  - CNCF Graduated ではない

### 選択肢 D: Talos Linux

- 概要: Sidero Labs 製の K8s 専用 immutable Linux ディストリビューション
- メリット:
  - SSH 不在 / 全 API 駆動 / immutable で攻撃面が極小
  - K8s 用途に純化された OS、運用 surface が単純
- デメリット:
  - 採用組織の運用エンジニアが普通の Linux で運用したい場合と乖離
  - Talos 固有の API（machined）の学習が要る
  - 採用組織が既存 RHEL / Ubuntu 標準に揃える方針なら整合しない

### 選択肢 E: マネージド K8s（EKS / GKE / AKS）

- 概要: クラウド事業者のマネージド K8s
- メリット:
  - control-plane 運用を事業者に委譲できる
  - autoscaling / Node Group が自動
- デメリット:
  - **オンプレ完結要件（NFR-F-SYS-001）に違反**、選択肢として成立しない
  - 海外リージョンへのデータ流出懸念（NFR-G-RES-001 国内保管要件）

## 決定理由

選択肢 A（CAPI + kubeadm）を採用する根拠は以下。

- **業界標準性**: CAPI は SIG Cluster Lifecycle が公式管理する vanilla K8s 標準の cluster provisioning であり、採用組織の世代交代後も「世間で標準的に学ぶスキル」で保守できる。Talos（D）の特殊性、RKE2（C）のディストリビューション独自性は 10 年保守で重荷になる
- **オンプレ完結要件との整合**: マネージド K8s（E）は NFR-F-SYS-001 違反で論外。kubeadm 直接（B）は宣言管理が崩れるため、採用初期段階の小規模オンプレでのみ代替維持
- **環境差分の overlay 吸収**: Infrastructure Provider 切替で vSphere / AWS / GCP / OpenStack / Bare Metal の差分を `infra/environments/{dev,staging,prod}/` overlay 1 段で吸収できる。CAPI 以外だと環境別に bootstrap script を持つ必要があり、ディレクトリ設計（ADR-DIR-002）が破綻する
- **GitOps 整合**: Cluster CR / KubeadmControlPlane CR を Argo CD で管理することで、cluster lifecycle 自体がコードレビュー対象になる（ADR-CICD-001 / ADR-POL-002 の SoT 思想と整合）
- **退路の確保**: kubeadm 直接実行（B）を `kubeadm-init-config.yaml` として併置することで、CAPI を運用したくない採用組織や検証段階での退路を残す。CAPI への移行は kubeadm 由来 cluster の adoption 機能で可能

## 帰結

### ポジティブな帰結

- vanilla K8s 互換が維持され、ADR-CNCF-001 の Conformance 宣言と整合
- cluster lifecycle が宣言的になり、Argo CD で cluster 自体を版管理できる
- 環境差分が overlay 1 段で吸収され、採用組織の環境（vSphere / AWS / etc）への展開が低コスト
- 採用組織の運用エンジニアが標準スキルで保守できる（Talos / RKE2 の独自学習コストを回避）

### ネガティブな帰結 / リスク

- CAPI Provider 体系の学習コストがある。`docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/02_k8sブートストラップ.md` で Runbook 化する必要
- bootstrap cluster（management cluster）が別途必要で、初回構築の 2 段階性がある（first cluster は kubeadm 直接で立てて CAPI を install、self-management に切り替える）
- Infrastructure Provider のバージョン追従が必要（vSphere CAPV 等は別途リリースサイクル）

### 移行・対応事項

- `infra/k8s/bootstrap/cluster-api.yaml` を CAPI v1.8+ で標準化、Infrastructure Provider 別の overlay を `infra/environments/prod/k8s-overlay/` に分離
- kubeadm 直接実行版（`kubeadm-init-config.yaml`）は採用初期 / 検証用に維持、CAPI 移行手順を Runbook 化
- Cluster CR を Argo CD ApplicationSet で管理する経路を `deploy/apps/application-sets/infra-clusters.yaml` で確立
- bootstrap cluster の運用方針（kind / 別物理クラスタ）を採用組織のリソース状況で選択する手順を Runbook 化

## 関連

- ADR-DIR-002（infra 分離）— `infra/k8s/bootstrap/` の物理配置
- ADR-CNCF-001（CNCF Conformance）— vanilla K8s 維持の整合性
- ADR-NET-001（CNI 選定）— bootstrap 後の CNI 適用
- ADR-CICD-001（Argo CD）— Cluster CR の GitOps 管理
- ADR-POL-002（local-stack SoT）— dev / kind との切り分け
- IMP-DEV-POL-006（ローカル開発スタック）

## 参考文献

- Cluster API 公式: cluster-api.sigs.k8s.io
- kubeadm 公式: kubernetes.io/docs/setup/production-environment/tools/kubeadm/
- CNCF Project Maturity Levels
