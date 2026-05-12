---
id: plan.infra.scenario_kyverno_policy_addition
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

# Kyverno admission policy 追加

## 一文方針

新しい security 要件 / OSS 追加 / 脅威モデル更新に伴い、IaC 宣言 → staging 先行検証 → Conftest メタ検証 → GitOps 本番適用の手順で Kyverno admission policy を安全に追加する。

## Trigger（発火条件）

- 新しい security 要件が生じた時
- OSS 追加に伴い新たな admission 制御が必要になった時
- 脅威モデル更新により Kyverno policy の追加が必要になった時

## 想定頻度 / 典型きっかけ

想定頻度: 月次〜四半期。典型きっかけ: 「security 担当者から hostNetwork=true を禁止する新 policy の追加要求が来た」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: security 担当者（脅威モデルレビュー）、dual reviewer

## 前提

- Kyverno が cluster に導入済みで GitOps（Argo CD）で管理されていること
- `infra_enforcement_catalog.lock.yaml` に既存 policy カタログが記録済みであること

## 流れ

1. 新 policy の対象（Pod / Deployment / ServiceAccount 等）と audit 目的を明確化
2. Kyverno ClusterPolicy の YAML を OpenTofu / Kustomize の IaC で宣言
3. staging cluster に Kyverno policy を先行適用し既存ワークロードへの影響確認
4. 違反 workload が staging で検出された場合: 修正 or exemption（audit mode に留める）を決定
5. Conftest でポリシーのメタ検証（policy as code の lint）
6. 本番 cluster への GitOps 適用（Argo CD）
7. `infra_enforcement_catalog.lock.yaml` に新 policy を追記し dual reviewer sign-off

## 関連適合仕様 / 関連 OSS

- infra 強制機構: [../../../04_詳細設計/02_強制機構/04_infra強制機構.md](../../../04_詳細設計/02_強制機構/04_infra強制機構.md)
- infra 設計方針 セキュリティ方針: [../../../03_概要設計/05_infra設計方針/04_セキュリティ方針.md](../../../03_概要設計/05_infra設計方針/04_セキュリティ方針.md)
- 関連 OSS: Kyverno / Argo CD / Conftest / OpenTofu / Kustomize

## 期待結果 / 観測指標

- 新 policy が本番 cluster に適用されていること
- 既存ワークロードへの意図しない影響がないこと（staging で確認済み）
- `infra_enforcement_catalog.lock.yaml` に新 policy が追記されていること
- dual reviewer sign-off が記録されていること

## 失敗時の挙動 / escalation

- staging で既存ワークロード大量違反 → policy 見直し + PR 差し戻し
- Conftest メタ検証 fail → IaC 修正後に再 lint + 再レビュー
- 本番適用後に violation 検出 → Kyverno audit mode に降格 + 影響調査 + security 担当者 escalation

**escalate 先**: security 担当者 / Mattermost `#infra-incident`
**SLA: 1h 以内（拘束）**
**runbook**: Backstage runbook `kyverno-policy-rollback` を参照

## 関連参照

- [README.md](README.md) — infra 担当者シナリオ index
- [08_secret_rotation.md](08_secret_rotation.md) — secret rotation シナリオ
- [../../../03_概要設計/05_infra設計方針/README.md](../../../03_概要設計/05_infra設計方針/README.md) — infra 設計方針
