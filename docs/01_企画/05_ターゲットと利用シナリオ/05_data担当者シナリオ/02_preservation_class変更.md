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
  defense_in_depth_layers: []
  proof_classes: []
---

# preservation_class 変更

## 一文方針

業務要件の変更に起因する [5 preservation_class](../../../03_概要設計/06_data設計方針/README.md) の昇降格を IaC 宣言 → [restore_drill AND-gate](../../../03_概要設計/06_data設計方針/07_復旧訓練方針.md) → dual reviewer sign-off の順で完結させ、クラス変更が実績のないまま本番に反映されないことを保証する。

## Trigger（発火条件）

業務要件の変更（SLA 厳格化 / DR 要件変更）により preservation_class の昇降格が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: 年次〜不定期（SLA 変更 / DR 要件変更時）。典型きっかけ: 「規制対応で医薬品 GMP データの RPO が 5 分→60 秒に厳格化され、v1_cluster_replicated から v1_cross_region_replicated への昇格が求められた」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: infra 担当者（IaC / network topology 変更）
- 関与: dual reviewer（sign-off）

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


## 関連参照

- [data 設計方針](../../../03_概要設計/06_data設計方針/README.md) — 5 preservation_class の設計思想と replication_topology の全体方針
- [restore drill シナリオ](03_restore_drill.md) — AND-gate を構成する restore_drill の実施手順と drill cadence の詳細
- [シナリオ index](README.md) — data 担当者シナリオ全体の構成と preservation_class 一覧
