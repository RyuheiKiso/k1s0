---
id: plan.overview.scenario_ops_postmortem_pr_merge_gate
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

# postmortem_PR_merge_gate

## 一文方針

incident 発生後の postmortem PR を作成 / review し、次 release の物理 prerequisite として merge を完了させる。blameless culture 維持と action item の timeline 記録も完了させる。

> 木曜午前 10 時、ops 担当者が先週の availability_loss incident の postmortem ドキュメントを GitHub で作成する。blameless 記述のチェックリストを確認しながら、timeline と action item 3 件（Kafka partition 設定変更 / SLO alert 閾値見直し / runbook 更新）を draft する。dual reviewer となる ops 担当者とセキュリティ担当者を PR に assign し、`release_gate.lock.yaml` の postmortem_pending フラグが green になるまで次 release は保留となる。

## ペルソナ要約

主役: ops 担当者（シニア級）、目的: incident 終結後に blameless postmortem PR を作成し、次 release の物理 gate として merge を完了させる

## 現状業務での痛み

- postmortem ドキュメントが作成されても release と連動していないため、action item 未完了のまま次 release が走るケースがある
- blame culture が根強く、postmortem の記述が特定個人への批判になりやすい
- action item の追跡が属人化しており、GitHub Issue 化されずに消えてしまうことがある
- postmortem の作成タイミングが曖昧で、incident close から数週間後に作成されることがある

## k1s0 でこう変わる

- `release_gate.lock.yaml` の postmortem_pending フラグが、postmortem PR merge を物理 prerequisite として強制する
- blameless culture チェックリストが PR template に組み込まれ、構造的に blame を排除する
- action item が自動で GitHub Issue 化され、timeline が明示されることで追跡が可能になる
- incident 6 phase の最終 phase（postmortem）が postmortem PR 作成のトリガとして自動化される

## Trigger

incident 6 phase の最終 phase (postmortem) が到達した時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 週次〜月次（incident 発生後）
- 典型きっかけ: 「availability_loss incident の recover phase が完了し、postmortem phase に移行した通知が Mattermost に届いた」
- 頻度根拠: incident 発生頻度に依存。月次 1〜3 件の incident が発生する想定

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| ops 担当者（主役） | シニア | 本社 IT 室 / リモート | Mattermost #ops-incident / GitHub PR list | postmortem draft 作成・action item GitHub Issue 化・PR merge |
| dual reviewer（ops）| シニア | 本社 IT 室 / リモート | GitHub PR list | blameless チェック・action item 妥当性確認・sign-off |
| dual reviewer（security）| シニア | 本社 IT 室 / リモート | GitHub PR list | security action item 確認・sign-off |

## 個人 KPI / 達成感

- postmortem PR 作成: incident recover phase 完了後 48 時間以内
- action item の GitHub Issue 化率: 100%
- postmortem PR merge: incident から 1 週間以内
- blameless 記述違反（特定個人への批判）: 0 件/月

## 工数 / 関与人数 / コスト感

- postmortem draft 作成: 1〜2 時間
- dual review + 修正: 30 分〜1 時間
- 全体: 2〜4 時間
- 関与人数: 3 名（ops + dual reviewer 2 名）

## 前提

- incident 6 phase playbook が `scenario_catalog.lock.yaml` に存在し、postmortem phase 到達の自動通知が設定されている
- `release_gate.lock.yaml` の postmortem_pending フラグが有効
- postmortem PR template（blameless チェックリスト / action item 記載欄 / timeline 欄）が GitHub repository に設定されている

## 流れ

1. **postmortem phase 通知受信**: Mattermost `#ops-incident` に postmortem phase 到達の自動通知が届く
2. **postmortem draft 作成**: GitHub で postmortem PR を作成する。PR template のチェックリストに従い記述する
   - incident 概要・影響範囲・timeline（5W1H 形式）
   - 根本原因（blameless 記述）
   - action item（GitHub Issue 番号付き）
3. **blameless チェック**: PR template のチェックリストで blame 記述がないことを確認する
4. **action item GitHub Issue 化**: 全 action item を GitHub Issue として起票し、due_date と assignee を設定する
5. **dual reviewer assign**: ops 担当者 + security 担当者を PR reviewer に assign する
6. **review 対応**: reviewer のフィードバックを反映し、blameless 記述と action item の妥当性を修正する
7. **PR merge**: dual reviewer の sign-off 後に merge する。`release_gate.lock.yaml` の postmortem_pending フラグが green に更新される
8. **Mattermost 報告**: merge 完了を Mattermost `#ops-incident` に投稿し、関係者に周知する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0h | incident system | postmortem phase 到達通知 | Mattermost: 「[incident #42] postmortem phase に移行しました」 |
| T+4h | ops 担当者 | postmortem draft PR 作成 | GitHub: PR #123 open |
| T+6h | ops 担当者 | action item GitHub Issue 化完了 | GitHub: Issue #124, #125, #126 open |
| T+8h | dual reviewer | review 開始 | — |
| T+24h | ops 担当者 | review フィードバック反映・re-request | — |
| T+36h | dual reviewer | sign-off | GitHub: approved |
| T+48h | ops 担当者 | PR merge | release_gate.lock.yaml postmortem_pending: green |
| T+48h | ops 担当者 | Mattermost 報告 | 「[incident #42] postmortem PR merge 完了。release_gate green」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 警報配信 | 高 | 警報配信 incident の postmortem は最優先で作成・merge する |
| FA 生産指示 | 高 | 生産指示配信障害の postmortem は製造 SLA に直結 |
| ライン稼働監視 | 中 | ライン稼働 SLO 超過 incident の postmortem 作成 |

## 関連適合仕様 / 関連 OSS

- 運用ループ適合仕様
- security 強制機構
- 関連 OSS: GitHub（PR / Issue 管理）/ Mattermost（通知）/ Backstage（postmortem template）

## 期待結果 / 観測指標

- postmortem PR が incident recover 完了後 48 時間以内に作成されている
- 全 action item が GitHub Issue 化され、due_date と assignee が設定されている
- `release_gate.lock.yaml` の postmortem_pending フラグが green に更新されている
- blameless 記述違反が 0 件

## 失敗時の挙動 / escalation

- **postmortem PR が 48 時間以内に作成されない**: `release_gate.lock.yaml` の postmortem_pending フラグが red のまま次 release をブロックし、ops チームリーダーに Mattermost で自動 escalation する
- **blameless チェックリスト未通過**: dual reviewer が sign-off を拒否し、ops 担当者が blameless 記述に修正する。SLA: 24 時間以内に再 review
- **action item の GitHub Issue 化が漏れる**: release_gate の action_item_pending count が正確でなくなるため、自動監査スクリプトが PR merge 前にチェックし fail する

## 失敗パターン (anti-pattern)

1. **incident 対応メモをそのまま postmortem にコピーする**: blame 記述が残りやすく、blameless チェックで引っかかる。draft 前に blameless guidelines を必ず参照する
2. **action item を PR 本文だけに記載し GitHub Issue 化しない**: 追跡が困難になり、due_date 超過が見えなくなる
3. **postmortem PR を後回しにして次 release を優先しようとする**: `release_gate.lock.yaml` が物理ブロックするため release は不可。postmortem merge を最優先にする

## 関連参照

- [ops 担当者シナリオ index](README.md)
- [incident_response_7class主導](09_incident_response_7class主導.md)
- [release_切り_dual_sign_off](13_release_切り_dual_sign_off.md)
