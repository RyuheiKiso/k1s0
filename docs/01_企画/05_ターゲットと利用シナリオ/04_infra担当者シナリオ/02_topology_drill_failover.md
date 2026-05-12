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

> 朝 10 時、本社 IT 室の infra 担当者（シニア級）が topology_drill.lock.yaml で drill cadence カレンダーを確認中に、v1_zone_replicated の 90 日 cadence 到来に気付く。手元には Litmus chaos experiment 設定と Perses SLO dashboard、Mattermost 越しに ops 担当者と dual reviewer がいる。

## ペルソナ要約

主役: infra 担当者（シニア級）、目的: 5 topology_class の failover drill を定期 cadence で実施し DR 対応能力を継続的に保証する

## 現状業務での痛み

- failover 手順を本番障害まで実際に試さないため、DR 宣言時に手順の欠陥が初めて発覚する
- drill cadence が文書管理で属人化し、担当者が変わると drill が長期間未実施になる
- drill 結果の記録が残らず、前回 drill からの改善状況が追跡できない

## k1s0 でこう変わる

- topology_drill.lock.yaml が drill cadence を強制管理し、期限超過が CI で自動検知される
- Litmus chaos experiment が drill 手順を自動実行し、属人化を排除する
- drill 結果が topology_drill.lock.yaml に green / fail エントリで記録され、DR 対応能力の推移が可視化される

## Trigger（発火条件）

- drill cadence（5 topology_class の定期 drill 間隔）の到来時
- DR 演習の指示があった時

## 想定頻度 / 典型きっかけ

想定頻度: class 別 14〜180 日。典型きっかけ: 「v1_zone_replicated の drill cadence（90 日）が到来し、zone 障害シミュレーションを Litmus で実施するタイミングになった」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: ops 担当者（SLO 監視）、dual reviewer

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（infra）| シニア | 本社 IT 室 | topology_drill.lock.yaml / Perses dashboard | drill cadence 確認 / Litmus 実行 / failover 動作検証 |
| 関与（ops）| シニア | 本社 IT 室 / リモート | Perses dashboard | drill 中 SLO 監視 / SLO 閾値超過時 alert |
| 承認（dual reviewer）| シニア | 本社 IT 室 / リモート | Mattermost `#infra-ops` | drill 結果レビュー / sign-off |

## 個人 KPI / 達成感

- 5 topology_class 全 drill が green cadence 内に完了していることを lock.yaml で定量確認できる
- failover 自動切り替え成功 + SLO 維持の達成感を drill ごとに得られる

## 工数 / 関与人数 / コスト感

- 工数: 半日〜1 日/drill（Litmus experiment 設定 1h + drill 実行 2h + 結果記録 1h）
- 関与人数: 3 名（infra 担当者・ops 担当者・dual reviewer）
- コスト感: 低〜中。Litmus が experiment を自動実行するため手動コストは最小化される

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

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（infra は全業務の k8s cluster / network / storage の基盤を担うため）。特に影響度が高い 2 業務:

- **警報配信**: failover 自動切り替えの RTO 要件が最も厳しく、drill で警報配信 namespace の切り替え latency が仕様値内に収まることを重点確認する。
- **FA（設備操作）**: 工場自動化の操作指示が failover 中に欠落すると設備損傷リスクがあるため、drill で操作指示の continuity を確認する。

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

## Timeline

| T+ | actor | action | Mattermost 投稿例 |
|---|---|---|---|
| 0 | infra 担当者 | topology drill 開始（Litmus chaos experiment apply） | `@infra-oncall v1_zone_replicated drill 開始 / 対象: prod-zone-a / T+0 10:00` |
| 5 分 | infra 担当者 | failover orchestrator 自動切り替え動作を Perses で観測 | `failover 切り替え確認中 / SLO 監視継続` |
| 15 分 | ops 担当者 | SLO（可用性 / RTO / RPO）が仕様値以内であることを確認 | `SLO 正常範囲 / RTO 仕様値以内 / lag 12ms` |
| 60 分 | infra 担当者 | 切断解除 → 元 cluster への rebalance 完了を確認 | `全 pod Running / rebalance 完了 / drill green` |
| 1 営業日 | infra 担当者 | topology_drill.lock.yaml に green エントリ追記 + postmortem 着手 | `#postmortem drill-report-zone-replicated PR 作成済` |

## 失敗パターン (anti-pattern)

- drill なし本番 DR 宣言: lock.yaml の cadence 超過が CI に検知され DR gate が blocked になる
- 結果未記録: drill 後に lock.yaml を更新しないと coverage check が fail する

## 関連参照

- [README.md](README.md) — infra 担当者シナリオ index
- [06_Chaos_drill実行.md](06_Chaos_drill実行.md) — Chaos drill 実行シナリオ
- [../../../03_概要設計/05_infra設計方針/README.md](../../../03_概要設計/05_infra設計方針/README.md) — infra 設計方針
- [DR cross-region 実 failover（data-14）](../05_data担当者シナリオ/14_DR_cross_region実failover.md) — infra topology drill が green の状態が、data 担当者主導の DR 実 failover の前提となる
