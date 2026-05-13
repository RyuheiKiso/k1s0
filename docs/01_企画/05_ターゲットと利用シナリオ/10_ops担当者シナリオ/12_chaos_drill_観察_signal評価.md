---
id: plan.overview.scenario_ops_chaos_drill_signal_evaluation
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

# chaos_drill_観察_signal評価

## 一文方針

infra 軸 / security 軸の chaos drill を ops 担当者として観察し、SLO error budget 消費を評価して drill 結果の postmortem を起票する。

> 毎月第 3 水曜、ops 担当者が infra 担当者と security 担当者の chaos drill に observer として参加する。Grafana の SLO dashboard を開き、drill が始まると同時に 5 signal の変動を記録する。Chaos Mesh が警報配信 pod 1 台を強制停止すると、latency P99 が 120ms から 280ms に上昇する。error budget の消費速度を確認しながら、ops 担当者は drill 観察ログを記録し、drill 終了後に postmortem の起票を行う。

## ペルソナ要約

主役: ops 担当者（シニア級）、目的: chaos drill の観察中に SLO error budget 消費を評価し、drill 結果の postmortem を起票する

## 現状業務での痛み

- chaos drill の観察役が存在せず、drill 中の SLO 影響が記録されないまま終了する
- drill によって error budget を意図せず消費してしまっても、消費量が把握されていない
- drill 結果の postmortem が作成されないため、drill から得た学びが次の改善につながらない
- drill の停止判断基準（error budget 消費上限）が定義されておらず、drill が SLO に深刻な影響を与えるリスクがある

## k1s0 でこう変わる

- ops 担当者が chaos drill の公式 observer として参加し、5 signal の変動をリアルタイムで記録する
- drill 停止判断基準（error budget 消費上限）が事前定義され、ops 担当者が判断権を持つ
- drill 終了後の postmortem が ops 担当者によって必ず起票され、学びが runbook 更新（シナリオ 04）につながる
- error budget 消費量が `chaos_drill_report.lock.yaml` に記録され、月次 SLO review で参照される

## Trigger

chaos drill 実施日程到来時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 月次
- 典型きっかけ: 「毎月第 3 水曜の chaos drill スケジュールが到来した」「infra 担当者から chaos drill 実施の連絡が届いた」
- 頻度根拠: 月次 chaos drill が infra / security 軸の定例となっているため

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| ops 担当者（主役、observer） | シニア | 本社 IT 室 / リモート | Grafana SLO dashboard / chaos_drill_report.lock.yaml | 5 signal 観察・error budget 消費評価・drill 停止判断・postmortem 起票 |
| infra 担当者 | シニア | 本社 IT 室 / リモート | Chaos Mesh dashboard | chaos drill 実施・復旧操作 |
| security 担当者 | シニア | 本社 IT 室 / リモート | Chaos Mesh / security dashboard | security chaos drill 実施 |

## 個人 KPI / 達成感

- chaos drill 観察参加率: 100%（月次）
- drill 中の SLO error budget 消費量記録率: 100%
- drill 結果 postmortem 起票率: 100%（drill 終了後 24 時間以内）

## 工数 / 関与人数 / コスト感

- drill 観察（立会）: 1〜2 時間
- postmortem 起票: 30〜60 分
- 関与人数: 3〜5 名（ops + infra + security）

## 前提

- chaos drill スケジュールが Mattermost `#ops-chaos-drill` に事前通知されている
- drill 停止判断基準（error budget 消費上限: 5%）が事前に合意されている
- `chaos_drill_report.lock.yaml` が build artifact として存在する

## 流れ

1. **drill 事前確認**: drill 前に対象コンポーネント・chaos 種別（pod kill / network partition / latency injection 等）と停止判断基準を確認する
2. **SLO dashboard 準備**: Grafana で対象コンポーネントの 5 signal dashboard を開き、drill 前の baseline 値を記録する
3. **drill 観察開始**: infra / security 担当者が chaos drill を開始すると同時に、5 signal の変動を記録する
4. **error budget 消費監視**: drill 中の SLO error budget 消費速度を監視する。消費上限（5%）に達した場合は drill 停止を要求する
5. **drill 停止判断**: error budget 消費が上限に達するか、サービス復旧が観察できない場合は infra / security 担当者に drill 停止を要求する
6. **復旧確認**: drill 終了後、5 signal が baseline 水準に復帰したことを確認する
7. **`chaos_drill_report.lock.yaml` 記録**: drill 種別・5 signal 変動・error budget 消費量・復旧時間を YAML に記録する
8. **postmortem 起票**: drill 結果の postmortem を GitHub PR として起票し、改善 action item を登録する（シナリオ 03 の手順に準拠）

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T-15m | ops 担当者 | SLO dashboard 準備・baseline 値記録 | — |
| T+0m | infra 担当者 | chaos drill 開始（pod kill 実行） | Mattermost: 「[chaos-drill] 警報配信 pod kill 開始」 |
| T+1m | ops 担当者 | 5 signal 変動観察開始・記録 | — |
| T+5m | ops 担当者 | error budget 消費量確認（5% 以内か） | — |
| T+15m | infra 担当者 | chaos drill 終了・復旧操作 | Mattermost: 「[chaos-drill] drill 終了・復旧操作開始」 |
| T+20m | ops 担当者 | 5 signal baseline 復帰確認 | — |
| T+25m | ops 担当者 | chaos_drill_report.lock.yaml 記録 | — |
| T+30m | ops 担当者 | postmortem GitHub PR 起票 | GitHub: PR open |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 警報配信 | 高 | 警報配信の chaos drill は availability SLO の耐障害性検証 |
| SCADA テレメトリ収集 | 高 | テレメトリ収集の network partition drill |
| FA 生産指示 | 中 | 生産指示配信の pod kill drill |

## 関連適合仕様 / 関連 OSS

- 運用ループ適合仕様
- 検証規律適合仕様
- 関連 OSS: Chaos Mesh（chaos drill）/ Grafana（SLO dashboard）/ GitHub（postmortem PR）

## 期待結果 / 観測指標

- drill 中の 5 signal 変動が `chaos_drill_report.lock.yaml` に記録されている
- error budget 消費量が事前合意の上限（5%）以内
- postmortem GitHub PR が drill 終了後 24 時間以内に起票されている

## 失敗時の挙動 / escalation

- **error budget 消費が上限（5%）に達した**: ops 担当者が即座に drill 停止を要求する。infra 担当者が chaos experiment を停止し、サービス復旧操作を開始する
- **drill 後に 5 signal が baseline に復帰しない**: ops 担当者が incident_response_7class（シナリオ 09）を発火し、availability_loss として対応する
- **`chaos_drill_report.lock.yaml` の記録が漏れる**: drill 参加記録なしと見なされ、次月の chaos drill 前に再記録が要求される

## 失敗パターン (anti-pattern)

1. **drill 中に error budget 消費を監視せずに観察する**: 想定以上の error budget 消費が発生しても気づかず、本番 SLO に深刻な影響を与える
2. **drill 結果 postmortem を省略する**: drill から得た学び（resilience 改善 action item）が消え、毎月同じ問題が再現される
3. **ops 担当者が observer 役を「形式参加」と捉える**: drill 停止判断権を持つ ops の役割は重要。passive な観察ではなく、積極的な SLO 保護の立場で参加する

## 関連参照

- [ops 担当者シナリオ index](README.md)
- [postmortem_PR_merge_gate](03_postmortem_PR_merge_gate.md)
- [runbook整備_Backstage_TechDocs](04_runbook整備_Backstage_TechDocs.md)
