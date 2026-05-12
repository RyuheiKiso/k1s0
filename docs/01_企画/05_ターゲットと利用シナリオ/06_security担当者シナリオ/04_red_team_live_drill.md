---
id: plan.security.scenario_red_team_live_drill
axis: security
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, C, D, E]
  proof_classes: []
---

# red_team_live drill 実施

## 一文方針

半年 cycle で実施する `v1_red_team_live` drill を、staging cluster 上で `v1_insider_application` および `v1_insider_operator` actor class を想定した実機 red-team 演習として行い、escalation 経路の detection rate が success criteria を満たすことを `drill_progress.lock.yaml` で記録してクローズする。

## Trigger（発火条件）

半年 cycle 到来時（4 月 / 10 月の第 1 週）かつ直近の infra chaos drill（シナリオ 05）が green であることを確認した後。

## 想定頻度 / 典型きっかけ

想定頻度: 半年。典型きっかけ: 「10 月の live drill cycle。今期は insider_operator（cluster-admin 権限所有者が OpenBao super-secret に不正アクセスを試みる）シナリオを Litmus で実機演習する」

## 主役 / 関与者

- 主役: security 担当者（シニア級、red-team リード + 攻撃担当 1 名）
- 防衛側観察者: ops 担当者・infra 担当者（escalation 経路の実稼働確認）
- 参加（任意）: formal 担当者（proof_class bind が期待通りに防御として機能するか観察）

## 前提

- staging cluster が production と同等の Kyverno 25+ admission policy / Istio mTLS / Envoy `jwt_authn` / SPIRE SVID / pgaudit / audit hash chain を持つ
- `drill_progress.lock.yaml` の chaos_failure_drill `last_green_at` が 30 日以内（chaos が green でなければ live drill の信頼性が低い）
- 演習範囲の scope document（対象 cluster / 対象 namespace / 除外 PII データ）が dual reviewer sign-off 済み
- Mattermost `#security-drill` channel に live drill 開始通知済み（防衛側チームが alert を実際に受け取るかを試験するため、実施日時は防衛側には未通知）

## 流れ

1. red-team リード（security 担当者）が scope document に沿った攻撃 vector を選択する。`v1_insider_operator` の典型: RBAC drift を利用した cluster-admin 取得 → OpenBao DEV mode namespace へのアクセス試行
2. 攻撃リードが staging cluster に対し実際の kubectl コマンドを実行し始める。Falco / Tetragon / audit_event の detection が正しく発火するか時刻を記録する
3. detection phase での alert が Mattermost `#security-incident` に届くまでの時間を計測。SLA（L1 on-call からの応答: 15 分）を記録する
4. 防衛側（ops 担当者）が IR playbook に沿って contain action を取る。Kyverno admission block / OpenBao revoke / NetworkPolicy 適用が正しく動作するかを確認する
5. 攻撃リードが成功した vector と失敗した vector を記録する。成功した vector は `threat_model.lock.yaml` の対応 cell が `explicit_unreachable=true` でないか確認し、矛盾があれば即時 cell 更新 PR を起票する
6. live drill の全結果を `ir_drill.lock.yaml` に記録する（Tekton job が自動更新）。記録フィールド: `drill_class` / `actor_class` / `detection_rate` / `contain_time` / `success_criteria_met`
7. postmortem PR を起票（全 incident class の playbook との gap を記録し、action item を GitHub Issue 化）
8. `drill_progress.lock.yaml` の `v1_red_team_live.last_green_at` が Tekton job で更新されたことを確認

## 関連適合仕様 / 関連 OSS

- security 強制機構: [../../../04_詳細設計/02_強制機構/06_security強制機構.md](../../../04_詳細設計/02_強制機構/06_security強制機構.md)
- 関連 OSS: Litmus / Falco / Tetragon / Kyverno / SPIRE / OpenBao / Mattermost（自製 escalation engine）

## 期待結果 / 観測指標

- `drill_progress.lock.yaml` の `v1_red_team_live.last_green_at` が当日日付で更新済み
- detection rate が success criteria（`≥ target`、現 v1 = 100% of critical vectors detected）を満たす
- contain 時間が IR playbook の SLA 以内（contain complete ≤ 60 分）
- postmortem PR が起票済み、action item 全件 GitHub Issue 化済み

## 失敗時の挙動 / escalation

- **detection rate が target 未満（critical vector が 検知されなかった）**: 当該 vector を即シナリオ 01（threat_model レビュー）の action item として起票し、対応 cell の mitigation_class を更新する。live drill は `red` 判定、postmortem が ship blocker として `release_gate.lock.yaml` に記録される
- **live drill が staging 外に波及した（PII data に触れた等）**: 即時演習停止。data 担当者 + ops 担当者を Mattermost `#data-incident` に招集（SLA: 10 分）。staging / production の分離状態を `drill_progress.lock.yaml` に `scope_violation=true` で記録し、separation gap の修正 PR が merge されるまで次回 live drill は禁止
- **Tekton job が `drill_progress.lock.yaml` を更新できない**: シナリオ 03 の failback と同様。手書き禁止、job が green になるまで記録は保留。Mattermost `#security` に報告

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [security 設計方針: セキュリティ訓練方針](../../../03_概要設計/07_security設計方針/08_セキュリティ訓練方針.md) — `v1_red_team_live` cadence と success criteria
- [security 設計方針: インシデント対応方針](../../../03_概要設計/07_security設計方針/07_インシデント対応方針.md) — 6 phase playbook の正典
- [infra 担当者シナリオ: Chaos drill 実行](../04_infra担当者シナリオ/06_Chaos_drill実行.md) — chaos drill との前提関係
