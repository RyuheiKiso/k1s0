---
id: plan.infra.scenario_platform_upgrade
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

# platform upgrade（運用基盤 OSS upgrade）

## 一文方針

infra 担当者が運用基盤 OSS 自身（Argo CD / Backstage / Kyverno / OpenBao / Litmus 等）を upgrade し、single OSS deep-dive 方針と GitOps 宣言的真の原則を維持する。

> 朝 9 時、本社 IT 室の infra 担当者（シニア級）が Argo CD UI で staging cluster の ApplicationSet を確認中に、「Argo CD v2.11 公開 / ApplicationSet generator 機能追加」の upstream release note が Mattermost `#infra-ops` に流れていることに気付く。手元には cluster_inventory.lock.yaml と Helm chart の values.yaml、Mattermost 越しに ops 担当者・tier1 担当者・dual reviewer がいる。

## Trigger（発火条件）

- 運用基盤 OSS の minor / major release が公開され、upgrade が必要になった時
- CVE 対応で緊急 upgrade が必要になった時

## 想定頻度 / 典型きっかけ

想定頻度: 四半期〜年次（planned upgrade）/ イベント駆動（緊急 CVE 時）

典型きっかけ: 「Argo CD v2.11 が公開され、ApplicationSet generator に必要な機能が追加された。staging で先行検証してから本番 upgrade を実施する」「Kyverno に CVSS 8.5 の CVE が公開され、4h 以内の緊急 upgrade が必要になった」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: ops 担当者（upgrade 中の SLO 監視）/ tier1 担当者（tier1 Library との互換確認）
- 承認: dual reviewer（infra 担当者 2 名、変更 PR の author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（infra）| シニア | 本社 IT 室 | Argo CD UI / cluster_inventory.lock.yaml | CHANGELOG 確認 / Harbor mirror 確認 / staging 先行 upgrade / Litmus chaos drill / 本番 progressive 適用 |
| 関与（ops）| シニア | 本社 IT 室 / リモート | Perses dashboard（platform-availability パネル）| upgrade 中 SLO 監視 / SLO 超過時 escalation |
| 関与（tier1）| シニア | 本社 IT 室 / リモート | Harbor mirror / tier1 Library CI | tier1 Library との互換確認 / breaking change 解析支援 |
| 承認（dual reviewer）| シニア | 本社 IT 室 / リモート | Mattermost `#infra-ops` | staging 確認結果レビュー / cluster_inventory.lock.yaml sign-off |

## 前提

- `cluster_inventory.lock.yaml` に現行バージョンが記録済みであること
- Helm chart が Harbor mirror に存在すること
- 各 OSS の Helm chart が IaC（Kustomize / Helm）で宣言的に管理済みであること
- k8s cluster upgrade（シナリオ 01）とは原則独立して実施可能。ただし upgrade 対象 OSS（Argo CD 等）が特定 k8s minor バージョンに依存する場合は 01 を先行させてから本シナリオを実施する。

## 流れ

1. upgrade 対象の OSS と version を確認し、CHANGELOG / release note で breaking change をチェックする
2. Harbor mirror に upgrade 対象の Helm chart / image が存在するか確認する（存在しない場合は tier1 担当者に Harbor mirror 更新を依頼）
3. staging cluster で先行 upgrade を実施する
   - `infra/helm/<oss-name>/values.yaml` の version を更新し PR 作成
   - Argo CD の `ApplicationSet` で staging cluster に先行適用
4. staging で upgrade 後の動作確認を実施する
   - Argo CD: ApplicationSet が staging で同期成功、UI が正常起動、Git リポジトリとの同期が green
   - Backstage: プラグインが全て起動、API が正常応答
   - Kyverno: 全 policy が Applied 状態、違反ゼロ
   - OpenBao: unseal 成功、dynamic secret の lease 取得成功
5. Litmus chaos drill（該当 OSS の kill chaos）を staging で実施し、upgrade 後も chaos から回復できることを確認する
6. 本番 cluster への upgrade: Argo CD progressive delivery（canary 10%→50%→100%）で段階適用
7. 本番 upgrade 完了後に `cluster_inventory.lock.yaml` を更新し、dual reviewer sign-off を取得する
8. upgrade 後 24h は Perses で SLO を監視する

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（infra は全業務の k8s cluster / network / storage の基盤を担うため）。特に影響度が高い 2 業務:

- **受注**: 受注業務は Argo CD / Kyverno に最も依存した業務 namespace を持つため、platform upgrade の staging 確認で受注 namespace の ApplicationSet sync と Kyverno policy 適用を最優先に green 確認する。
- **SCADA**: Argo CD ApplicationSet generator の仕様変更が SCADA の multi-cluster 配信設定に影響するため、staging で SCADA の IoT データ収集 pipeline が正常動作することを確認してから本番 upgrade する。

## 関連適合仕様 / 関連 OSS

- クラスタ位相適合仕様: [../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md](../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md)
- infra 強制機構: [../../../04_詳細設計/02_強制機構/04_infra強制機構.md](../../../04_詳細設計/02_強制機構/04_infra強制機構.md)
- 関連 OSS: Argo CD / Argo Rollouts / Kyverno / OpenBao / Litmus / Backstage / Helm / Harbor

## 期待結果 / 観測指標

- artifact: `cluster_inventory.lock.yaml` に upgrade 済みバージョンが記録済み
- staging: 流れ step 4 の全確認項目（Argo CD / Backstage / Kyverno / OpenBao の各動作確認）が green
- ci: Litmus chaos drill（upgrade 後の kill chaos）が staging で green
- progressive: 本番 Argo Rollouts progressive delivery 全 step 完了（10%→50%→100% の pause + health check 全通過）
- slo: upgrade 後 24h の SLO ≥ 99.9%（Perses `cluster-availability` + `platform-availability` パネル）
- sign-off: dual reviewer（infra 担当者 2 名）sign-off 完了

## 失敗時の挙動 / escalation

- **staging での upgrade 失敗（OSS が起動不能）**: rollback（`helm rollback <release> <revision>`）を即時実施。本番 upgrade を中止。tier1 担当者と協力して breaking change の解析（**SLA: 4h 以内**に rollback 完了）。**postmortem 期限: 3 営業日以内**。
- **本番 progressive upgrade 中に SLO 超過**: Argo Rollouts の自動 rollback が発動することを確認。ops 担当者に Mattermost `#infra-incident` で通報（**SLA: 10 分以内**）。Backstage runbook `platform-upgrade-rollback` を参照。
- **Kyverno upgrade 後に既存 workload が policy 違反と判定**: 違反 workload を特定し、Kyverno policy を audit mode に一時的に降格（**SLA: 1h 以内**に原因特定）。恒久修正まで audit mode を維持しレポートを daily で ops 担当者に共有。

## 関連参照

- [infra 担当者シナリオ index](./README.md) — infra 担当者シナリオ全体の構成と 5 topology_class 一覧
- [cluster upgrade](./01_cluster_upgrade.md) — k8s 自体の upgrade シナリオ。本シナリオは運用基盤 OSS の upgrade
- [GitOps 配信 / progressive delivery](./05_GitOps配信_progressive_delivery.md) — upgrade で使用する Argo Rollouts progressive delivery の詳細
- [infra 設計方針 GitOps 配信方針](../../../03_概要設計/05_infra設計方針/05_GitOps配信方針.md) — Argo CD ApplicationSet generator の設計指針
