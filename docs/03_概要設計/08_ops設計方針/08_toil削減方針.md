---
id: arch.ops.toil_reduction_policy
axis: ops
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.ops.ops_index
  - arch.ops.runbook_policy
  - arch.ops.alert_policy
covered_by:
  defense_in_depth_layers: [B, D, E]
  proof_classes: []
lock_artifacts:
  - automation_backlog.lock.yaml
  - toil_quarterly_target.lock.yaml
  - toil_event_history.lock.yaml
  - toil_slo_bindings.lock.yaml
---

# ops toil 削減方針

## 一文方針
- 全 on-call activity を `toil_classes.yaml` の 5 class に分類して `toil_minutes_per_week` metric として ClickHouse に永続化し、SRE 50% rule（個人別 toil ≤ 50% / week）を SLO 化、quarterly target で monotonic decrease を ship blocker として bind、減少達成は automation backlog の Tekton Pipeline 化 / runbook 化 / SLO 再 tuning の三本柱で物理 enforce する。

## 至高路線における立ち位置
- 文章 only の toil 削減目標禁止、quarterly target は build artifact、未達成は ship blocker
- toil の手動 spreadsheet 管理禁止、ClickHouse 自動集計のみ
- SRE 50% rule の opt-out 禁止
- automation backlog の永久放置（P3 のまま 4 quarter 以上）禁止
- 「toil 削減は次 quarter で」を理由に SRE 50% rule 違反を放置禁止
- automation の half-baked 化（manual fallback path 残存）禁止
- toil 計上の対象を engineering work まで広げる禁止

## toil の定義（5 class）

| class | 内容 | 削減経路 |
|---|---|---|
| `v1_manual_runbook` | Markdown only runbook の手動実行（[runbook 方針](07_runbook方針.md) で cap 5%）| Tekton Pipeline 化（v1_full_auto / v1_partial_auto への migration）|
| `v1_alert_triage` | false positive alert の triage / silence 操作 | alert tuning（noise budget violation → threshold 調整 / SLO 再 tune）|
| `v1_repetitive_ops` | 繰返発生する manual operation（rotation / cleanup / certificate renewal の手動部分等）| Argo Workflows CronWorkflow 化 / External Secrets Operator 化 / cert-manager 化 |
| `v1_human_handoff` | 部分自動化されているが人間の確認 / 承認が必要な step（特に severity_freeze 解除等）| dry-run automation での pre-validation + ChatOps 二人承認の standardize |
| `v1_postmortem_overhead` | postmortem 記述に必要な手動 timeline 整理 / metric query / artifact 収集 | postmortem auto-generator 強化（timeline 自動生成 / metric query 自動 embed / artifact 自動 link）|

### non-toil（engineering work）
- design / architecture / 新機能実装 / 新軸 spec 作成 / 新 OSS 評価 / pair_target migration の設計部分 / chaos drill scenario の新規追加
- 上記は `toil_minutes` に含めない（自動化対象外、削減対象は toil 5 class のみ）

## toil 計測

### 物理計測手段
- 自製 escalation engine: page response 時間 / runbook execution 時間（v1_alert_triage / v1_manual_runbook）
- GitHub Actions: CronWorkflow 化前 manual ops の自己申告 timer（v1_repetitive_ops、ChatOps 経由 `/toil-start` `/toil-stop`）
- Tekton Pipeline: dry-run 後の human approval 時間（v1_human_handoff）
- postmortem PR: author 主観の time spent 自己申告（v1_postmortem_overhead）

### 集計
- ClickHouse 上の `toil_event` subject に永続、weekly aggregation で `toil_minutes_per_week` metric を Sloth で SLO 化
- dashboard: Perses で個人 / team / 軸別 toil ratio を可視化、quarterly review で必須 review

## SRE 50% rule（v1 SLO 化）

### per-individual
- `v1_individual_toil_ratio ≤ 0.50`（toil_minutes_per_week / total_working_minutes_per_week）

### per-team
- `v1_team_toil_ratio ≤ 0.50`（team 全 individual の weighted average）

### 違反時の処置
1. 即時 alert（noise budget alert routing 経由）+ Mattermost #ops-toil channel 投稿
2. quarterly review で恒常的 violation の根本原因解析
3. 連続 4 週違反は automation backlog item の implementation を ship blocker

## quarterly toil target（monotonic decrease ship blocker）
- 各 quarter で前 quarter 比 ≥ 5% 減少を target、未達成は次 quarter ship blocker（ただし減少率改善 PR が merged されれば release allow）
- target 例（v1）:
    - Q1 baseline: team 全体 `toil_minutes_per_week` 中央値 240 min/person
    - Q2 target: ≤ 228 min（-5%）
    - Q3 target: ≤ 217 min（-5%）
    - Q4 target: ≤ 206 min（-5%）
- quarterly target は `toil_quarterly_target.lock.yaml` に build artifact 化

## automation backlog
- 全 toil 識別済 item は `automation_backlog.lock.yaml` に build artifact 化
- 各 row: `(item_id, toil_class, source, owner, expires_at, automation_target, implementation_status, quarterly_priority)`

### quarterly priority
- **P0**: ship blocker（同 root cause の incident recurrence が 3 回以上、または SRE 50% rule 連続 4 週違反）
- **P1**: current quarter 内 implementation target
- **P2**: next quarter 内 implementation target
- **P3**: backlog 維持（quarterly review で再評価）

### implementation 完了
- Tekton Pipeline 化 / runbook v1_partial_auto 以上への昇格 / SLO 再 tune の merged PR を伴う
- 自己申告のみは禁止
- 完了後 30 日間は `toil_minutes_per_week` metric で削減効果検証

## 三本柱の削減経路

### 経路 1: Tekton Pipeline 化（v1_manual_runbook → v1_full_auto / v1_partial_auto）
- [runbook 方針](07_runbook方針.md) と双方向 lock
- Pipeline build artifact + dry-run 検証 + ChatOps trigger を一セットで delivered

### 経路 2: alert tuning（v1_alert_triage 削減）
- [アラート方針](02_アラート方針.md) の noise budget violation handling と双方向 lock
- threshold 調整 / SLO 再 tune / runbook 改善 / alert deprecation の四経路

### 経路 3: CronWorkflow 化 / Operator 化（v1_repetitive_ops 削減）
- 例: cert-manager 化（手動 cert renewal を回避）/ External Secrets Operator 化（手動 secret 配信回避）/ Argo Workflows CronWorkflow 化（定期 cleanup の自動化）

## toil 削減と engineering balance
- SRE 50% rule で確保された 50% engineering time の使途は本 spec の管掌外
- 限定的なガイド:
    - automation backlog の implementation
    - 新 OSS の pair_target 評価
    - chaos drill scenario の追加
    - new spec / new axis design
    - red team table top scenario の追加

## toil の anti-pattern
- 以下を toil として誤計上しない（CI で検出）:
    - design review / architecture discussion
    - new feature implementation
    - 新軸 spec / 新 runbook scenario の新規執筆
    - chaos drill / red team drill の scenario 設計時間
    - new OSS evaluation
- 上記を toil として計上した PR は CI で誤計上 lint、revision request 必須

## toil の自己申告における honesty 規律
- `toil_event` subject の自己申告 row は audit_event に chain
- 後追いで ChatOps の audit log と cross check
- 不一致は quarterly review で proactive な dialogue（blameless）
- 過小申告のリスクが大きいため、申告 honesty を文化として強調

## 強制機構との bind
- 本方針は [ops 強制機構](../../04_詳細設計/02_強制機構/07_ops強制機構.md) の以下経路で物理 enforce される:
    - 層 B: Conftest による toil quarterly target monotonic decrease check
    - 層 D: Kyverno `block-on-toil-target-miss` admission policy
    - 層 E: cosign signed `automation_backlog.lock.yaml`

## 採用しない方針
- 文章 only の toil 削減目標: 禁止
- toil の手動 spreadsheet 管理: 禁止
- SRE 50% rule の opt-out: 禁止
- automation backlog の永久放置: 禁止
- 「toil 削減は次 quarter で」を理由に SRE 50% rule 違反を放置: 禁止
- automation の half-baked 化: 禁止
- toil 計上の対象を engineering work まで広げる: 禁止

## 関連参照
- [ops 設計方針 index](README.md)
- [アラート方針](02_アラート方針.md)
- [runbook 方針](07_runbook方針.md)
- [運用インシデント方針](04_運用インシデント方針.md)
