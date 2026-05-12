---
id: plan.overview.scenario_ops_slo_burn_rate_alert
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

# SLO監視_burn_rate_alert

## 一文方針

MWMBR alerting による SLO burn rate alert を受信し、5 signal_class (latency / throughput / error / saturation / availability) を分類して escalation 先を判断し、初動対応を完了させる。

> 火曜午後 2 時、ops 担当者の Mattermost `#ops-slo-alert` に「警報配信パイプラインの latency P99 burn rate が 1h window で 14.4x に達した」という MWMBR alert が届く。担当者は Prometheus dashboard を開き、5 signal_class の現在値を一覧確認する。latency spiker が特定の Kafka partition に集中していることを発見し、tier1 担当者に Mattermost DM で escalation する。

## ペルソナ要約

主役: ops 担当者（シニア級）、目的: SLO burn rate alert を受信後、5 signal_class を分類して escalation 先を即時判断し初動対応を完了する

## 現状業務での痛み

- alert が多すぎて noise になっており、critical な burn rate 超過を見落とすケースがある
- 5 signal_class の分類が曖昧で、どの担当者に escalation すべきか判断に時間がかかる
- MWMBR の 1h / 6h window の意味が共有されておらず、alert の緊急度判断が属人化している
- alert 受信からエスカレーションまでのフローが文書化されていないため、担当者によって対応品質がばらつく

## k1s0 でこう変わる

- MWMBR alerting が burn rate の緊急度（fast burn / slow burn）を自動判定し、alert noise を構造的に削減する
- 5 signal_class × escalation 先の対応表が `escalation_policy.lock.yaml` に定義され、判断が即時化される
- SLO dashboard が signal_class ごとに自動整理され、root cause の特定時間が短縮される
- alert 対応フローが Backstage runbook として標準化され、担当者間の品質ばらつきがなくなる

## Trigger

SLO burn rate alert が Mattermost / pager に届いた時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 週次〜月次（alert 発生時）
- 典型きっかけ: 「Mattermost `#ops-slo-alert` に latency P99 の 1h window fast burn alert が届いた」
- 頻度根拠: MWMBR alerting の設計上、fast burn は月 0〜3 件、slow burn は月 1〜5 件程度の発火が想定される

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| ops 担当者（主役） | シニア | 本社 IT 室 / リモート | Mattermost #ops-slo-alert / Prometheus SLO dashboard | 5 signal_class 分類・escalation 先判断・初動対応 |
| tier1 担当者 | シニア | 本社 IT 室 / リモート | GitHub / Buf CI | latency/error signal の root cause 調査（escalation 受け） |
| infra 担当者 | シニア | 本社 IT 室 / リモート | Harbor / Kubernetes dashboard | saturation signal の node / resource 調査（escalation 受け） |

## 個人 KPI / 達成感

- alert 受信から escalation 先判断完了まで: 5 分以内
- false positive 率（alert 発火後に実際の影響なし）: 20% 以下
- patch SLO（P1: 1h / P2: 4h）遵守率: 95% 以上

## 工数 / 関与人数 / コスト感

- 初動対応（分類 + escalation 判断）: 5〜10 分
- root cause 特定まで（escalation 先と連携）: 30 分〜2 時間
- 関与人数: 1〜3 名（ops + escalation 先担当者）

## 前提

- Prometheus / Mimir に MWMBR alerting ルールが設定されている（1h fast burn / 6h slow burn window）
- `escalation_policy.lock.yaml` に signal_class × escalation 先の対応表が定義されている
- Backstage runbook に signal_class 別の初動手順が整備されている

## 流れ

1. **alert 受信**: Mattermost `#ops-slo-alert` または pager で MWMBR alert を受信する
2. **alert 内容確認**: signal_class（latency / throughput / error / saturation / availability）と window（1h fast burn / 6h slow burn）を確認する
3. **Prometheus dashboard 参照**: 5 signal_class の現在値を一覧確認し、burn rate の拡大速度を把握する
4. **escalation 先判断**: `escalation_policy.lock.yaml` の signal_class × escalation 先対応表を参照して escalation 先を決定する
   - latency / throughput / error: tier1 担当者
   - saturation: infra 担当者
   - availability: ops + 全軸担当者
5. **初動対応**: escalation 先に Mattermost DM で alert 内容を通知し、Backstage runbook の該当手順を共有する
6. **incident 宣言判断**: burn rate が 1h window で 14.4x を超えている場合は incident_response_7class（シナリオ 09）に移行する
7. **対応記録**: 対応内容を Mattermost `#ops-slo-alert` に記録し、`rotation_handover.lock.yaml` の next_shift_notes に追記する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0m | MWMBR alerting | Mattermost #ops-slo-alert に alert 自動投稿 | 「[SLO ALERT] 警報配信 latency P99 burn rate 14.4x (1h window)」 |
| T+1m | ops 担当者 | alert 受信・signal_class 確認 | — |
| T+3m | ops 担当者 | Prometheus dashboard で 5 signal 一覧確認 | — |
| T+5m | ops 担当者 | escalation 先判断・DM 送信 | Mattermost DM: 「tier1 担当者: latency spiker 発生、Kafka partition X を確認してください」 |
| T+10m | tier1 担当者 | root cause 調査開始 | — |
| T+30m | ops 担当者 | 対応 progress 確認・Mattermost に progress report | 「[SLO ALERT update] tier1 調査中。latency P99 が回復傾向」 |
| T+60m | ops 担当者 | alert close または incident 宣言 | 「[SLO ALERT closed] latency P99 正常範囲に復帰」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 警報配信 | 高 | availability / latency SLO は警報配信 SLA に直結 |
| FA 生産指示 | 高 | 生産指示配信の throughput / error SLO 遵守 |
| SCADA テレメトリ収集 | 中 | テレメトリ収集パイプラインの saturation / throughput |

## 関連適合仕様 / 関連 OSS

- SLO 適合仕様
- 運用ループ適合仕様
- 関連 OSS: Prometheus / Mimir（SLO alerting）/ Mattermost（通知）/ Backstage（runbook）

## 期待結果 / 観測指標

- alert 受信から escalation 先 DM 送信まで 5 分以内
- 全 alert に対して対応記録が Mattermost に投稿されている
- patch SLO 超過件数が monthly 0 件（P1）/ 2 件以下（P2）

## 失敗時の挙動 / escalation

- **escalation 先担当者が応答しない（10 分以内）**: L2 escalation が発火し tech lead に通知。SLA: 10 分以内に代替担当者 assign
- **5 signal_class の分類が判断できない**: Backstage runbook `slo-triage-guide` を参照し、不明な場合は ops チームの Mattermost グループに相談する
- **burn rate が急上昇して 14.4x を超えた**: incident_response_7class（シナリオ 09）に移行し、on-call rotation の全員に通知する

## 失敗パターン (anti-pattern)

1. **alert を確認せずに次 shift に持ち越す**: SLO error budget が枯渇し release_gate がブロックされる。ops 担当者は alert 受信後 5 分以内に確認リアクションを付ける規律を持つ
2. **全 alert を tier1 に escalation する**: noise が増え tier1 の集中度が下がる。`escalation_policy.lock.yaml` の signal_class 別対応表を必ず参照する
3. **Prometheus dashboard を開かずに escalation する**: root cause の手がかりなしに escalation するため、担当者が不要な調査時間を費やす

## 関連参照

- [ops 担当者シナリオ index](README.md)
- [incident_response_7class主導](09_incident_response_7class主導.md)
- [postmortem_PR_merge_gate](03_postmortem_PR_merge_gate.md)
