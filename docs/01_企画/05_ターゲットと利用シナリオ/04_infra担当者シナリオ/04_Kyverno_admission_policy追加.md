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

> 朝 9 時、本社 IT 室の infra 担当者（シニア級）が Kyverno UI の policy report 画面を確認中に、security 担当者から Mattermost `#infra-ops` で「hostNetwork=true を禁止する新 policy の追加要求」が届いていることに気付く。手元には infra_enforcement_catalog.lock.yaml と Kyverno ClusterPolicy の YAML テンプレート、Mattermost 越しに security 担当者と dual reviewer がいる。

## ペルソナ要約

主役: infra 担当者（シニア級）、目的: Kyverno admission policy を GitOps で管理し policy drift を構造的に防ぐ

## 現状業務での痛み

- admission policy を手書き YAML で管理して policy drift が発生し、本番 cluster に意図しない policy が適用される
- policy 変更のレビュープロセスがなく、誤った policy 設定が production に影響を与えるまで気付かれない
- policy のカバレッジが把握できず、どのリソースに policy が適用されているかが不明確

## k1s0 でこう変わる

- Kyverno policy が GitOps で管理され、全変更が PR レビュー経由で審査される
- policy dry-run が CI で必須化され、意図しない policy 変更が merge 前に検知される
- policy カバレッジが CI で計測され、未適用リソースが定量的に把握できる

## Trigger（発火条件）

- 新しい security 要件が生じた時
- OSS 追加に伴い新たな admission 制御が必要になった時
- 脅威モデル更新により Kyverno policy の追加が必要になった時

## 想定頻度 / 典型きっかけ

想定頻度: 月次〜四半期。典型きっかけ: 「security 担当者から hostNetwork=true を禁止する新 policy の追加要求が来た」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: security 担当者（脅威モデルレビュー）、dual reviewer

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（infra）| シニア | 本社 IT 室 | Kyverno UI（policy report） | policy IaC 宣言 / staging 先行適用 / Conftest lint / 本番 GitOps 適用 |
| 関与（security）| シニア | 本社 IT 室 / リモート | security dashboard / Mattermost `#security-ops` | 脅威モデルレビュー / policy 要件定義 / 本番適用後の violation 確認 |
| 承認（dual reviewer）| シニア | 本社 IT 室 / リモート | Mattermost `#infra-ops` | PR レビュー / infra_enforcement_catalog.lock.yaml sign-off |

## 個人 KPI / 達成感

- policy drift 0 件が CI で確認でき、policy 管理の精度向上を定量的に得られる
- policy PR の review cycle 短縮を数値で確認でき、チームの生産性改善を実感できる

## 工数 / 関与人数 / コスト感

- 工数: 1〜2 日（policy GitOps 移行 4h + dry-run 設定 2h + CI 統合 4h）
- 関与人数: 2〜3 名（infra 担当者・security 担当者・dual reviewer）
- コスト感: 低〜中。GitOps 移行は一度の作業で完了し継続コストは PR レビューのみ

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

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | infra 担当者 | 既存 Kyverno policy を Git リポジトリに移行し Argo CD に登録 | `Kyverno policy GitOps 移行完了 / Argo CD 登録` |
| 4h | infra 担当者 | policy dry-run を CI に統合し既存 policy の dry-run green を確認 | `dry-run CI 統合完了 / 既存 policy green` |
| 1d | infra 担当者 | policy カバレッジ計測を追加し PR 提出 | `policy coverage N% / PR #NNN 提出` |
| 1d+2h | dual reviewer + security 担当者 | GitOps 設定と policy カバレッジを確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（infra は全業務の k8s cluster / network / storage の基盤を担うため）。特に影響度が高い 2 業務:

- **受注**: 受注業務は最多の business policy を持つため、新 Kyverno policy が受注 namespace の Pod に意図しない admission 拒否を引き起こさないことを staging で重点確認する。
- **FA（設備操作）**: 工場操作に関わる Pod が policy 違反で起動不可になると設備制御が停止するため、FA namespace を対象とした exemption 設計を security 担当者と事前合意する。

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

## 失敗パターン (anti-pattern)

- kubectl apply で直接 policy 変更: GitOps を迂回した手動 apply は drift detection が検知する
- dry-run なしの policy 変更: dry-run を省略した policy 変更は CI gate が阻止する

## 関連参照

- [README.md](README.md) — infra 担当者シナリオ index
- [08_secret_rotation.md](08_secret_rotation.md) — secret rotation シナリオ
- [../../../03_概要設計/05_infra設計方針/README.md](../../../03_概要設計/05_infra設計方針/README.md) — infra 設計方針
