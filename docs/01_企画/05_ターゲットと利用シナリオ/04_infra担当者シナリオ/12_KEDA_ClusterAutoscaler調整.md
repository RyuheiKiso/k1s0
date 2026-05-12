---
id: plan.infra.scenario_keda_autoscaler_tuning
axis: infra
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.infra.infra_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [D]
  proof_classes: []
---

# KEDA / ClusterAutoscaler 調整

## 一文方針

infra 担当者が KEDA の scaling 指標（Kafka lag / CPU / memory / custom metrics）と Cluster Autoscaler の node 追加閾値を業務 SLO に合わせてチューニングし、過剰 scaling と scaling 遅延のバランスを最適化する。

> 朝 9 時、本社 IT 室の infra 担当者（シニア級）が Perses dashboard の `keda-scaling-time` パネルを確認中に、FA 生産指示 Kafka topic の lag が昨日のピーク時に `lagThreshold` を超えても scale out に 3 分かかっていた事象を発見する。手元には KEDA の ScaledObject YAML と過去 30 日の Kafka lag グラフ、Mattermost 越しに ops 担当者・tier2 担当者・dual reviewer がいる。

## ペルソナ要約

主役: infra 担当者（シニア級）、目的: KEDA / Cluster Autoscaler の scaling 設定を最適化し surge 時のスケール遅延を解消する

## 現状業務での痛み

- KEDA scaling 設定が属人的で、surge 時にスケール遅延が発生してもパラメータ調整の根拠がない
- Cluster Autoscaler のノード追加ラグが予測できず、surge 時に処理遅延やタイムアウトが発生する
- scaling 設定の変更履歴が残らず、問題発生時にどのパラメータが原因かの特定が困難

## k1s0 でこう変わる

- KEDA ScaledObject が GitOps で管理され、scaling パラメータの変更が PR レビューで審査される
- Perses の scaling メトリクスが lag / queue depth の推移を可視化し、パラメータ調整の根拠が定量化される
- keda.lock.yaml が scaling 設定の変更履歴を記録し、問題発生時のパラメータ追跡が即時に可能になる

## Trigger（発火条件）

- SLO 違反（Scaling 遅延で response time 超過）
- 過剰 scaling によるコスト増大
- 新しい scaling 指標（Kafka lag 等）の追加が必要になった時

## 想定頻度 / 典型きっかけ

想定頻度: 四半期〜年次。典型きっかけ: 「FA 生産指示の Kafka topic の lag が増大し、consumer Pod が scale out されるまでに SLO が違反される事象が頻発した。KEDA の `lagThreshold` を調整する必要が生じた」「tier2 の Scheduler バッチ増加で月末の node 需要が急増し、Cluster Autoscaler の `scale-down-delay-after-add` で scale in が早すぎる問題が発生した」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: ops 担当者（SLO 定義確認）/ tier2 担当者（Kafka consumer の SLO 確認）
- 承認: dual reviewer（infra 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（infra）| シニア | 本社 IT 室 | Perses dashboard（keda-scaling-time / kafka-consumer-lag）| lag 推移分析 / lagThreshold 再設定 / staging chaos test / 本番 GitOps 適用 |
| 関与（ops）| シニア | 本社 IT 室 / リモート | Perses dashboard（slo-compliance-rate）| SLO 定義確認 / 調整後 1 週間の SLO 監視 |
| 関与（tier2）| ミドル | 本社 IT 室 / リモート | Kafka dashboard | Kafka consumer の SLO 確認 / lagThreshold 調整値の妥当性確認 |
| 承認（dual reviewer）| シニア | 本社 IT 室 / リモート | Mattermost `#infra-ops` | ScaledObject YAML PR レビュー / cluster_inventory.lock.yaml sign-off |

## 個人 KPI / 達成感

- surge 時の scaling 完了時間が SLO 以内であることを Perses で定量確認でき、設定最適化の達成感を得られる
- scaling 設定 PR の品質向上を数値で確認でき、チームの scaling 知識が定量化される

## 工数 / 関与人数 / コスト感

- 工数: 1〜2 日（KEDA GitOps 移行 4h + メトリクス設定 2h + パラメータ調整 4h）
- 関与人数: 2〜3 名（infra 担当者・ops 担当者・dual reviewer）
- コスト感: 低〜中。GitOps 移行後は継続コストがパラメータ調整のみになる

## 前提

- [オートスケール方針](../../../03_概要設計/05_infra設計方針/06_オートスケール方針.md)（KEDA + Cluster Autoscaler）が確立済み
- `slo_catalog.lock.yaml` に調整対象 Service の SLO が記録済み

## 流れ

1. Perses の SLO ダッシュボードで scaling 遅延 / 過剰 scaling の発生状況を確認する（過去 30 日の data を分析）
2. 調整対象の ScaledObject / HPA / Cluster Autoscaler 設定を確認する（`kubectl get scaledobject -A`）
3. 調整の方針を決定する:
   - KEDA Kafka scaler: `lagThreshold` 値を Perses の `kafka-consumer-lag` 推移から適切な値に再設定
   - CPU / memory scaler: `targetAverageUtilization` を SLO のバッファ（SLO 目標値の 80%）に設定
   - Cluster Autoscaler: `scale-down-delay-after-add` / `scale-down-unneeded-time` を月末需要パターンに合わせて調整
4. staging cluster で調整値をテストする（Litmus chaos で負荷をかけて scaling 動作を確認）
5. 調整前後の scaling 速度と SLO 違反率を Perses で比較する
6. 本番 cluster に適用（GitOps 経由 / `infra/keda/` 配下の ScaledObject YAML を更新）
7. 本番適用後 1 週間 Perses で scaling 指標を監視する
8. `cluster_inventory.lock.yaml` に調整内容を記録し dual reviewer sign-off を取得する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | infra 担当者 | 既存 KEDA ScaledObject を GitOps リポジトリに移行し Perses メトリクスを設定 | `KEDA GitOps 移行完了 / メトリクス設定` |
| 4h | infra 担当者 | surge テストで scaling 遅延を計測し minReplicaCount / cooldown を調整 | `surge テスト完了 / scaling 遅延 N 秒 / パラメータ調整` |
| 1d | infra 担当者 | 調整後の scaling テスト green を確認し keda.lock.yaml を更新して PR 提出 | `scaling テスト green / lock.yaml 更新 / PR #NNN` |
| 1d+2h | dual reviewer + ops 担当者 | scaling 設定と lock.yaml を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（infra は全業務の k8s cluster / network / storage の基盤を担うため）。特に影響度が高い 2 業務:

- **受注（ピーク）**: 受注業務は月末・キャンペーン時に API リクエストが急増するため、Cluster Autoscaler の `scale-down-delay-after-add` を受注ピークパターンに合わせて調整し、scale in が早すぎてレイテンシ劣化が発生しないようにする。
- **SCADA（burst）**: センサーデータの定期バーストで Kafka lag が急増する SCADA consumer の `lagThreshold` を、burst 周期の実測値から適切に再設定して scale out 遅延による計測欠損を防ぐ。

## 関連適合仕様 / 関連 OSS

- クラスタ位相適合仕様: [../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md](../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md)
- SLO 適合仕様: [../../../04_詳細設計/01_適合仕様/07_SLO適合仕様.md](../../../04_詳細設計/01_適合仕様/07_SLO適合仕様.md)
- 関連 OSS: KEDA（event-driven autoscaler）/ Cluster Autoscaler / Litmus（chaos test）/ Perses（SLO monitoring）

## 期待結果 / 観測指標

- slo: SLO 違反率が調整後 1 週間で目標値以下（Perses `slo-compliance-rate` で確認）
- scaling: Kafka lag が `lagThreshold` 以上になると 60 秒以内に scale out 開始（Perses `keda-scaling-time` で確認）
- cost: 過剰 scaling が減少（node 利用率が調整前比 10% 以上改善）
- artifact: `cluster_inventory.lock.yaml` に調整内容記録済み
- sign-off: dual reviewer（infra 担当者 2 名）sign-off 完了

## 失敗時の挙動 / escalation

- **調整後に SLO 違反が増加**: 元の設定に rollback し、ops 担当者に Mattermost `#infra-incident` で報告（**SLA: 違反検出後 30 分以内に rollback**）。調整前の Perses data を再分析して次回調整計画を立て直す（**SLA: 5 営業日以内**）。
- **staging の chaos test で期待通り scale out しない**: staging で問題を再現させ原因調査してから本番に適用しない（staging fail = 本番適用中止）。

## 失敗パターン (anti-pattern)

- scaling パラメータの勘頼り調整: メトリクスなしの感覚的なパラメータ変更は設定 drift を引き起こす
- GitOps 外の kubectl patch: ScaledObject を直接 patch すると drift detection が検知する

## 関連参照

- [infra 担当者シナリオ index](./README.md) — infra 担当者シナリオ全体の構成
- [node lifecycle](./09_node_lifecycle.md) — KEDA scale out が node 不足で失敗した場合のシナリオ
- [infra 設計方針 オートスケール方針](../../../03_概要設計/05_infra設計方針/06_オートスケール方針.md) — KEDA + Cluster Autoscaler の設計指針
