---
id: plan.data.scenario_preservation_class_change
axis: data
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.data.data_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, C, D]
  proof_classes: []
---

# preservation_class 変更

## 一文方針

業務要件の変更に起因する [5 preservation_class](../../../03_概要設計/06_data設計方針/README.md) の昇降格を IaC 宣言 → [restore_drill AND-gate](../../../03_概要設計/06_data設計方針/07_復旧訓練方針.md) → dual reviewer sign-off の順で完結させ、クラス変更が実績のないまま本番に反映されないことを保証する。

> 朝 9 時、本社 IT 室の data 担当者（シニア級）が `preservation_class.lock.yaml` を開き、規制対応チームから前日に届いていた「医薬品 GMP データの RPO 厳格化」依頼メールに気付く。手元には OpenTofu の IaC ファイルと CloudNativePG cluster manifest、Mattermost 越しに infra 担当者と dual reviewer がいる。

## ペルソナ要約

主役: data 担当者（シニア級）、目的: preservation_class 変更を全データストアに漏れなく反映し retention policy の抜け漏れを排除する

## 現状業務での痛み

- preservation_class 変更が手作業で各データストアに反映されており、抜け漏れが監査指摘になる
- 変更対象のデータストア一覧が文書管理で、網羅性の確認に時間がかかる
- 変更後の確認が手動で、適用漏れの発見が遅れる

## k1s0 でこう変わる

- preservation_class.lock.yaml が全データストアの class 設定を管理し、変更の抜け漏れが CI で自動検知される
- 変更対象一覧が lock.yaml から自動生成されるため、網羅性確認の手間がなくなる
- 変更後の CI validation が全ストアの class 設定を自動確認し、適用漏れを即時検知する

## Trigger（発火条件）

業務要件の変更（SLA 厳格化 / DR 要件変更）により preservation_class の昇降格が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: 年次〜不定期（SLA 変更 / DR 要件変更時）。典型きっかけ: 「規制対応で医薬品 GMP データの RPO が 5 分→60 秒に厳格化され、v1_cluster_replicated から v1_cross_region_replicated への昇格が求められた」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: infra 担当者（IaC / network topology 変更）
- 関与: dual reviewer（sign-off）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（data）| シニア | 本社 IT 室 | `preservation_class.lock.yaml` / CloudNativePG dashboard | class 変更宣言・staging drill 実施・lock.yaml 更新 |
| 関与（infra）| シニア | 本社 IT 室 / リモート | Argo CD / Kyverno | IaC / network topology 変更・replication 設定更新 |
| 承認（dual reviewer）| シニア | 本社 / リモート | Mattermost `#data-ops` | sign-off レビュー |

## 個人 KPI / 達成感

- preservation_class 設定の CI coverage 100% が定量確認でき、retention 管理の完全性を実感できる
- 変更作業の所要時間短縮を数値で確認でき、運用効率の改善を達成感として得られる

## 工数 / 関与人数 / コスト感

- 工数: 半日〜1 日（lock.yaml 更新 2h + CI validation 確認 1h + 全ストア反映確認 2h）
- 関与人数: 2〜3 名（data 担当者・compliance 担当者・dual reviewer）
- コスト感: 低。lock.yaml が変更対象を自動列挙するため手動確認コストが最小化される

## 前提

- 5 preservation_class が定義済み
  - `v1_local_only`
  - `v1_zone_replicated`
  - `v1_cluster_replicated`
  - `v1_cross_region_replicated`
  - `v1_global_replicated`
- `preservation_class.lock.yaml` に全 data store の現在の class が記録済み
- restore_drill の drill cadence が class 別に定義済み
- AND-gate: restore_drill green が確認されるまで本番の class 変更を保留する規約が確立済み

## 流れ

1. 変更対象の data store（CloudNativePG cluster / Kafka topic / Valkey / ClickHouse）と変更前後の class を明確化し、変更理由を記録する
2. 新 class の `replication_topology`（`sync_within_cluster` / `async_cross_cluster` / `sync_cross_region` / `cell_partitioned_multi_region`）を IaC（OpenTofu / Kustomize）で宣言する
3. CloudNativePG Cluster manifest の `replicas` / `streaming_replication` / `barman-cloud` の設定を新 class に合わせて更新する
4. 新 class の restore_drill を staging 環境で実施する。drill cadence は昇格後の新 class のサイクルに合わせる
5. **AND-gate**: restore_drill が green であることを確認するまで、本番の preservation_class 変更を保留する
6. AND-gate 通過後、`preservation_class.lock.yaml` を新 class の内容に更新し、dual reviewer sign-off を得る

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | data 担当者 | preservation_class.lock.yaml で変更対象ストアを確認し変更 PR を作成 | `lock.yaml 変更対象確認 / PR #NNN 作成` |
| 2h | data 担当者 | 全対象ストアに class 変更を適用し CI validation を実行 | `全ストア変更適用 / CI validation green` |
| 4h | data 担当者 | 変更後の retention 動作を staging で確認し PR を提出 | `staging 確認完了 / PR #NNN 提出` |
| 1d | dual reviewer + compliance 担当者 | lock.yaml と全ストア設定を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（data は全業務の PostgreSQL / Kafka / ClickHouse の永続化基盤を担うため）。特に影響度が高い 2 業務:

- **FA（ファクトリーオートメーション）**: 生産ラインの制御データは DR 能力の変化が直接的な稼働停止リスクに連動するため、class 昇格による RPO 短縮の恩恵が最も大きい。
- **受注管理**: 受注データは事業継続の最重要データであり、class 変更による replication topology の変更は受注処理の可用性 SLA に直結する。

## 関連適合仕様 / 関連 OSS

- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- 関連 OSS: CloudNativePG / Strimzi（Kafka）/ Rook+Ceph

## 期待結果 / 観測指標

- IaC が新 class の replication_topology を宣言し、apply が成功している
- staging での restore_drill が green（restore_window 仕様内）
- `preservation_class.lock.yaml` が新 class を反映済み
- dual reviewer 2 名の sign-off が記録済み

## 失敗時の挙動 / escalation

- **restore_drill fail**: AND-gate が通過できないため本番 class 変更は保留のまま。原因調査と改善 PR を提出してから再 drill を実施する。
- **IaC apply fail**: 新 class の manifest を rollback し、infra 担当者と協力して原因を特定する。
- **staging で data 不整合**: class 変更を中断し、**postmortem 期限: 3 営業日以内**（Backstage runbook `data-postmortem-template` を使用）。
- **staging で data 不整合発生**: class 変更を即時中断。data 担当者 + infra 担当者に Mattermost `#data-incident` で連絡（**SLA: 30 分以内**）。Backstage runbook `preservation-class-rollback` を参照。**postmortem 期限: 3 営業日以内**。
- **restore_drill fail（staging で drill が restoration window 超過）**: 1.0.0 ship blocker 認定。Backstage ticket `data-drill-fail-<date>` を起票し次 drill までに改善計画提出（**SLA: 5 営業日以内**）。


## 失敗パターン (anti-pattern)

- lock.yaml を迂回した直接変更: Kafka topic や PostgreSQL tablespace を直接変更すると lock.yaml が不整合になり CI が検知する
- 一部ストアのみ更新: lock.yaml の全ストア一括変更を省略すると coverage check が fail する

## 関連参照

- [data 設計方針](../../../03_概要設計/06_data設計方針/README.md) — 5 preservation_class の設計思想と replication_topology の全体方針
- [restore drill シナリオ](03_restore_drill.md) — AND-gate を構成する restore_drill の実施手順と drill cadence の詳細
- [シナリオ index](README.md) — data 担当者シナリオ全体の構成と preservation_class 一覧
