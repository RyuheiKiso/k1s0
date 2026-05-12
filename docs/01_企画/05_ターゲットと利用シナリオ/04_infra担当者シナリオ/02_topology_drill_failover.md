---
id: plan.infra.scenario_topology_drill_failover
axis: infra
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.infra.infra_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes: []
---

# topology drill failover

## 一文方針

[5 topology_class](../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md) すべての failover drill を定期 cadence で実施し、failover orchestrator の自動切り替えと SLO 維持を確認することで、DR 対応能力を継続的に保証する。

## Trigger（発火条件）

- drill cadence（5 topology_class の定期 drill 間隔）の到来時
- DR 演習の指示があった時

## 想定頻度 / 典型きっかけ

想定頻度: class 別 14〜180 日。典型きっかけ: 「v1_zone_replicated の drill cadence（90 日）が到来し、zone 障害シミュレーションを Litmus で実施するタイミングになった」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: ops 担当者（SLO 監視）、dual reviewer

## 前提

- 5 topology_class（v1_local_only / v1_zone_replicated / v1_cluster_replicated / v1_cross_region_replicated / v1_global_replicated）の drill cadence が `topology_drill.lock.yaml` に記録済みであること
- failover orchestrator（5 種）が `topology_drill.lock.yaml` で管理されていること

## 流れ

1. 対象 topology_class と failover orchestrator（5 種）を `topology_drill.lock.yaml` で確認
2. [Litmus chaos experiment](../../../03_概要設計/05_infra設計方針/07_Chaos工学方針.md) で対象 zone / cluster / region を模擬的に切断
3. failover orchestrator が自動で代替 cluster / zone に切り替わることを確認
4. SLO（可用性 / RTO / RPO）が drill 中も仕様値以内に収まることを Perses で観測
5. 切断解除後に元 cluster への自動 rebalance を確認
6. drill 結果を `topology_drill.lock.yaml` に green エントリとして追記
7. dual reviewer sign-off

## 関連適合仕様 / 関連 OSS

- クラスタ位相適合仕様: [../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md](../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md)
- 関連 OSS: Litmus（chaos experiment）/ Argo Rollouts（progressive failover）/ Perses（SLO 観測）

## 期待結果 / 観測指標

- failover orchestrator が自動切り替えを成功させること
- SLO（可用性 / RTO / RPO）が drill 中も仕様値以内に収まること
- 切断解除後に元 cluster への自動 rebalance が完了すること
- `topology_drill.lock.yaml` に green エントリが追記されていること

## 失敗時の挙動 / escalation

- failover 自動切り替え失敗 → drill abort（Backstage ticket `chaos-drill-abort-<date>` を起票）+ **postmortem 期限: 2 営業日以内**（Backstage runbook `drill-postmortem-template` を使用）+ 1.0.0 ship blocker
- SLO 閾値超過 → drill abort + ops 担当者に自動 alert（SLA: 5 分以内）。drill 結果は fail として `topology_drill.lock.yaml` に記録、次 drill までに改善計画提出
- rebalance 未完了 → incident 登録 + infra 担当者による手動復旧手順実施

**escalate 先**: ops 担当者 / 自動 alert
**SLA**: SLO 閾値超過時は 5 分以内 alert
**runbook**: Backstage runbook 参照（topology drill 対象 class に対応する runbook）

## 関連参照

- [README.md](README.md) — infra 担当者シナリオ index
- [06_Chaos_drill実行.md](06_Chaos_drill実行.md) — Chaos drill 実行シナリオ
- [../../../03_概要設計/05_infra設計方針/README.md](../../../03_概要設計/05_infra設計方針/README.md) — infra 設計方針
