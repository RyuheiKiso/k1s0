---
id: plan.security.scenario_red_team_table_top
axis: security
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, C]
  proof_classes: []
---

# red_team_table_top 実施

## 一文方針

月次 rotation で担当 actor class が決定した `v1_red_team_table_top` drill を、`scenario_catalog.lock.yaml` から randomly pick した build artifact シナリオに沿って机上演習し、playbook の全 step 踏破と `drill_progress.lock.yaml` への記録・postmortem PR 起票まで一連で完結させる。

## Trigger（発火条件）

月次 rotation で担当 actor class（`v1_external_unauth` / `v1_external_auth` / `v1_insider_application` / `v1_insider_operator` / `v1_supply_chain`）が決定した時。

## 想定頻度 / 典型きっかけ

想定頻度: 月次（5 actor class を 5 ヶ月で rotation）。典型きっかけ: 「今月の担当は `v1_external_auth` 。シナリオ pick は `authenticated_tenant_data_exfiltration_via_odata_filter_injection` が抽選で選ばれた」

## 主役 / 関与者

- 主役: security 担当者（シニア級、今月の担当者）
- 参加: security 担当者 全員・ops 担当者（escalation path の確認役）
- 参加（任意）: formal 担当者（proof_class bind の確認）・test 担当者（scenario corpus への追加判断）

## 前提

- `scenario_catalog.lock.yaml` が build artifact として存在し、全 threat_class を coverage 必須で管理済み
- `drill_progress.lock.yaml` が存在し、前月の `last_green_at` が記録済み
- 各 playbook が build artifact 化されており（文章 only scenario 禁止）、各 step に物理 action（API call / kubectl / OpenBao CLI 等）の pointer が存在する
- 参加者全員が `03_security設計方針/07_インシデント対応方針.md` の 6 phase を通読済み

## 流れ

1. `scenario_catalog.lock.yaml` の random pick（generator が seed-based に選択）で当月 scenario を確定し、Mattermost `#security-drill` channel に通知する
2. 担当 security 担当者が facilitator となり、演習開始を宣言。Tekton job `security-table-top-runner` を `dry_run=true` で起動し、演習ログを audit_event として emit する
3. scenario の 6 phase（detection → triage → contain → eradicate → recover → postmortem）を順に読み上げ、各 phase で「実際に取るべき物理 action」を参加者が声に出して回答する。回答が build artifact の playbook と一致するか確認する
4. detection phase: 「どの signal source が最初に alert を出すか」を 09 観測 SoR の architecture（Falco / Tetragon / audit_event / cosign verify / replication lag SLI）から特定する
5. contain phase: 担当 actor class に対応する contain action（NetworkPolicy / OpenBao revoke / Kyverno block / Harbor quarantine のいずれか）を明示し、物理 endpoint（kubectl CLI / Mattermost ChatOps slash command）を口頭で確認する
6. playbook 全 step を踏破後、success criteria（`v1_red_team_table_top` の cadence: playbook step 全踏破 + postmortem PR）を判定する
7. postmortem PR を Markdown で起票。必須 section（timeline / root cause / contributing factors / mitigation / lessons learned / action items）を記入し、action item は GitHub Issue 化 + due date を付与する
8. Tekton job が `drill_progress.lock.yaml` の `last_green_at` を自動更新する（手書き禁止）

## 関連適合仕様 / 関連 OSS

- security 強制機構: [../../../04_詳細設計/02_強制機構/06_security強制機構.md](../../../04_詳細設計/02_強制機構/06_security強制機構.md)
- 関連 OSS: Tekton / Mattermost / Argo Workflows（自製 escalation engine）

## 期待結果 / 観測指標

- `drill_progress.lock.yaml` の `v1_red_team_table_top.last_green_at` が当日日付で更新済み
- postmortem PR が起票済み（action item 全件 GitHub Issue 化済み）
- Kyverno `block-on-drill-overdue` チェックが green（cadence 30 日以内）
- 演習の audit_event が 09 観測 SoR に記録済み

## 失敗時の挙動 / escalation

- **参加者不足（security 担当者 1 名しか出席できない）**: 最低 2 名参加が条件。ops 担当者をサポート参加として追加招集する。当月 drill は延期し、cadence 違反になる前（30 日以内）に再スケジュールする
- **playbook step に物理 action pointer が欠落していた**: step を「TODO: physical action pointer missing」として postmortem action item に記録し、`scenario_catalog.lock.yaml` の修正 PR を当日中に起票する。drill 自体は playbook 改訂後に再実施
- **Tekton job が `drill_progress.lock.yaml` を更新できない（手書き制約違反）**: job の失敗 log を確認し、lock yaml generator の bug として修正 PR を起票。手書きは禁止のため、job が green になるまで `drill_progress.lock.yaml` は更新しない。Mattermost `#security` に報告（SLA: 翌営業日）

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [security 設計方針: セキュリティ訓練方針](../../../03_概要設計/07_security設計方針/08_セキュリティ訓練方針.md) — 7 drill class cadence と success criteria
- [security 設計方針: インシデント対応方針](../../../03_概要設計/07_security設計方針/07_インシデント対応方針.md) — 7 incident class × 6 phase playbook の正典
- [security 強制機構](../../../04_詳細設計/02_強制機構/06_security強制機構.md) — `block-on-drill-overdue` admission policy の詳細
