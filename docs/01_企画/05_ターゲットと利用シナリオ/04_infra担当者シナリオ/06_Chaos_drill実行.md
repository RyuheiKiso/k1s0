---
id: plan.infra.scenario_chaos_drill
axis: infra
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.infra.infra_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C, D]
  proof_classes: []
---

# Chaos drill 実行

## 一文方針

topology_class / preservation_class / clock_integrity_class 別の drill cadence に従い、[Litmus chaos experiment](../../../03_概要設計/05_infra設計方針/07_Chaos工学方針.md) を GitOps 経由で実行して SLO 維持と自動復旧を確認し、結果を lock.yaml に記録する。

> ランチ後 13 時、本社 IT 室の infra 担当者（シニア級）が Backstage の drill runbook カレンダーで「v1_global_replicated の 14 日 cadence 到来」を確認し、午後 drill 開始のタイミングで Litmus chaos experiment の準備を始める。手元には topology_drill.lock.yaml と Perses SLI ベースライン記録、Mattermost 越しに ops 担当者と dual reviewer がいる。

## ペルソナ要約

主役: infra 担当者（シニア級）、目的: Chaos drill を SLO メトリクス評価と組み合わせ、可観測な耐障害性検証を実施する

## 現状業務での痛み

- chaos drill の結果を観察するだけでメトリクス評価がなく、SLO 維持が主観的な判断になる
- chaos experiment の設定が属人的で、担当者ごとに実施する experiment が異なる
- drill 後のポストモーテムが形骸化し、SLO 改善につながる action item が出ない

## k1s0 でこう変わる

- Litmus chaos experiment が GitOps で管理され、全 drill が同一設定で実施されることが保証される
- Perses の SLO dashboard が drill 中のメトリクスを自動記録し、SLO 維持を定量的に評価できる
- drill 結果が chaos_drill.lock.yaml に記録され、改善 action item の追跡が可能になる

## Trigger（発火条件）

- drill cadence（topology_class / preservation_class / clock_integrity_class 別）の到来時

## 想定頻度 / 典型きっかけ

想定頻度: 定期（class 別 14〜180 日 / 全 class 合計で年間約 50-60 件 drill）。典型きっかけ: 「v1_global_replicated の drill cadence（14 日）が到来し、multi-region cell 分断シミュレーションを実施」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: ops 担当者（SLO 監視）、dual reviewer

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（infra）| シニア | 本社 IT 室 | Backstage drill runbook カレンダー / Perses dashboard | drill 種別選択 / SLI ベースライン記録 / Litmus apply / 自動復旧確認 |
| 関与（ops）| シニア | 本社 IT 室 / リモート | Perses dashboard（SLO パネル）| drill 中 SLO 監視 / 閾値超過時 abort alert |
| 承認（dual reviewer）| シニア | 本社 IT 室 / リモート | Mattermost `#infra-ops` | drill 結果レビュー / lock.yaml sign-off |

## 個人 KPI / 達成感

- SLO 維持率が drill ごとに計測され、耐障害性向上を定量的に確認できる
- drill 完了後の action item 消化率を追跡でき、継続的改善の達成感を得られる

## 工数 / 関与人数 / コスト感

- 工数: 半日〜1 日/drill（experiment 実行 2h + SLO 評価 1h + 結果記録 1h）
- 関与人数: 3 名（infra 担当者・ops 担当者・dual reviewer）
- コスト感: 低。Litmus が experiment を自動実行するため手動コストは評価と記録のみ

## 前提

- Litmus chaos experiment が GitOps で管理されており staging / production の drill runbook が Backstage に登録済みであること

## 流れ

1. 対象の drill 種別（Pod kill / Node drain / Network partition / Disk fill / Clock skew）を Backstage runbook から選択
2. staging cluster での pre-drill: SLI ベースライン（error rate / latency / recovery time）を Perses で記録
3. Litmus chaos experiment を GitOps 経由で apply（kubectl apply は GitOps 経路のみ許可）
4. chaos 注入中の SLO 維持を Perses で監視（閾値超過 → 自動 abort + alert）
5. chaos 注入停止後の自動復旧を観測（circuit breaker reset / Pod reschedule）
6. drill 結果（green / fail / abort）を `topology_drill.lock.yaml` または該当 lock.yaml に追記
7. dual reviewer sign-off

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | infra 担当者 | Litmus chaos experiment を apply し Perses SLO 記録を開始 | `chaos drill 開始 / SLO 記録開始 / T+0` |
| 10分 | ops 担当者 | SLO メトリクスが仕様値以内であることを Perses でリアルタイム確認 | `SLO 正常範囲 / エラー率 0.1% 以下` |
| 30分 | infra 担当者 | experiment 終了 / SLO 評価 / chaos_drill.lock.yaml に結果記録 | `drill 完了 / SLO 維持 / lock.yaml green エントリ追記` |
| 1営業日 | infra 担当者 | postmortem で action item を Backstage ticket に起票 | `postmortem 完了 / action item N 件 起票` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（infra は全業務の k8s cluster / network / storage の基盤を担うため）。特に影響度が高い 2 業務:

- **警報配信**: Chaos drill で警報配信 Pod が kill されても 5 topology_class の failover orchestrator により RTO 以内に自動復旧することを drill で確認し、製造ライン停止通知の continuity を保証する。
- **FA（設備操作）**: 設備操作指示を担う Pod の Node drain / Network partition chaos で指示欠落が発生しないことを確認し、工場自動化の安全性を chaos 工学で継続的に証明する。

## 関連適合仕様 / 関連 OSS

- クラスタ位相適合仕様: [../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md](../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md)
- 時刻整合適合仕様: [../../../04_詳細設計/01_適合仕様/13_時刻整合適合仕様.md](../../../04_詳細設計/01_適合仕様/13_時刻整合適合仕様.md)
- 関連 OSS: Litmus / Perses / Backstage / Argo CD

## 期待結果 / 観測指標

- chaos 注入中も SLO が仕様値以内に収まること
- chaos 注入停止後に自動復旧（circuit breaker reset / Pod reschedule）が完了すること
- drill 結果が `topology_drill.lock.yaml` または該当 lock.yaml に green エントリとして記録されること
- dual reviewer sign-off が記録されていること

## 失敗時の挙動 / escalation

- SLO 閾値超過 / drill abort → ops 担当者 + security 担当者に Mattermost `#chaos-drill-fail` で通報（SLA: 10 分以内）。1.0.0 ship blocker として Backstage ticket 起票
- 自動復旧未完了 → incident 登録 + infra 担当者による手動復旧 + ship blocker 判定
- GitOps 経路以外での kubectl apply 試行 → Kyverno policy で拒否 + audit log 記録

**escalate 先**: ops 担当者 + security 担当者 / Mattermost `#chaos-drill-fail`
**SLA**: SLO 閾値超過 / drill abort は 10 分以内通報
**runbook**: Backstage runbook `chaos-drill-abort` を参照。1.0.0 ship blocker として ticket 起票必須

## 失敗パターン (anti-pattern)

- 観察のみの drill: SLO 評価なしで「問題なし」と判断すると定量的な耐障害性証明にならない
- lock.yaml 未更新: drill 後に結果を記録しないと drill cadence の追跡ができなくなる

## 関連参照

- [README.md](README.md) — infra 担当者シナリオ index
- [02_topology_drill_failover.md](02_topology_drill_failover.md) — topology drill failover シナリオ
- [01_cluster_upgrade.md](01_cluster_upgrade.md) — cluster upgrade シナリオ
- [../../../03_概要設計/05_infra設計方針/README.md](../../../03_概要設計/05_infra設計方針/README.md) — infra 設計方針
