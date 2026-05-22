---
id: detail.ops.ops_enforcement
axis: ops
phase: detail
kind: enforcement
status: draft
depends_on:
  - arch.ops.ops_index
  - arch.ops.slo_driven_ops_policy
  - arch.ops.alert_policy
  - arch.ops.oncall_policy
  - arch.ops.operational_incident_policy
  - arch.ops.postmortem_policy
  - arch.ops.change_management_policy
  - arch.ops.runbook_policy
  - arch.ops.toil_reduction_policy
  - detail.ops.ops_loop_conformance
  - detail.cross_edge.ops_edge_cluster
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes: []
lock_artifacts:
  - ops_enforcement.lock.yaml
trace:
  fr_ids:
  - FR-ops-002

---

# ops 強制機構

## 一文方針
- ops 層の規律は Kyverno admission policy + Argo Rollouts AnalysisTemplate + Argo CD sync wave + 各層の物理機構の cross-axis bind + cosign signed build artifact の AND-gate + ops_event の hash chain 物理 enforce の 6 種で enforce、手書き policy / dashboard / runbook は CI fail、最終 safety net は cosign signature の物理的真正性と Kyverno admission webhook の deploy block。

## 5 層 defense-in-depth

### 層 A: compile（loop / catalog / SLA の type check）
- `ops_loop.lock.yaml` schema 検証（jsonschema / CUE）
- alert_catalog / runbook_catalog / rotation roster の field 完備 check
- budget_action_bindings の各 pointer が実在 artifact を指すこと
- postmortem_section_schema に対する postmortem PR の section 一致 check
- change_classes / incident_classes / toil_classes / runbook_classes / alert_classes の enum 整合
- rollout strategy AnalysisTemplate の SLO query syntax check（PromQL parse）

### 層 B: lint（policy / 規約 check）

#### Conftest custom rule（Rego）
- alert ⇔ runbook 1:1 coverage
- 全 severity = page / freeze に runbook_ref 必須
- ownership_table 完全 coverage（全 service に 4 軸 ownership）
- rotation roster gap = 0
- flag cleanup_due_at 期限内
- postmortem PR SLA 内
- change calendar global freeze 内 PR の v1_emergency 注釈
- automation backlog の P0 item に implementation owner 必須
- toil quarterly target monotonic decrease

#### Semgrep custom rule
- postmortem PR の blameless 文体 lint
- non-toil mis-classification 検出

#### generator output drift = 0
- alert generator output と AnalysisTemplate generator output の手書き drift = 0

### 層 C: integration test（chainsaw / kuttl / Testcontainers）
- 全 ops admission policy が intended state を block / pass する test
- Argo Rollouts AnalysisTemplate の SLO breach simulation で auto rollback 試験
- Tekton Pipeline runbook の dry-run 実行（全 v1_full_auto / v1_partial_auto runbook 対象）
- Mattermost ChatOps slash command の audit_event emit 試験
- flagd build artifact から flag toggle の dev test
- Litmus light scenario で chaos drill runbook の自動 replay
- postmortem PR auto-close & deploy block の end-to-end 試験

### 層 D: runtime
- Kyverno admission webhook: 本層担当 policy の deploy / migration / DDL 全 block
- Argo Rollouts controller: AnalysisTemplate の SLO breach detection で auto rollback 物理発火
- Argo CD: sync wave による change orchestration、drift detect で self-heal
- Alertmanager: alert routing tree / inhibition rule の物理発火
- 自製 escalation engine: rotation gap warning / fatigue budget violation page
- Tekton Triggers: alertmanager webhook → Pipeline 自動起動
- flagd: build artifact から flag definition serving
- cron-based audit chain verifier: ops_event hash chain の online verification

### 層 E: 物理 enforcement
- cosign signature の cryptographic verification
- Kyverno admission webhook の物理拒否
- Ceph RGW Object Lock Compliance mode: postmortem archive / runbook archive の retention
- ops_event hash chain: cryptographic chain divergence の物理特定
- Argo Rollouts pause state: canary 中の SLO breach で controller が pod の new traffic shift を物理停止

## Kyverno admission policy 一覧（ops 担当部分）

| policy 名 | 内容 |
|---|---|
| `require-change-class-annotation` | 全 PR / Deployment に change_class 注釈必須 |
| `require-rollout-strategy` | 本番 namespace の Deployment は Argo Rollouts に変換、AnalysisTemplate 必須 |
| `require-ownership-table-entry` | service deploy 前に ownership_table.lock.yaml に 4 軸 ownership 登録必須 |
| `require-runbook-for-alert` | alert_catalog の page-level alert に runbook_ref 必須 |
| `require-flag-cleanup-due-at` | feature flag 宣言時に cleanup_due_at 注釈必須 |
| `block-on-active-incident` | active incident（status != resolved）の関連 namespace への自動 deploy block |
| `block-on-postmortem-overdue` | postmortem PR 72h SLA 違反の関連 service deploy block |
| `block-on-postmortem-merge-overdue` | postmortem PR 14d merge SLA 違反の service 全 deploy block |
| `block-on-budget-burnt` | SLO budget 残 < 25% の関連 namespace への deploy block |
| `block-on-burn-rate-fast` | fast burn rate ≥ 14.4x の関連 namespace への deploy block |
| `block-on-rotation-gap-unhandled` | rotation_gap_class が v1_unhandled_gap で on-call 関連 PR merge block |
| `block-on-fatigue-budget-violation` | individual / team fatigue budget 連続違反時に paging budget rebalance PR が必要、それまで関連 service deploy block |
| `block-on-flag-cleanup-overdue` | v1_release_flag 90 日越え cleanup 未完了で関連 service deploy block |
| `block-on-experiment-flag-overdue` | v1_experiment_flag 30 日越え未削除で関連 service deploy block |
| `block-on-runbook-drill-overdue` | runbook の last_drill_at が cadence 越えで関連 service deploy block |
| `block-on-toil-target-miss` | quarterly toil target 未達成で次 quarter の deploy block |
| `block-on-non-cosign-rollout` | Argo Rollouts の image / AnalysisTemplate が cosign signed でない場合 block |
| `block-during-global-freeze` | global freeze window 内の v1_emergency 以外の change を block |
| `block-during-per-service-freeze` | per-service freeze 対象 service への change を block |
| `require-mattermost-incident-ref` | active incident 中の break-glass token 発行に incident_ref 必須 |
| `require-audit-event-on-chatops` | 全 ChatOps slash command 実行で audit_event emit 必須 |
| `require-postmortem-section-completeness` | postmortem PR の 6 必須 section 完備 |
| `audit-rotation-swap` | rotation roster の swap 操作を全件 audit_event emit |
| `audit-freeze-bypass` | freeze 解除 / break-glass を全件 audit_event emit |
| `audit-flag-runtime-toggle` | feature flag UI 上手動 toggle を全件 audit_event emit |

## admission policy のライフサイクル
- 全 admission policy は build artifact（`ops/policy/kyverno-ops-policies.yaml`）から生成、手書き禁止
- generator の input:
    - `ops_loop.lock.yaml`（loop ↔ phase ↔ action）
    - `budget_action_bindings.lock.yaml`（SLO ↔ action）
    - `alert_catalog.lock.yaml` / `runbook_catalog.lock.yaml` / `ownership_table.lock.yaml`
    - `incident_lifecycle.lock.yaml` / `incident_history.lock.yaml` / `recurrence_table.lock.yaml`
    - `postmortem_progress.lock.yaml` / `action_item_catalog.lock.yaml`
    - `change_calendar.lock.yaml` / `rollout_strategy.lock.yaml` / `flag_lifecycle.lock.yaml`
    - `automation_backlog.lock.yaml` / `toil_quarterly_target.lock.yaml`
    - `roster.lock.yaml` / `fatigue_log.lock.yaml` / `handover_log.lock.yaml`
    - `error_budget.lock.yaml`（13 SLO）
- drift: runtime 上の admission policy が build artifact と乖離した場合、Argo CD が drift として検出 + self-heal

## break-glass
- 緊急時に admission policy を bypass する経路:
    1. OpenBao response wrapping で短期 cluster-admin 相当 token + Argo CD admin token + Argo Rollouts promote token を発行
    2. 発行操作は audit_event subject に `v1_break_glass` fact emit（必須 column: actor / reason / ticket_ref / expected_duration / incident_ref）
    3. 使用後 1 時間以内に postmortem PR 起票必須
    4. break-glass 使用は 13 SLO error budget consumption として記録
    5. break-glass 経由の policy bypass / freeze 解除 / canary promote / flag toggle / rotation swap も後追いで artifact 化
    6. severity_freeze / security_bridge の break-glass は二人承認必須

## CI 不変条件（17 項）
- [運用ループ適合仕様](../01_適合仕様/17_運用ループ適合仕様.md) の「CI 不変条件（17 項）」を物理 enforce する。違反は merge 不可。

## 採用しない強制機構
- OPA Gatekeeper 単独運用: 採用しない、Kyverno L1+ 深耕
- 商用 admission engine: 採用しない
- 文章 only の運用ルール: 禁止
- 「便利だから break-glass を常用」: 禁止
- 「business 都合で freeze を一時 off」: 禁止
- rollout の手動 promote default: 禁止
- postmortem PR を skip して deploy 続行: 禁止
- audit_event のサンプリング: 禁止
- cosign signing key を CI runner に長期保持: 禁止

## 関連参照
- [ops 設計方針 index](../../03_概要設計/08_ops設計方針/README.md)
- [運用ループ適合仕様](../01_適合仕様/17_運用ループ適合仕様.md)
- [ops 運用 UI](../04_運用UI開発者体験/04_ops運用UI.md)
- [ops_edge_cluster](../03_クロスカッティング適合仕様/10_ops_edge_cluster.md)
