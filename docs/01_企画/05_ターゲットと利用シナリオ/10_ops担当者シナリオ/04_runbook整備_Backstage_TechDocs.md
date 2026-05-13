---
id: plan.overview.scenario_ops_runbook_backstage_techdocs
axis: overview
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.ops.ops_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C, D]
  proof_classes: []
---

# runbook整備_Backstage_TechDocs

## 一文方針

新規 incident class 発生時や runbook が陳腐化した時に、Backstage TechDocs で runbook を更新し、SRE 50% rule 下での toil 削減改善を記録する。

> 月曜午前 9 時、先週発生した privilege_escalation incident の postmortem action item として「break-glass runbook の手順が古い」という指摘が上がった。ops 担当者は Backstage TechDocs を開き、`break-glass-execution` runbook の手順 3 を `v1_emergency_step_up` の最新 spec に合わせて更新する。改訂後、月次 toil review の改善記録として Backstage の `toil_reduction_log.md` に改訂内容を追記する。

## ペルソナ要約

主役: ops 担当者（シニア級）、目的: runbook を Backstage TechDocs で最新状態に保ち、toil 削減効果を記録する

## 現状業務での痛み

- runbook が Confluence / Wiki / ローカルメモに分散し、incident 対応時に最新版がどれか分からない
- runbook 更新が postmortem action item として挙がっても、誰も担当しないまま陳腐化が進む
- toil 削減のための改善を記録する場所がなく、改善効果が可視化されない
- 新規 incident class が発生するたびに runbook 新規作成が遅れ、次の incident で同じ手順を一から考えることになる

## k1s0 でこう変わる

- Backstage TechDocs が runbook の single source of truth となり、incident 対応時の参照先が一元化される
- postmortem action item の「runbook 更新」が GitHub Issue として追跡され、due_date 付きで完了が担保される
- `toil_reduction_log.md` で toil 削減改善を記録し、月次 toil review（シナリオ 08）と連動して SRE 50% rule の維持が可視化される
- incident class 別 runbook template が整備され、新規 runbook の作成時間が短縮される

## Trigger

新規 incident class 発生時 / 月次 toil 見直し時 / SRE 50% rule 超過検知時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 月次
- 典型きっかけ: 「postmortem action item で `break-glass-execution` runbook の更新指示が来た」「月次 toil review で runbook 不備による toil が 5% を占めていた」
- 頻度根拠: 月次 toil review（シナリオ 08）と連動するため月次が基本。新規 incident class 発生時はイベント駆動で追加される

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| ops 担当者（主役） | シニア | 本社 IT 室 / リモート | Backstage TechDocs / GitHub Issue | runbook 更新・toil 削減記録 |
| peer reviewer（ops）| シニア | 本社 IT 室 / リモート | Backstage TechDocs | runbook 更新内容の review・sign-off |

## 個人 KPI / 達成感

- postmortem action item の runbook 更新完了率: 100%（due_date 以内）
- runbook 陳腐化率（最終更新から 6 ヶ月超）: 10% 以下
- toil_reduction_log への月次記録率: 100%

## 工数 / 関与人数 / コスト感

- 既存 runbook 更新: 30〜60 分
- 新規 runbook 作成: 1〜3 時間
- peer review: 30 分
- 関与人数: 2 名（ops + peer reviewer）

## 前提

- Backstage TechDocs に runbook ディレクトリが存在し、incident class 別の runbook template が有効
- GitHub に runbook 更新 PR フローが整備されている
- `toil_reduction_log.md` が Backstage TechDocs 内に存在する

## 流れ

1. **更新トリガ確認**: postmortem action item / 月次 toil review / SRE 50% rule 超過のどのトリガで発火したかを確認する
2. **対象 runbook 特定**: Backstage TechDocs で対象 runbook を開き、最終更新日と現状の手順を確認する
3. **更新内容 draft**: 手順の陳腐化箇所を特定し、最新 spec（`v1_emergency_step_up` / `escalation_policy.lock.yaml` 等）に合わせて更新する
4. **新規 runbook 作成（新規 incident class の場合）**: incident class 別 runbook template を使用し、新規 runbook を作成する
5. **GitHub PR 作成**: Backstage TechDocs の runbook を GitHub PR として提出し、peer reviewer を assign する
6. **peer review + merge**: peer reviewer の sign-off 後に merge する
7. **toil 削減記録**: `toil_reduction_log.md` に更新内容と toil 削減効果（推定削減時間）を追記する
8. **Mattermost 報告**: Mattermost `#ops-runbooks` に更新完了を報告し、関係者に周知する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0h | ops 担当者 | 更新トリガ確認・対象 runbook 特定 | — |
| T+1h | ops 担当者 | 更新内容 draft 完了 | — |
| T+2h | ops 担当者 | GitHub PR 作成・peer reviewer assign | GitHub: PR #234 open |
| T+4h | peer reviewer | review 開始 | — |
| T+6h | peer reviewer | sign-off | GitHub: approved |
| T+7h | ops 担当者 | PR merge | Backstage TechDocs 更新完了 |
| T+8h | ops 担当者 | toil_reduction_log 追記・Mattermost 報告 | 「[runbook] break-glass-execution 手順 3 を v1_emergency_step_up spec に合わせて更新」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 警報配信 | 高 | 警報配信 incident の runbook は ops 対応速度に直結 |
| FA 生産指示 | 中 | 生産指示障害 runbook の陳腐化防止 |
| SCADA テレメトリ収集 | 中 | SCADA 関連 incident の runbook 整備 |

## 関連適合仕様 / 関連 OSS

- 運用ループ適合仕様
- 検証規律適合仕様
- 関連 OSS: Backstage TechDocs（runbook 管理）/ GitHub（PR / Issue 管理）/ Mattermost（通知）

## 期待結果 / 観測指標

- postmortem action item の runbook 更新が due_date 以内に完了している
- Backstage TechDocs の runbook 最終更新日が 6 ヶ月以内に保たれている
- `toil_reduction_log.md` に月次の改善記録が追記されている

## 失敗時の挙動 / escalation

- **runbook 更新 GitHub Issue が due_date を超過**: ops チームリーダーに Mattermost で自動 escalation。SLA: 24 時間以内に更新完了または新 due_date 設定
- **peer reviewer が 48 時間以内に review しない**: ops 担当者が Mattermost DM でリマインド。それでも応答なしの場合は ops チームリーダーが代替 reviewer を assign
- **Backstage TechDocs の更新が正常に反映されない**: GitHub Actions の TechDocs build を手動 trigger し、build log を確認する

## 失敗パターン (anti-pattern)

1. **runbook を Backstage 以外の場所（Confluence 等）に作成する**: single source of truth が崩れ、incident 対応時に古い runbook を参照するリスクが生じる
2. **toil 削減効果を推定せずに記録する**: toil review でデータが使えず、SRE 50% rule 遵守の判断ができなくなる
3. **peer review をスキップして直接 main に push する**: 手順ミスが runbook に取り込まれ、次の incident 対応で誤った手順を踏むリスクがある

## 関連参照

- [ops 担当者シナリオ index](README.md)
- [SRE_50pc_rule_toil削減](08_SRE_50pc_rule_toil削減.md)
- [postmortem_PR_merge_gate](03_postmortem_PR_merge_gate.md)
