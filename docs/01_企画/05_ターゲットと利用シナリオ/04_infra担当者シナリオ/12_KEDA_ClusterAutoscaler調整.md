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

## 関連参照

- [infra 担当者シナリオ index](./README.md) — infra 担当者シナリオ全体の構成
- [node lifecycle](./09_node_lifecycle.md) — KEDA scale out が node 不足で失敗した場合のシナリオ
- [infra 設計方針 オートスケール方針](../../../03_概要設計/05_infra設計方針/06_オートスケール方針.md) — KEDA + Cluster Autoscaler の設計指針
