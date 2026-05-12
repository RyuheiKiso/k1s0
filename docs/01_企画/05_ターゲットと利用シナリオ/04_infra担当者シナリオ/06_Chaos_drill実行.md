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

## Trigger（発火条件）

- drill cadence（topology_class / preservation_class / clock_integrity_class 別）の到来時

## 想定頻度 / 典型きっかけ

想定頻度: 定期（class 別 14〜180 日 / 全 class 合計で年間約 50-60 件 drill）。典型きっかけ: 「v1_global_replicated の drill cadence（14 日）が到来し、multi-region cell 分断シミュレーションを実施」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: ops 担当者（SLO 監視）、dual reviewer

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

## 関連参照

- [README.md](README.md) — infra 担当者シナリオ index
- [02_topology_drill_failover.md](02_topology_drill_failover.md) — topology drill failover シナリオ
- [01_cluster_upgrade.md](01_cluster_upgrade.md) — cluster upgrade シナリオ
- [../../../03_概要設計/05_infra設計方針/README.md](../../../03_概要設計/05_infra設計方針/README.md) — infra 設計方針
