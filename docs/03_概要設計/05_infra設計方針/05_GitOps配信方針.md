---
id: arch.infra.gitops_delivery_policy
axis: infra
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.infra.infra_index
  - arch.infra.cluster_composition_policy
covered_by:
  defense_in_depth_layers: [B, D, E]
  proof_classes: []
---

# infra GitOps 配信方針

## 一文方針
- cluster 状態の SoT は git、配信は Argo CD、progressive delivery は Argo Rollouts、event 駆動自動化は Argo Events。手動 kubectl apply は drift として CI が検出する。

## 至高路線における立ち位置
- Flux 採用しない（Argo CD L1+ 単一深耕）
- Spinnaker 採用しない（複雑性 / pair_target 候補としても評価しない）
- manual kubectl apply / helm upgrade 禁止
- cluster-side automation script（cron + kubectl）禁止、全自動化は Argo Events / Argo Workflow / Tekton 経由

## レイヤ構成

### SoT: git monorepo（または GitOps 専用 repo）
- app-of-apps pattern: root ApplicationSet が leaf ApplicationSet を generate
- `cluster_inventory.lock.yaml` が cluster catalog の単一の真（[クラスタ位相適合仕様](../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md) と double-bound）
- ApplicationSet generator は cluster_inventory に基づき topology_class 別の deploy plan を build

### 配信 plane: Argo CD
- 1 cluster = 1 Argo CD instance（multi-cluster mode は cluster failover 時に single point of failure となるため）
- Argo CD 同士の cross-cluster sync は federation pattern（v1_multi_cluster_active_active 以上で必須）

### progressive delivery: Argo Rollouts
- canary / blue-green / experiment の 3 strategy
- AnalysisTemplate で SLO ベース自動 rollback、[SLO 適合仕様](../../04_詳細設計/01_適合仕様/07_SLO適合仕様.md) の error budget freeze と直結

### event 駆動: Argo Events
- region failover trigger（[クラスタ位相適合仕様](../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md) の v1_multi_region_active_passive promote runbook）
- drill 実演 trigger（Litmus chaos experiment 起動）
- drift 検出時の self-healing trigger

## ApplicationSet generator pattern
- **cluster generator**: `cluster_inventory.lock.yaml` を入力に cluster 単位の Application を生成
- **matrix generator**: topology_class × tenant × tier の組合せで Application を生成、tier2 / tier3 の per-tenant deploy に活用
- **pull request generator**: PR ベース ephemeral environment（preview environment）を auto provision
- **cluster decision generator**: Argo Events からの decision に基づく動的 deploy plan
- **Apicurio schema generator**: `apicurio_subjects/` git path を入力に、各 cluster の Apicurio Operator CR を生成

## progressive delivery 戦略

### canary（default）
- 5% → 20% → 50% → 100% の 4 段階
- 各段階で Prometheus metric / 13 SLI を AnalysisTemplate で評価
- error budget freeze 状態では canary を block
- rollback: metric 違反検出後 60 秒以内に automatic

### blue-green
- 重要な data 層 schema 変更（12 v1_l2_starlike_event breaking change 等）で適用
- active / preview の二系統を並行運転、cutover は Service selector switch

### experiment
- feature flag（OpenFeature + flagd）と組合せた A/B
- 13 SLO の sub-class metric を experiment 単位で集計

## ApplicationSet override の禁止
- 個別 cluster 用の override 値は Kustomize overlay で declarative 化、ApplicationSet template を runtime で patch する経路は禁止
- tier1 / tier2 / tier3 の Application が cluster_inventory の topology_class を override する経路は持たない（dimension override 禁止 cross-axis）

## sync 戦略
- automated sync: default true、prune true、self-heal true
- manual sync: break-glass 用途のみ、Backstage 経由の操作 audit 必須
- sync wave: dependency 順序を annotation で宣言、Kyverno + cert-manager + OpenBao が tier 各層より先に sync
- sync timeout: 5 分、超過時は alert + Argo Events trigger で runbook 起動

## drift 検出 / 修正
- drift 検出: Argo CD が定期 reconciliation で git と cluster 状態を比較
- drift 修正: self-heal=true で自動修正、ただし stateful workload（PV / DB）は drift 検出後 alert のみで auto-correction せず（人間 review 必須）
- drift audit: drift event は `infra.gitops.drift.event` として 09 観測可能性 SoR に bulk insert

## promotion パイプライン（dev → stage → prod の 3 環境）
- dev: 単一 cluster（v1_single_zone）、PR merge で auto-deploy
- stage: 本番と同じ topology_class（v1_multi_zone_per_cluster 以上）、自動 promote だが Argo Rollouts canary が必須
- prod: Argo Events で manual approval（OpenBao response wrapping で発行された短期 token 必須）
- cross-region promote: v1_multi_region_active_passive 以上で region 間 promote を Argo Events runbook で stage 化

## 配信失敗時の挙動
- canary 失敗: Argo Rollouts が automatic rollback、13 SLO error budget consumption を audit
- 全 cluster sync 失敗: Argo CD が cluster 単位 alert、Argo Events 経由で oncall
- drift 自動修正失敗: runbook 起動、break-glass cluster-admin token 発行 path に escalate

## 強制機構との bind
- 本方針は [infra 強制機構](../../04_詳細設計/02_強制機構/04_infra強制機構.md) の以下経路で物理 enforce される:
    - 層 B: Conftest による ApplicationSet generator 注釈 check
    - 層 D: Argo CD reconciliation + Argo Rollouts AnalysisTemplate
    - 層 E: cosign signed manifest + git ref pin

## 採用しない方針
- Flux: 採用しない
- Spinnaker: 採用しない
- manual kubectl apply / helm upgrade: 禁止
- cluster-side automation script（cron + kubectl）: 禁止

## 関連参照
- [infra 設計方針 index](README.md)
- [クラスタ構成方針](01_クラスタ構成方針.md)
- [IaC 方針](08_IaC方針.md)
- [クラスタ位相適合仕様](../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md)
