---
id: arch.infra.iac_policy
axis: infra
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.infra.infra_index
  - arch.infra.gitops_delivery_policy
  - detail.infra.infra_enforcement
covered_by:
  defense_in_depth_layers: [A, B, D]
  proof_classes: []
---

# infra IaC 方針

## 一文方針
- cluster 自体は OpenTofu、cluster 内 manifest は Kustomize + Helm、ローカル開発は Tilt。手動操作で構築・変更する経路は持たない。

## 至高路線における立ち位置
- Pulumi 採用しない（OpenTofu L1+ 単一深耕）
- Crossplane v2 候補
- kubectl manifest yaml の直編集禁止
- Ansible / Chef / Puppet 採用しない（cluster bootstrap 後の構成管理は不要、Kubernetes 自体が宣言的）

## レイヤ構成

### 層 1: 物理基盤 / 仮想基盤
- OpenTofu（Terraform fork、MPL 2.0）
- 対象: cluster bootstrap、node pool 作成、network / storage primitives、cloud account / on-prem provisioning

### 層 2: cluster 内 OSS
- Helm（3rd party チャート、tag 固定参照）
- Kustomize（自製 / overlay 適用）
- 対象: Argo CD / Kyverno / cert-manager / OpenBao / External Secrets / Rook / Longhorn / KEDA / Litmus / Istio / Envoy Gateway / Jaeger v2 / ClickHouse Operator 等

### 層 3: tier1 / tier2 / tier3 / data Application
- Kustomize（自製 manifest）
- Argo CD ApplicationSet で配信

### 層 4: 開発者 inner loop
- Tilt（local Kubernetes、kind / minikube 上で iterative development）

## OpenTofu 規約
- state: remote state（Ceph RGW backend with state locking via DynamoDB-compatible 機能）
- module: `cluster_inventory.lock.yaml` と double-bound、cluster 単位の module を 1 つの真として持つ
- plan / apply: CI が plan を実行、PR レビュー必須、apply は merge 後 GitHub Actions で
- destroy: 明示的 PR + manual approval（OpenBao response wrapping token）必須
- drift: weekly OpenTofu plan で drift 検出、検出後は alert + 自動 import / 手動 review

## Kustomize / Helm の使い分け
- 自製 manifest: Kustomize（base + overlay）
- 3rd party OSS: Helm chart を Kustomize で wrap（helm template 出力を Kustomize の resource として import）
- 値の override: Kustomize patches / Helm values の双方を overlay layer で declarative 化
- ApplicationSet template runtime patch は禁止（dimension override 禁止）

## Helm chart の管理
- Helm chart の出所: upstream chart repo を Harbor の OCI registry に mirror、cluster からは Harbor のみ pull
- chart version: tag 固定参照、`*` / latest は禁止
- chart 改修: fork は最小限、upstream に PR を投げる規律を維持。fork 維持は [OSS ライフサイクル適合仕様](../../04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md) の pair_target 移行 trigger 候補

## Tilt（ローカル開発）
- kind / k3d で local Kubernetes、Tilt で iterative development
- 本番 manifest と Tilt manifest は同一の Kustomize base を共有、overlay のみ dev / tilt 別
- 本番 cluster と divergence しないよう、Tilt 専用の手動 workaround は CI が drift として検出

## secret の IaC 内取扱い
- secret 値を IaC ファイルに直書きすることは禁止
- OpenBao の reference のみを IaC に書き、External Secrets が runtime に inject
- state に secret が含まれる場合（OpenTofu の sensitive output 等）は state 自体を OpenBao 暗号化 backend に保存、state 内 secret 値は HMAC でマスク

## IaC code 規約
- linting: tflint / terraform fmt（OpenTofu 互換）/ kustomize build / helm lint を CI 必須
- testing: Terratest / kuttl / chainsaw で unit / integration test
- module versioning: semantic versioning、breaking change は [スキーマ進化適合仕様](../../04_詳細設計/01_適合仕様/06_スキーマ進化適合仕様.md) の v1_config_yaml class と同型扱いで managed deprecation

## rollback
- cluster 単位 rollback: OpenTofu state previous revision に revert + apply
- manifest 単位 rollback: Argo CD で git revert、自動 sync
- DB / data 層 rollback: [データ保全適合仕様](../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md) の restore_drill 経路と整合

## promotion（dev → stage → prod）
- [GitOps 配信方針](05_GitOps配信方針.md) と同期
- IaC PR が merge されると Argo CD が自動 sync、Argo Rollouts canary で段階的 promote

## 強制機構との bind
- 本方針は [infra 強制機構](../../04_詳細設計/02_強制機構/04_infra強制機構.md) の以下経路で物理 enforce される:
    - 層 A: tflint / terraform fmt / kustomize build / helm lint / kubeval
    - 層 B: Conftest による IaC build artifact 生成 check
    - 層 D: OpenBao response wrapping token + Argo CD reconciliation

## 採用しない選択肢
- Pulumi: 採用しない
- Crossplane: v2 候補
- kubectl manifest yaml の直編集: 禁止
- Ansible / Chef / Puppet: 採用しない

## 関連参照
- [infra 設計方針 index](README.md)
- [GitOps 配信方針](05_GitOps配信方針.md)
- [クラスタ構成方針](01_クラスタ構成方針.md)
