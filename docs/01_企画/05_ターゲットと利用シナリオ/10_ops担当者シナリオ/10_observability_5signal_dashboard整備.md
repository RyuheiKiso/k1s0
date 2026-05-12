---
id: plan.overview.scenario_ops_observability_5signal_dashboard
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

# observability_5signal_dashboard整備

## 一文方針

観測 5 signal (latency / throughput / error / saturation / availability) の dashboard を整備し、新軸追加や SLO 閾値変更に合わせて Tempo / Mimir / Pyroscope / Vector / OpenTelemetry の設定を更新する。

> 月次 review の朝、ops 担当者が Grafana dashboard を開く。先月追加された data 軸 15（PII DSAR export）の SLO が dashboard に反映されていない。ops 担当者は Mimir の alerting rule と Grafana dashboard JSON を更新し、Vector の pipeline に PII DSAR export の latency / throughput metric を追加する。OpenTelemetry Collector の span sampling rate も新軸向けに調整する。

## ペルソナ要約

主役: ops 担当者（シニア級）、目的: 5 signal dashboard を新軸・SLO 変更に追従させ、observability を常に最新の状態に保つ

## 現状業務での痛み

- 新軸や新機能が追加されるたびに dashboard が手動更新されず、観測ブラインドスポットが生まれる
- Tempo / Mimir / Pyroscope / Vector / OpenTelemetry が分散していて、設定変更時の影響範囲が把握しにくい
- SLO 閾値変更が alerting rule に反映されないまま運用が続き、旧 SLO で alert が発火し続ける
- dashboard の更新が属人的で、ops 担当者交代後に整備が止まるケースがある

## k1s0 でこう変わる

- 新軸追加 / SLO 変更が Mattermost `#ops-slo-changes` に自動通知され、ops 担当者の dashboard 更新トリガが明確になる
- Grafana dashboard / Mimir alerting rule の設定が GitOps（GitHub） で管理され、変更履歴が追跡可能になる
- Vector pipeline の metric 設定が新軸に対応したテンプレートで整備され、追加工数が削減される
- OpenTelemetry Collector の設定変更が Kubernetes ConfigMap として管理され、設定ドリフトが防止される

## Trigger

新軸 SLO 追加時 / SLO 閾値変更時 / 月次 review 時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 月次
- 典型きっかけ: 「data 軸 15 が追加され、PII DSAR export の SLO が定義された」「月次 review で警報配信の latency SLO 閾値を P95 200ms → 150ms に変更した」
- 頻度根拠: 月次 review cadence に加え、新軸追加がイベント駆動で発生する

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| ops 担当者（主役） | シニア | 本社 IT 室 / リモート | Grafana dashboard / GitHub（dashboard as code） | dashboard 更新・alerting rule 更新・Vector pipeline 追加 |
| tier1 担当者 | シニア | 本社 IT 室 / リモート | GitHub PR list | OpenTelemetry instrumentation 確認（escalation 受け） |

## 個人 KPI / 達成感

- 新軸 SLO の dashboard 反映: SLO 定義から 5 営業日以内
- SLO 閾値変更の alerting rule 反映: 変更決定から 2 営業日以内
- 5 signal coverage（全 SLO 定義済み軸が dashboard で可視化されている率）: 100%

## 工数 / 関与人数 / コスト感

- 新軸 dashboard 追加: 2〜4 時間
- SLO 閾値変更: 30〜60 分
- 月次 review（全 dashboard 確認）: 1〜2 時間
- 関与人数: 1〜2 名（ops + 必要時 tier1）

## 前提

- Grafana dashboard が GitHub リポジトリで GitOps 管理されている
- Mimir の alerting rule が YAML ファイルとして管理されている
- Vector pipeline 設定が Kubernetes ConfigMap として管理されている
- OpenTelemetry Collector の設定が Kubernetes ConfigMap として管理されている

## 流れ

1. **更新トリガ確認**: 新軸 SLO 追加 / SLO 閾値変更 / 月次 review のどのトリガで発火したかを確認する
2. **対象 signal_class と SLO 確認**: 更新対象の signal_class（latency / throughput / error / saturation / availability）と SLO 値を確認する
3. **Grafana dashboard 更新**: dashboard JSON を更新し、新 SLO panel を追加する。GitHub PR を作成し、peer review を経て merge する
4. **Mimir alerting rule 更新**: 新 SLO / 変更閾値に合わせて alerting rule YAML を更新する。MWMBR の 1h / 6h window が正しく設定されているか確認する
5. **Vector pipeline 追加（新軸の場合）**: 新軸の metric を Vector pipeline に追加し、Mimir への metric 転送設定を更新する
6. **OpenTelemetry Collector 設定確認**: 新軸の trace / span が OpenTelemetry で収集されているか確認する。必要に応じて sampling rate を調整する
7. **Tempo / Pyroscope 設定確認**: trace（Tempo）と profiling（Pyroscope）の設定が新軸に対応しているか確認する
8. **動作確認**: dashboard に新 SLO panel が正常に表示され、alerting rule のテスト発火を確認する
9. **Mattermost 報告**: Mattermost `#ops-slo-changes` に更新完了を報告する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0h | ops 担当者 | 更新トリガ確認・対象 signal/SLO 確認 | — |
| T+1h | ops 担当者 | Grafana dashboard JSON 更新 PR 作成 | GitHub: PR open |
| T+2h | ops 担当者 | Mimir alerting rule 更新 PR 作成 | GitHub: PR open |
| T+3h | ops 担当者 | Vector pipeline / OTel Collector 設定更新 | — |
| T+4h | peer reviewer | dashboard / alerting rule PR review | — |
| T+5h | ops 担当者 | PR merge・動作確認 | — |
| T+5.5h | ops 担当者 | Mattermost 報告 | 「[observability] data軸15 SLO dashboard 追加完了」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 警報配信 | 高 | 警報配信の 5 signal dashboard が ops 対応の起点 |
| SCADA テレメトリ収集 | 高 | テレメトリ収集の throughput / latency dashboard |
| FA 生産指示 | 中 | 生産指示配信の error / availability dashboard |

## 関連適合仕様 / 関連 OSS

- 観測適合仕様
- 運用ループ適合仕様
- 関連 OSS: Grafana / Mimir / Tempo / Pyroscope / Vector / OpenTelemetry Collector / Mattermost

## 期待結果 / 観測指標

- 全 SLO 定義済み軸の 5 signal が Grafana dashboard で可視化されている
- Mimir alerting rule が最新 SLO 閾値で設定されている
- dashboard の GitOps PR が全 merge 済みで、設定ドリフトが 0 件

## 失敗時の挙動 / escalation

- **新軸 SLO の dashboard 反映が 5 営業日を超過**: Mattermost `#ops-slo-changes` に自動 alert。ops チームリーダーに escalation する
- **alerting rule 更新後に alert が誤発火する**: PR を revert し、alerting rule の閾値設定を再確認する。SLA: 1 時間以内に revert または修正
- **Vector pipeline 追加後に metric が Mimir に到達しない**: Vector の pipeline config と Mimir の scrape 設定を確認し、infra 担当者に escalation する

## 失敗パターン (anti-pattern)

1. **dashboard を GitHub 管理せずに Grafana UI 直接編集する**: 設定が追跡できず、誰かの変更を上書きするリスクがある
2. **新軸追加時に alerting rule の更新を忘れる**: dashboard は見えるが、alert が発火しない「サイレント障害」状態になる
3. **月次 review で全 dashboard を確認せずに「問題なし」と報告する**: 陳腐化した dashboard が放置され、incident 対応時に観測ブラインドスポットが露呈する

## 関連参照

- [ops 担当者シナリオ index](README.md)
- [SLO監視_burn_rate_alert](02_SLO監視_burn_rate_alert.md)
- [capacity_planning_KEDA調整](11_capacity_planning_KEDA調整.md)
