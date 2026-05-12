---
id: plan.infra.scenario_cluster_upgrade
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

# cluster upgrade

## 一文方針

k8s minor version の skew が 1 に達する前、または upgrade window（≤ 30 日）内に、staging 先行 → chaos drill → progressive delivery の手順で安全に本番 cluster を upgrade する。

## Trigger（発火条件）

- k8s minor version の skew が 1 に達した時
- upgrade window（≤ 30 日）の到来時
- Backstage calendar alert による通知

## 想定頻度 / 典型きっかけ

想定頻度: 四半期（upgrade window ≤ 30 日）。典型きっかけ: 「k8s v1.31 のサポートが k8s upstream で EOL を迎え、v1.32 へのアップグレード window 30 日に達した」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: ops 担当者（SLO 監視）、dual reviewer

## 前提

- `cluster_inventory.lock.yaml` に現行 version が記録済みであること
- 全 [5 topology_class](../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md) の drill が直近の cadence で green であること

## 流れ

1. staging cluster で upgrade を先行実施（本番より 1 version 先行でテスト）
2. OpenTofu の IaC manifest を新 minor version に更新し PR を作成
3. Argo CD の ApplicationSet で staging cluster に自動適用
4. Testcontainers integration test（本番 OSS と同 version）が staging で全 green であることを確認
5. [Litmus chaos experiment](../../../03_概要設計/05_infra設計方針/07_Chaos工学方針.md)（Chaos_drill 5 階層）を staging で実行し SLO が維持されることを確認
6. 本番 cluster への Argo CD progressive delivery（Argo Rollouts）で段階的 apply
7. upgrade 完了後 `cluster_inventory.lock.yaml` を更新し dual reviewer sign-off
8. upgrade window 30 日を超えないよう calendar alert（Backstage）で追跡

## 関連適合仕様 / 関連 OSS

- クラスタ位相適合仕様: [../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md](../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md)
- infra 強制機構: [../../../04_詳細設計/02_強制機構/04_infra強制機構.md](../../../04_詳細設計/02_強制機構/04_infra強制機構.md)
- 関連 OSS: OpenTofu / Argo CD / Argo Rollouts / Litmus / Testcontainers / Backstage

## 期待結果 / 観測指標

- 本番 cluster が新 minor version に upgrade されていること
- upgrade 中および upgrade 後に SLO が維持されていること
- `cluster_inventory.lock.yaml` が新 version で更新されていること
- dual reviewer sign-off が記録されていること

## 失敗時の挙動 / escalation

- staging でのドリル fail → 本番 upgrade 中止、原因究明後に再実施
- 本番 upgrade 途中 fail → ops 担当者に Mattermost `#infra-incident` で即時通報（SLA: 10 分以内）。Backstage runbook `k8s-upgrade-rollback` を起動。postmortem は 2 営業日以内
- upgrade window 30 日超過 → Backstage alert で ops 担当者 escalation

**escalate 先**: ops 担当者 / Mattermost `#infra-incident`
**SLA**: 本番 upgrade fail 時は 10 分以内通報、postmortem は 2 営業日以内
**runbook**: Backstage runbook `k8s-upgrade-rollback`

## 関連参照

- [README.md](README.md) — infra 担当者シナリオ index
- [06_Chaos_drill実行.md](06_Chaos_drill実行.md) — Chaos drill 実行シナリオ
- [../../../03_概要設計/05_infra設計方針/README.md](../../../03_概要設計/05_infra設計方針/README.md) — infra 設計方針
