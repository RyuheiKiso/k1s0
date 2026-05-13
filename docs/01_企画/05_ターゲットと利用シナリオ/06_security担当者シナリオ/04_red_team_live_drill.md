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

> 朝 9 時、security 担当者（シニア級、red-team リード）が staging cluster にアクセスし、live drill の scope document を確認する。防衛側の ops 担当者・infra 担当者は実施日時を知らされておらず、Mattermost `#security-drill` には「本日中に live drill を実施する」との事前通知のみが届いている。攻撃リードは `v1_insider_operator` シナリオ（cluster-admin 権限所有者が OpenBao super-secret に不正アクセスを試みる）を選択し、kubectl コマンドを手元に展開している。

## ペルソナ要約

主役: security 担当者（シニア級）、目的: red team live drill の結果を共有し再現 attack を防ぐための防御強化を実施する

## 現状業務での痛み

- live drill 結果が担当者間で共有されず、同一の attack vector が繰り返し成功する
- drill 後の防御強化 action が追跡されず、脆弱性が長期間放置される
- drill 結果の機密性管理が不明確で、結果の取り扱いが属人的になる

## k1s0 でこう変わる

- drill 結果が red_team.lock.yaml に記録され、attack vector と防御強化 action が全員で共有される
- 防御強化 action が Backstage ticket で追跡され、脆弱性対処の完了が定量的に確認される
- drill 結果の機密性分類が lock.yaml に定義され、取り扱いが標準化される

## Trigger（発火条件）

半年 cycle 到来時（4 月 / 10 月の第 1 週）かつ直近の infra chaos drill（シナリオ 05）が green であることを確認した後。

## 想定頻度 / 典型きっかけ

想定頻度: 半年。典型きっかけ: 「10 月の live drill cycle。今期は insider_operator（cluster-admin 権限所有者が OpenBao super-secret に不正アクセスを試みる）シナリオを Litmus で実機演習する」

## 主役 / 関与者

- 主役: security 担当者（シニア級、red-team リード + 攻撃担当 1 名）
- 防衛側観察者: ops 担当者・infra 担当者（escalation 経路の実稼働確認）
- 参加（任意）: formal 担当者（proof_class bind が期待通りに防御として機能するか観察）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（security、攻撃側）| シニア | staging cluster ターミナル | scope document / staging kubectl context | 実機 attack vector の実行・detection rate 計測・結果記録 |
| 防衛側（ops）| シニア〜ミドル | 本社 IT 室 / リモート | Mattermost #security-incident / Prometheus | IR playbook に沿った contain action・SLO budget 監視 |
| 防衛側（infra）| シニア〜ミドル | 本社 IT 室 / リモート | Harbor dashboard / Kyverno audit log | quarantine 操作・NetworkPolicy 適用確認 |

## 個人 KPI / 達成感

- 防御強化 action の消化率を ticket で定量確認でき、セキュリティ改善の達成感を得られる
- 同一 attack vector の再発 0 件を追跡でき、防御強化の効果を数値で確認できる

## 工数 / 関与人数 / コスト感

- 工数: 1〜2 日（drill 準備 2h + drill 実施 4h + 結果共有 2h + action 起票 2h）
- 関与人数: 4〜6 名（security 担当者・red team・tier1〜tier3 担当者）
- コスト感: 中〜高。drill 自体は高コストだが防御強化の継続的効果により投資対効果が高い

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

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | security 担当者 | red_team.lock.yaml で drill シナリオを確認し live drill を開始 | `live drill 開始 / シナリオ確認 / T+0` |
| 4h | security 担当者 + red team | live drill を実施し attack vector と侵入経路を記録 | `drill 完了 / attack vector N 件特定` |
| 1d | security 担当者 | drill 結果を lock.yaml に記録し防御強化 action を起票 | `結果共有完了 / action item N 件起票` |
| 1d+2h | dual reviewer | drill 結果と action item を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

live drill は実機演習であり、全 9 業務を対象とする staging cluster の防御力を直接測定する。特に影響度が高い業務:

- **品質検査結果配信**: PII / business_data としての検査データが exfiltration シナリオの典型的な target であり、`v1_insider_application` actor による RLS bypass を live で試行・検知できるかを測定する。
- **図面 collaborative review**: 設計図面は IP / business_data として保護対象であり、`v1_insider_operator` actor による不正アクセスが live で detection されるかを確認する最重要シナリオ。
- **在庫最新値**: insider threat による在庫データの exfiltration は business_data / insider threat の典型的な attack vector であり、live drill での detection rate 確認が必須。

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

## 失敗パターン (anti-pattern)

- drill 結果の非共有: 結果を担当者のみが知る状態は組織的な防御強化につながらない
- action item なしの drill 完了: 防御強化 action が 0 件の drill は CI が形骸化として検知する

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [security 設計方針: セキュリティ訓練方針](../../../03_概要設計/07_security設計方針/08_セキュリティ訓練方針.md) — `v1_red_team_live` cadence と success criteria
- [security 設計方針: インシデント対応方針](../../../03_概要設計/07_security設計方針/07_インシデント対応方針.md) — 6 phase playbook の正典
- [infra 担当者シナリオ: Chaos drill 実行](../04_infra担当者シナリオ/06_Chaos_drill実行.md) — chaos drill との前提関係
