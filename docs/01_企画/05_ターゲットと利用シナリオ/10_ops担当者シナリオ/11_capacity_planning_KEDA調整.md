---
id: plan.overview.scenario_ops_capacity_planning_keda
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

# capacity_planning_KEDA調整

## 一文方針

KEDA / ClusterAutoscaler の月次 capacity planning で signal_class 別の予測モデルを更新し、次月の peak load に向けた scaling 設定を調整して release_gate に記録する。

> 毎月最終週、ops 担当者が ClickHouse の月次 throughput / saturation レポートを開く。先月の SCADA テレメトリ収集が月末に throughput spike を記録し、KEDA の ScaledObject がタイムリーにスケールアウトできなかったことが分かる。ops 担当者は KEDA の scaleTargetRef と cooldownPeriod を調整し、ClusterAutoscaler の node group 設定も peak load 想定に合わせて更新する。変更内容を `capacity_plan.lock.yaml` に記録し、release_gate に反映する。

## ペルソナ要約

主役: ops 担当者（シニア級）、目的: 月次 capacity planning で KEDA / ClusterAutoscaler を peak load に備えて調整し、release_gate に記録する

## 現状業務での痛み

- capacity planning が事後対応（saturation 発生後に設定変更）になっており、peak load での degradation を防げない
- KEDA と ClusterAutoscaler の設定が独立しており、調整時の影響範囲が把握しにくい
- signal_class 別の予測モデルが存在せず、感覚的な scaling 設定になっている
- capacity 変更の記録がなく、設定変更後に何が変わったかを追跡できない

## k1s0 でこう変わる

- ClickHouse の月次レポートが signal_class 別の throughput / saturation 予測を提供し、データドリブンな capacity planning が可能になる
- `capacity_plan.lock.yaml` が capacity 変更の構造化記録として機能し、設定変更の追跡が可能になる
- KEDA ScaledObject と ClusterAutoscaler の設定が GitOps で管理され、変更履歴が残る
- release_gate との連動で、capacity planning 完了が release の prerequisite として担保される

## Trigger

月次 capacity review 日程到来時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 月次
- 典型きっかけ: 「月末の throughput spike で KEDA のスケールアウトが追いつかなかった」「次月に製造ライン増設で SCADA テレメトリ量が 30% 増加する予定」
- 頻度根拠: capacity planning は月次 cadence が SRE best practice。大幅な負荷変動が予測される場合はスポットで実施

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| ops 担当者（主役） | シニア | 本社 IT 室 / リモート | ClickHouse レポート / Grafana dashboard | 予測モデル更新・KEDA / ClusterAutoscaler 設定調整・記録 |
| infra 担当者 | シニア | 本社 IT 室 / リモート | Kubernetes dashboard | node group 設定変更（escalation 受け）|

## 個人 KPI / 達成感

- 月次 capacity review 実施率: 100%
- peak load 時の saturation signal 超過件数: 月次 0 件
- `capacity_plan.lock.yaml` の月次記録率: 100%

## 工数 / 関与人数 / コスト感

- 月次 capacity planning: 1〜2 時間
- KEDA / ClusterAutoscaler 設定変更: 30〜60 分
- 関与人数: 1〜2 名（ops + 必要時 infra）

## 前提

- ClickHouse に月次 throughput / saturation レポートが自動生成される設定がある
- KEDA ScaledObject が GitOps（GitHub）で管理されている
- ClusterAutoscaler の node group 設定が Kubernetes YAML として管理されている
- `capacity_plan.lock.yaml` が build artifact として存在する

## 流れ

1. **月次レポート確認**: ClickHouse の月次 throughput / saturation レポートを開き、signal_class 別の peak 値と平均値を確認する
2. **予測モデル更新**: 前月実績と次月予測（製造計画 / 季節変動 / テナント増加）に基づき、signal_class 別の予測モデルを更新する
3. **KEDA ScaledObject 調整**: throughput / saturation の予測に基づき、scaleTargetRef / minReplicaCount / maxReplicaCount / cooldownPeriod を調整する。GitHub PR を作成する
4. **ClusterAutoscaler node group 設定調整**: peak load 想定に合わせて node group の min / max nodes を調整する。infra 担当者と協力して Kubernetes YAML を更新する
5. **`capacity_plan.lock.yaml` 記録**: 変更内容・予測根拠・次月 peak load 想定を YAML に記録し、GitHub commit で証跡を残す
6. **release_gate 更新**: `release_gate.lock.yaml` の capacity_planning_completed フラグを green に更新する
7. **Mattermost 報告**: Mattermost `#ops-capacity` に月次 capacity planning 完了を報告する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| T+0h | ops 担当者 | ClickHouse 月次レポート確認・予測モデル更新 | — |
| T+1h | ops 担当者 | KEDA ScaledObject 調整 PR 作成 | GitHub: PR open |
| T+1.5h | ops 担当者 | infra 担当者に node group 調整を依頼 | Mattermost DM: 「次月 peak load 向け node group 設定変更依頼」 |
| T+2h | infra 担当者 | ClusterAutoscaler 設定変更 PR 作成 | — |
| T+3h | ops 担当者 | KEDA PR merge・capacity_plan.lock.yaml 記録 | — |
| T+3.5h | ops 担当者 | release_gate 更新・Mattermost 報告 | 「[capacity] 月次 capacity planning 完了。KEDA / ClusterAutoscaler 調整済み」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| SCADA テレメトリ収集 | 高 | テレメトリ収集の throughput spike に向けた KEDA 調整 |
| FA 生産指示 | 高 | 生産計画増加時の capacity 増強 |
| 警報配信 | 中 | 警報配信 throughput の seasonal 変動対応 |

## 関連適合仕様 / 関連 OSS

- テナント容量適合仕様
- クラスタ位相適合仕様
- 関連 OSS: KEDA（event-driven autoscaling）/ ClusterAutoscaler / ClickHouse（月次レポート）/ GitHub（GitOps）

## 期待結果 / 観測指標

- `capacity_plan.lock.yaml` に月次変更記録が存在する
- KEDA ScaledObject が peak load 想定に合わせて更新されている
- 翌月の peak load 期間に saturation signal 超過が発生していない

## 失敗時の挙動 / escalation

- **KEDA ScaledObject の調整後に pod が expected count に到達しない**: Kubernetes events を確認し、infra 担当者に pod scheduling 問題を escalation する
- **ClusterAutoscaler の node group 設定変更が cloud provider 制約で拒否される**: infra 担当者が cloud console で quota を確認し、quota 増加申請を行う。SLA: 3 営業日以内に quota 解決
- **予測モデルが実際の peak load と大幅に乖離する**: 予測精度改善を自動化 backlog（シナリオ 08）に登録し、次月の capacity planning で予測手法を見直す

## 失敗パターン (anti-pattern)

1. **capacity planning を saturation alert 発生後に実施する**: 事後対応では peak load 中の degradation を防げない。月次 cadence での事前計画が必須
2. **KEDA のみ調整して ClusterAutoscaler を放置する**: pod 数は増加するが node が不足してスケジューリングが詰まるリスクがある
3. **変更を直接 kubectl apply し GitHub に記録しない**: 設定変更の追跡が不可能になり、次月の capacity planning で変更内容が分からなくなる

## 関連参照

- [ops 担当者シナリオ index](README.md)
- [observability_5signal_dashboard整備](10_observability_5signal_dashboard整備.md)
- [release_切り_dual_sign_off](13_release_切り_dual_sign_off.md)
