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

> 朝 9 時、本社 IT 室の infra 担当者（シニア級）が Argo CD UI で ApplicationSet の sync 状態を確認中に、Backstage calendar alert「k8s v1.31 upgrade window 残り 3 日」に気付く。手元には cluster_inventory.lock.yaml と OpenTofu plan 出力、Mattermost 越しに ops 担当者と dual reviewer がいる。

## ペルソナ要約

主役: infra 担当者（シニア級）、目的: k8s minor version upgrade を staging 先行・chaos drill・progressive delivery の安全手順で完結させる

## 現状業務での痛み

- cluster アップグレード手順が Confluence / Word 文書で管理されており、手順の陳腐化が本番障害まで気付かれない
- 担当者が変わると手順の解釈に個人差が生じ、upgrade の実施品質が属人化する
- staging での検証が不十分なまま本番 upgrade を実施し、非互換変更が本番で初めて発覚する

## k1s0 でこう変わる

- upgrade 手順が GitOps の IaC として管理され、OpenTofu plan で変更内容が事前に可視化される
- Backstage TechDocs の runbook が upgrade 手順の SoT となり、誰が実施しても同一品質が保証される
- Litmus chaos drill が staging で必須化され、非互換変更が本番 upgrade 前に検知される

## Trigger（発火条件）

- k8s minor version の skew が 1 に達した時
- upgrade window（≤ 30 日）の到来時
- Backstage calendar alert による通知

## 想定頻度 / 典型きっかけ

想定頻度: 四半期（upgrade window ≤ 30 日）。典型きっかけ: 「k8s v1.31 のサポートが k8s upstream で EOL を迎え、v1.32 へのアップグレード window 30 日に達した」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: ops 担当者（SLO 監視）、dual reviewer

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（infra）| シニア | 本社 IT 室 | Argo CD UI / Backstage calendar | upgrade window 確認 / staging 先行 upgrade / Argo Rollouts 段階適用 |
| 関与（ops）| シニア | 本社 IT 室 / リモート | Perses dashboard | upgrade 中 SLO 監視 / SLO 違反時 alert |
| 承認（dual reviewer）| シニア | 本社 IT 室 / リモート | Mattermost `#infra-ops` | PR レビュー / sign-off |

## 個人 KPI / 達成感

- upgrade window ≤ 30 日の SLO 達成率を Backstage で可視化でき、upgrade 完了の達成感を定量的に得られる
- staging chaos drill green → 本番 upgrade 可否ゲートで品質確認を数値で確認できる
- cluster_inventory.lock.yaml 更新が PR merge で完了し、upgrade 記録の完全性が保証される

## 工数 / 関与人数 / コスト感

- 工数: 1〜3 日（staging upgrade 4h + chaos drill 4h + 本番 progressive delivery 4h + lock.yaml 更新 1h）
- 関与人数: 3 名（infra 担当者・ops 担当者・dual reviewer）
- コスト感: 中。IaC で自動化されているため手順実施コストは低いが staging 先行で確認コストが追加される

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

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | infra 担当者 | Backstage calendar alert で upgrade window 到来を確認し staging upgrade を開始 | `@infra k8s v1.32 upgrade 開始 / staging 先行 / T+0` |
| 4h | infra 担当者 | staging で Litmus chaos drill を実行し SLO 維持を確認 | `staging chaos drill green / SLO 維持確認` |
| 1d | infra 担当者 | 本番 cluster に Argo Rollouts progressive delivery で段階 apply | `本番 upgrade 開始 / progressive delivery 実行中` |
| 1d+4h | infra 担当者 | upgrade 完了後 cluster_inventory.lock.yaml を更新し dual reviewer sign-off | `lock.yaml 更新 / PR #NNN / sign-off 完了` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（infra は全業務の k8s cluster / network / storage の基盤を担うため）。特に影響度が高い 2 業務:

- **受注**: upgrade 停止 window 中は受注システムが一時停止するため、upgrade 計画を sales 担当者に事前通知し window 設定を調整する。
- **警報配信**: 製造ライン停止を引き起こす alert のリアルタイム配信が upgrade window 中に途絶えないよう、警報配信 namespace を最後に migration する。

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

## 失敗パターン (anti-pattern)

- 手順書なしの直接 kubectl upgrade: IaC を迂回した手動 upgrade は drift detection CI が検知する
- staging スキップ: chaos drill なしの本番 upgrade は upgrade gate CI が阻止する

## 関連参照

- [README.md](README.md) — infra 担当者シナリオ index
- [06_Chaos_drill実行.md](06_Chaos_drill実行.md) — Chaos drill 実行シナリオ
- [../../../03_概要設計/05_infra設計方針/README.md](../../../03_概要設計/05_infra設計方針/README.md) — infra 設計方針
