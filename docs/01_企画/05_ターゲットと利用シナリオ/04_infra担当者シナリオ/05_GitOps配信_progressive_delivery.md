---
id: plan.infra.scenario_gitops_progressive_delivery
axis: infra
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.infra.infra_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [E]
  proof_classes: []
---

# GitOps 配信 progressive delivery

## 一文方針

新サービス / 新 Library version のデプロイを、cosign 署名検証 → Argo Rollouts canary → Perses SLO 監視 → release_gate 記録の手順で GitOps 経由で progressive に配信し、品質を保証する。

> 朝 9 時、本社 IT 室の infra 担当者（シニア級）が Argo CD UI で ApplicationSet の sync 状態を確認中に、tier1 Library v0.18.0 の PR が merge 済みで本番配信の準備が整っていることに気付く。手元には release_gate.lock.yaml と Argo Rollouts canary 設定、Mattermost 越しに ops 担当者と dual reviewer がいる。

## Trigger（発火条件）

- 新サービスのデプロイ要求が来た時
- 新 Library version のデプロイを GitOps 経由で progressive に配信する必要が生じた時

## 想定頻度 / 典型きっかけ

想定頻度: デプロイの都度（リリース cadence は週次〜月次）。典型きっかけ: 「tier1 Library の v0.18.0 を本番 cluster に段階配信する際に Argo Rollouts の canary strategy を設定した」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: ops 担当者（SLO 監視）、dual reviewer

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（infra）| シニア | 本社 IT 室 | Argo CD UI / release_gate.lock.yaml | ApplicationSet 宣言 / canary strategy 設定 / cosign 検証確認 / 段階配信監視 |
| 関与（ops）| シニア | 本社 IT 室 / リモート | Perses dashboard（SLO パネル）| canary 段階での SLO 監視 / SLO 閾値超過時 escalation |
| 承認（dual reviewer）| シニア | 本社 IT 室 / リモート | Mattermost `#infra-ops` | release PR レビュー / release_gate.lock.yaml sign-off |

## 前提

- Argo CD / Argo Rollouts が cluster に導入済みで GitOps で管理されていること
- cosign による image 署名が supply chain の必須要件であること
- `release_gate.lock.yaml` が release 管理の SOT（Source of Truth）であること

## 流れ

1. Argo CD [ApplicationSet generator](../../../03_概要設計/05_infra設計方針/05_GitOps配信方針.md) で対象 cluster / namespace のマニフェストを宣言
2. Argo Rollouts の Canary strategy を設定（step: 10% → 30% → 100% / pause 時間 / SLO 閾値で自動 rollback）
3. cosign signed image tag であることを Kyverno policy で確認（未署名は admission 拒否）
4. Argo Events で deploy イベントを Mattermost / 自製 escalation engine に通知
5. Perses dashboard で SLO（error rate / latency p99）が canary 段階で閾値内であることを確認
6. 全 step 完了後に `release_gate.lock.yaml` に release record を追記
7. dual reviewer sign-off

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（infra は全業務の k8s cluster / network / storage の基盤を担うため）。特に影響度が高い 2 業務:

- **受注**: 受注業務は最も多くのリリース cadence を持つため、progressive delivery の canary 段階で受注 API の error rate / latency p99 を Perses で重点監視し、異常時は自動 rollback で業務影響を局所化する。
- **SCADA**: IoT テレメトリ収集の Library update は大量データ処理に影響するため、canary 10% 段階でのデータ欠損がないことを確認してから次ステップに進む。

## 関連適合仕様 / 関連 OSS

- infra 強制機構: [../../../04_詳細設計/02_強制機構/04_infra強制機構.md](../../../04_詳細設計/02_強制機構/04_infra強制機構.md)
- 関連 OSS: Argo CD / Argo Rollouts / Argo Events / Kyverno / cosign / Perses / Mattermost

## 期待結果 / 観測指標

- 新サービス / 新 Library version が canary 段階を経て 100% 配信されていること
- SLO（error rate / latency p99）が全 canary 段階で閾値内に収まっていること
- `release_gate.lock.yaml` に release record が追記されていること
- dual reviewer sign-off が記録されていること

## 失敗時の挙動 / escalation

- cosign 署名未確認 → Kyverno admission 拒否 + deploy 中止 + supply chain 担当者 escalation
- canary 段階で SLO 閾値超過 → Argo Rollouts 自動 rollback + Perses alert 発火 + ops 担当者 escalation
- Argo Events 通知失敗 → deploy は継続するが手動での状況確認が必要

**escalate 先**: ops 担当者 / Mattermost `#infra-incident`（SLO 超過時）、supply chain 担当者（cosign 拒否時）
**SLA: 10 分以内（拘束）**
**runbook**: Backstage runbook `progressive-delivery-rollback` を参照

## 関連参照

- [README.md](README.md) — infra 担当者シナリオ index
- [04_Kyverno_admission_policy追加.md](04_Kyverno_admission_policy追加.md) — Kyverno admission policy 追加シナリオ
- [../../../03_概要設計/05_infra設計方針/README.md](../../../03_概要設計/05_infra設計方針/README.md) — infra 設計方針
