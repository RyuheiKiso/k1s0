---
id: arch.ops.slo_driven_ops_policy
axis: ops
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.ops.ops_index
  - detail.ops.ops_loop_conformance
  - detail.ops.ops_enforcement
covered_by:
  defense_in_depth_layers: [B, D, E]
  proof_classes: []
lock_artifacts:
  - budget_action_bindings.lock.yaml
  - burn_rate_thresholds.lock.yaml
  - freeze_history.lock.yaml
  - rollback_history.lock.yaml
---

# ops SLO 駆動運用方針

## 一文方針
- 全 SLO（13 軸 `error_budget.lock.yaml` の全 entry）に対し、error budget の burn rate を 5 段階の運用 action（observe / notify / page / freeze / rollback）に物理 bind した `budget_action_bindings.lock.yaml` を build artifact として固定し、burn rate threshold 越えは Alertmanager → Argo Rollouts / Argo CD / Kyverno で物理発火する。文章 only の budget policy は CI fail。

## 至高路線における立ち位置
- 単窓 alert 禁止、MWMBR 必須
- SLO 外 alert の page 化を cap 10%（infrastructure 例外のみ）
- budget 残 0% でも deploy 続行を禁止、freeze は物理発火（admission block）
- 手動 budget reset 禁止、window rolling 経路のみ
- SLO target の business 都合緩和禁止

## multi-window multi-burn-rate（MWMBR）方式
- 全 SLO に対し短窓 fast burn と長窓 slow burn の二重 alert を default 必須化
- **短窓 fast burn**: 1h burn rate ≥ 14.4x（30 日 budget の 2% を 1h で消費）→ page
- **長窓 slow burn**: 6h burn rate ≥ 6x（30 日 budget の 5% を 6h で消費）→ page
- **長窓 medium burn**: 3d burn rate ≥ 1x（30 日 budget の 10% を 3 日で消費）→ ticket
- threshold 値は SLO target に応じて Sloth + Pyrra で自動算出、手書き threshold は cap 0%

## 5 段階の運用 action

| phase | burn rate | action |
|---|---|---|
| phase 1: observe | < 1x | 何もしない（dashboard 観測のみ）|
| phase 2: notify | ∈ [1x, 6x) | Mattermost channel に informational 投稿、page しない |
| phase 3: page | ∈ [6x, 14.4x) | 自製 escalation engine 経由で primary on-call へ page、runbook pointer 必須 |
| phase 4: freeze | ≥ 14.4x or budget 残 < 25% | Kyverno admission policy が関連 namespace への deploy を block |
| phase 5: rollback | canary 中 SLO 違反 detect | Argo Rollouts が自動 rollback、active version を直前の stable に revert |

## budget_action_bindings.lock.yaml（build artifact）
- 1 row = `(slo_id, window_class, threshold_burn_rate, action_class, action_pointer, runbook_pointer)`
- action_pointer の物理形式:
    - `kind: alertmanager_route` `ref: <route_id>`
    - `kind: kyverno_policy` `ref: <policy_name>`
    - `kind: argo_rollouts` `ref: <rollout_name>`
    - `kind: tekton_pipeline` `ref: <pipeline_name>`
    - `kind: oncall_escalation` `ref: <escalation_id>`
- 文章 only action（kind: textual）禁止
- 全 SLO（v1 で約 50 entry）× 3 window class（fast / slow / medium）= 150 row が完全列挙

## budget arithmetic の物理化
- error budget 残量 = `(1 - SLO_target) - actual_failure_rate × elapsed / window`
- Sloth が recording rule で Prometheus に物化、Pyrra が dashboard panel として visualize
- budget burn rate は VictoriaMetrics 内で per-SLO の metric として永続、ClickHouse に長期 archive
- budget 復活: window 経過後の自動 rolling、手動 reset は禁止
- budget consumption 履歴は postmortem の必須 section

## SLO 別の binding 例（v1）

| SLO | phase 3 page | phase 4 freeze | phase 5 rollback |
|---|---|---|---|
| `v1_request_latency_p99` | runbook:tier1-latency-triage | kyverno:tier1-deploy-freeze-on-latency-burn | argo-rollouts:tier1-server |
| `v1_data_durability` | runbook:data-rpo-triage | kyverno:data-migration-freeze | - |
| `v1_replication_lag` | runbook:replication-lag-triage | kyverno:data-write-freeze | - |
| `v1_patch_window_compliance` | runbook:cve-patch-overdue | kyverno:cve-deploy-block | - |
| `v1_tenant_isolation_violation_count` | security incident bridge | security 14 と AND-gate | - |
| `v1_topology_drift` | runbook:topology-drift-triage | argo-cd:disable-auto-sync | - |

## budget 干渉の禁止（cross-SLO override）
- SLO 間で budget の貸し借り / 流用は禁止
- 例外: incident 発火時の budget 再配分は postmortem で必ず audit_event 化、文章 only 再配分は禁止

## at-large pause（全 SLO 一斉 freeze）
- infrastructure 起因の同時 multi-SLO breach 時に cluster-wide deploy freeze を一斉発動する経路を保持
- 発動条件: 複数 SLO の phase 4 freeze が 30 分以内重複、or infra incident class（cluster_unreachable / control_plane_failure）
- 発動経路: Kyverno cluster-wide policy（namespace label invariant）、解除は ops 主管 break-glass

## 強制機構との bind
- 本方針は [ops 強制機構](../../04_詳細設計/02_強制機構/07_ops強制機構.md) の以下経路で物理 enforce される:
    - 層 D: Argo Rollouts AnalysisTemplate が SLO breach detection で auto rollback
    - 層 D: Kyverno `block-on-burn-rate-fast` `block-on-budget-burnt` admission policy
    - 層 E: cosign signed AnalysisTemplate + budget_action_bindings.lock.yaml

## 採用しない方針
- 単窓 alert: 禁止
- SLO 外 alert の page 化: cap 10%（infrastructure 例外のみ）、超過は CI fail
- budget 残 0% でも deploy 続行: 禁止
- 手動 budget reset: 禁止
- SLO target の business 都合緩和: 禁止
- rollback の手動運用 default: 禁止、auto rollback が default

## 関連参照
- [ops 設計方針 index](README.md)
- [アラート方針](02_アラート方針.md)
- [変更管理方針](06_変更管理方針.md)
- [運用ループ適合仕様](../../04_詳細設計/01_適合仕様/17_運用ループ適合仕様.md)
- [tier1 SLO 適合仕様](../../04_詳細設計/01_適合仕様/07_SLO適合仕様.md)
