---
id: arch.ops.runbook_policy
axis: ops
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.ops.ops_index
  - arch.ops.alert_policy
  - arch.ops.operational_incident_policy
covered_by:
  defense_in_depth_layers: [B, C, D, E]
  proof_classes: []
lock_artifacts:
  - runbook_catalog.lock.yaml
  - runbook_role.lock.yaml
  - runbook_drill_progress.lock.yaml
  - runbook_archive_index.lock.yaml
---

# ops runbook 方針

## 一文方針
- 全 page-level alert / 全 incident class / 全 break-glass 経路に対し runbook を Backstage TechDocs（人間向け Markdown）+ Tekton Pipeline（実行可能 yaml）の二段構えで必須化し、`runbook_catalog.lock.yaml` に build artifact として固定、runbook 不在の page / incident は CI fail とする。Markdown only runbook は cap 5%（手作業 step 残存例外のみ）、Tekton 実行可能 runbook が default。

## 至高路線における立ち位置
- Markdown only runbook を default に禁止、Tekton 実行可能 runbook が default
- runbook 不在の page / incident 禁止、CI fail
- 演習 skip 禁止、cadence violation は service deploy block
- precondition なし runbook 禁止、機械検証必須
- rollback なし runbook 禁止、step 単位で rollback 必須
- runbook の personal storage（個人 wiki / Notion / 個人 GitHub Gist）禁止、Backstage TechDocs に集約
- severity_freeze / security_bridge の single-actor 実行禁止、二人承認 ChatOps 必須

## runbook の二段構え

### 段 1: human-readable（Backstage TechDocs）
- 目的: on-call が 5 分以内に状況把握 → 行動を取れる短い Markdown
- 内容: summary / preconditions / steps（番号付き）/ verification / rollback / escalation / linked_alerts / linked_postmortems
- 量: steps が 30 以下、推定実行時間が 30 分以下
- 検索: Backstage search で alert / incident class / SLI 名 / 軸名 から到達可能

### 段 2: machine-executable（Tekton Pipeline）
- 目的: 人間 ack 不要 / 部分自動化 / chatops 経由実行を可能にする pipeline yaml
- 内容: Tekton Task 連結、各 Task は idempotent / rollback 可能 / dry-run mode を持つ
- trigger 経路: Alertmanager webhook → Tekton Triggers / Mattermost slash command → Tekton Triggers / 手動 kubectl create
- 量: 1 pipeline = 1 runbook task class

- 全 runbook は段 1 + 段 2 の両方が build artifact（`docs/runbook/<id>.md` + `runbooks/<id>/pipeline.yaml`）として cosign signed で publish、片方欠落は CI fail（Markdown only allow は cap 5%）

## runbook class（5 class）

| class | bind | 例 |
|---|---|---|
| `v1_alert_runbook` | alert_catalog の page-level alert に 1:1 | tier1-latency-triage / data-rpo-triage / replication-lag-triage / cve-patch-overdue |
| `v1_incident_runbook` | incident_classes.yaml の 5 class に bind | slo-breach-triage / capacity-throttle / change-rollback / dependency-failover |
| `v1_break_glass_runbook` | break-glass token 発行 / cluster-admin 操作 / DB superuser 操作 | emergency-deploy-freeze / break-glass-token-issue / break-glass-revoke |
| `v1_drill_runbook` | chaos / restore / red team drill の実行 pipeline | chaos-zone-partition / restore-rpo-test / red-team-table-top-replay |
| `v1_change_runbook` | change management の orchestration | cluster-upgrade-pipeline / topology-cutover / migration-pair-cutover |

## runbook_catalog.lock.yaml（build artifact）
- 1 row = `(runbook_id, class, alert_ref, incident_class_ref, break_glass_ref, drill_class_ref, change_class_ref, markdown_ref, pipeline_ref, executable, last_drill_at, last_drill_outcome)`
- `executable`:
    - `v1_full_auto`
    - `v1_partial_auto`
    - `v1_human_only`（cap 5%、CI で count enforce）
- `last_drill_at`: 当 runbook の最後の演習実行 timestamp、cadence 内に演習実行されていることを CI 検証

## runbook の演習 cadence

| class | cadence |
|---|---|
| `v1_alert_runbook` | 90 日に 1 回（alert simulation + runbook execution）|
| `v1_incident_runbook` | 90 日に 1 回（chaos drill / table top exercise）|
| `v1_break_glass_runbook` | 30 日に 1 回（実 token 発行 + 即時 revoke）|
| `v1_drill_runbook` | drill 自体（self-bound）|
| `v1_change_runbook` | 60 日に 1 回（dry-run mode で実 cluster に対し idempotent 検証）|

- 演習結果は `drill_progress.lock.yaml`（security と共有）の対応 entry に書き込み

## runbook の品質規律

### precondition の機械検証
- runbook 実行前に Tekton Task で「現在の cluster state が precondition を満たすか」を assert
- 不一致時は実行 abort

### idempotency
- Tekton Task 単位で idempotent（同 input で複数回実行しても同 outcome）、retry-safe
- 副作用ありの Task は audit_event 必須

### rollback
- 各 step に rollback procedure を必ず付随
- step 半ばで abort された場合の cleanup pipeline を別 Tekton Pipeline として宣言

### dry-run mode
- 全 runbook は `--dry-run` flag で副作用なし実行可能、CI / pre-flight で routinely 実行

### timeout
- 各 step に timeout 必須、timeout 超過時は escalate flag を Mattermost incident channel に投稿

## ChatOps integration

### Mattermost slash command
- `/runbook list <alert_id>`: 候補表示
- `/runbook describe <runbook_id>`: 詳細表示
- `/runbook run <runbook_id> --dry-run`: dry-run
- `/runbook run <runbook_id>`: 本番実行（incident_ref 必須）

- 全 ChatOps 操作は audit_event subject に `v1_chatops` fact emit
- ChatOps 実行は Tekton Triggers 経由、`runbook_role.lock.yaml` で「誰がどの runbook を実行できるか」を rotation 階層別に制御
- severity = freeze / security_bridge の runbook 実行は on-call primary + secondary の二人承認 ChatOps 必須

## runbook の自動生成（partial）
- 軸別 spec の defense-in-depth 層 D（runtime）から runbook の skeleton を自動生成
- alert_catalog の各 alert に対し template runbook を generator が出力
- runbook 自動生成器は infra 11 / data 11 / security 12 の drill scenario と双方向 lock

## runbook の deprecation
1. `runbook_catalog.lock.yaml` に `deprecated=true` 注釈
2. 30 日間は warn-only（実行は可能だが warning 表示）
3. 30 日後 catalog から removal、Tekton Pipeline も削除
- removal 後の runbook 文献は archive_index に永続 retention

## 強制機構との bind
- 本方針は [ops 強制機構](../../04_詳細設計/02_強制機構/07_ops強制機構.md) の以下経路で物理 enforce される:
    - 層 B: Conftest による runbook ↔ alert / incident / break-glass / drill / change の 1:1 coverage check
    - 層 C: chainsaw による全 v1_full_auto / v1_partial_auto runbook の dry-run 試験
    - 層 D: Kyverno `require-runbook-for-alert` `block-on-runbook-drill-overdue` admission policy

## 採用しない方針
- Markdown only runbook を default: 禁止
- runbook 不在の page / incident: 禁止
- 演習 skip: 禁止
- precondition なし runbook: 禁止
- rollback なし runbook: 禁止
- 商用 runbook automation SaaS（Rundeck Enterprise / StackStorm Pro 等）: 採用しない
- runbook の personal storage: 禁止
- severity_freeze / security_bridge の single-actor 実行: 禁止
- audit_event なし runbook 実行: 禁止

## 関連参照
- [ops 設計方針 index](README.md)
- [アラート方針](02_アラート方針.md)
- [運用インシデント方針](04_運用インシデント方針.md)
- [変更管理方針](06_変更管理方針.md)
- [toil 削減方針](08_toil削減方針.md)
