---
id: plan.data.scenario_index
axis: data
phase: plan
kind: index
status: draft
depends_on:
  - arch.data.data_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# data 担当者シナリオ index

## 一文方針

data 担当者（シニア級）が日常的に踏む 14 シナリオを 1 ファイル 1 シナリオで列挙する。[5 preservation_class](../../../03_概要設計/06_data設計方針/README.md) の保全・復旧・暗号化と [restore_drill AND-gate](../../../03_概要設計/06_data設計方針/07_復旧訓練方針.md) の維持、Kafka / ClickHouse の運用、DR 実 failover を含む全 19 軸の defense-in-depth 層 E（13 cross-cutting 適合仕様を含む）の永続化部分を提供する。

## 担当者プロフィール

data 担当者はシニア級エンジニアを前提とし、CloudNativePG / Kafka（Strimzi）/ Valkey / ClickHouse / OpenBao の運用経験を持つ。詳細な要件は層別エンジニア要件を参照。

- 参照: [層別エンジニア要件](../../../02_要件定義/05_開発体制要件/01_層別エンジニア要件.md)

- 級: シニア
- 想定人数: 3-5 名
- 必須スキル: PostgreSQL / CloudNativePG / Kafka / Strimzi / ClickHouse / Apicurio Registry / Valkey / Rook+Ceph
- 責務: 5 preservation_class / 4 層保全 / restore_drill / migration toolchain

## 5 preservation_class と restore_drill AND-gate

| class | durability | restore_window | drill_cadence |
|---|---|---|---|
| v1_local_only | local | 4 h | 180 日 |
| v1_zone_replicated | zone_quorum | 60 sec | 90 日 |
| v1_cluster_replicated | cluster_quorum | 5 min | 60 日 |
| v1_cross_region_replicated | region_quorum | 5 min | 30 日 |
| v1_global_replicated | global_quorum | 0（continuous）| 14 日 |

**restore_drill AND-gate**: 全 5 class の restore_drill が green でなければ 1.0.0 ship 不可。drill fail はただちに ship blocker となる。

詳細: [データ保全適合仕様](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)

> **注記**: 「全 19 軸の defense-in-depth 層 E（13 cross-cutting 適合仕様を含む）」の「13」は `docs/04_詳細設計/03_クロスカッティング適合仕様/` 配下の適合仕様**ファイル数**です。tier3 の「13 層強制機構」（tier3 固有の enforcement 13 層）とは別概念です。

## 重要原則
- 物理 delete の発火経路: `archive_to_offline` 以外は持たない（[crypto-shred / crypto-erase](../../../03_概要設計/06_data設計方針/06_ライフサイクル方針.md) で DEK 削除が実質削除）
- forward-only migration: `down` migration は作成しない（[expand-contract pattern](../../../03_概要設計/06_data設計方針/08_マイグレーション方針.md) で対処）
- [atomic 三表書込](../../../03_概要設計/03_tier2設計方針/13_状態遷移パターン.md): state / outbox / audit table は必ず同一 DB トランザクションで書く

## シナリオ一覧

| # | シナリオ名 | trigger 概要 | 想定頻度 | 主たる関連適合仕様 | 種別 |
|---|------------|-------------|----------|-------------------|------|
| 01 | [schema migration](01_schema_migration.md) | tier2 の Domain Event schema / DB schema 変更が必要になった時 | 月次〜四半期 | データ保全適合仕様 / data 強制機構 | [計画] |
| 02 | [preservation_class 変更](02_preservation_class変更.md) | SLA / DR 要件変更により class 昇降格が必要になった時 | 年次〜不定期 | データ保全適合仕様 | [計画] |
| 03 | [restore drill](03_restore_drill.md) | drill cadence 到来時または DR 演習指示があった時 | 定期（class 別 14〜180 日） | データ保全適合仕様 | [周期] |
| 04 | [crypto-erase / archive_to_offline](04_crypto_erase_archive_to_offline.md) | テナント offboarding または long-term archive の offline 移行が必要になった時 | crypto-erase = 不定期（年間 0-5 件）/ archive_to_offline = 月次〜四半期 | データ保全適合仕様 | [緊急]+[計画] |
| 05 | [KEK / DEK rotation](05_暗号化変更_KEK_DEK_rotation.md) | KEK / DEK の定期ローテーションが必要になった時 | 年次（KEK rotation）+ 四半期〜年次（DEK rotation） | 鍵管理適合仕様 | [周期] |
| 06 | [replication lag / split-brain 対応](06_replication_lag_split_brain対応.md) | replication lag SLI アラートまたは split-brain 疑いが生じた時 | イベント駆動（alert 発火時） | データ保全適合仕様 | [緊急] |
| 07 | [PII 専用クラスタ運用](07_PII専用クラスタ運用.md) | 新規 PII 種別追加または PII 専用クラスタ設定変更が必要になった時 | 不定期（新 PII 種別追加時） | データ保全適合仕様 / PII dedicated cluster | [計画] |
| 08 | [Outbox / atomic 三表書込障害対応](08_Outbox_atomic三表書込障害対応.md) | Outbox relay 障害または atomic 三表書込 integrity 違反が検出された時 | イベント駆動（障害発生時） | データ保全適合仕様 | [緊急] |
| 09 | [pg_minor_upgrade](09_pg_minor_upgrade.md) | PostgreSQL minor version EOL 到来または security patch 必要時 | 年次〜不定期 | データ保全適合仕様 | [周期] |
| 10 | [tenant_onboarding](10_tenant_onboarding.md) | 新規テナント契約完了後 onboarding 開始時 | 月次〜四半期 | データ保全適合仕様 / テナント分離適合仕様 | [計画] |
| 11 | [Kafka topic・partition 変更](11_Kafka_Strimzi_topic_partition変更.md) | tier2 から新規 Domain Event topic 追加依頼 / throughput 増加で partition 変更が必要になった時 | 月次〜四半期 | データ保全適合仕様 | [計画] |
| 12 | [ClickHouse tiered storage 運用](12_ClickHouse_tiered_運用.md) | hot tier 容量 80% 超 / analytics クエリ性能劣化 / 新規 Projector 追加時 | 不定期（容量アラートまたは四半期レビュー） | データ保全適合仕様 | [計画]+[緊急] |
| 13 | [archive_to_offline 復元](13_archive_to_offline復元.md) | litigation hold / 外部監査 / 障害調査のため archived data を online に戻す必要が生じた時 | 不定期（年間 0-3 件） | データ保全適合仕様 | [緊急] |
| 14 | [DR cross-region 実 failover](14_DR_cross_region実failover.md) | primary region 喪失インシデント時（ops 担当者の DR 宣言受領時） | 非計画的（年間 0-1 件） | データ保全適合仕様 / クラスタ位相適合仕様 | [緊急] |

## 新規参画者向けオンボーディング

シニア級エンジニアとして着任した際の推奨学習順序:

- **Day 1-3**: README 全体読了 → 01（schema migration）→ 09（pg minor upgrade）を通読（最頻運用業務）
- **Day 4-7**: 03（restore drill）の次回 drill に観察役として参加し、drill 手順を身体で覚える
- **Week 2**: 10（tenant onboarding）を先輩担当者とペアで実施
- **Week 3-4**: 06（replication lag 対応）→ 08（Outbox 障害対応）のシミュレーションを staging で演習
- **Month 2 以降**: 02（preservation class 変更）→ 04（crypto erase）→ 05（暗号化変更）→ 07（PII クラスタ）→ 11（Kafka topic 変更）→ 12（ClickHouse tiered 運用）は発生時に担当
- **Month 3 以降**: 14（DR 実 failover）に備えて 03（restore drill）を実施済みにしておく。13（archive 復元）は法務 / 監査イベント駆動のため平常時は不要

## シナリオ間の依存関係

- **01（schema migration）と 09（pg minor upgrade）の排他**: 進行中の schema migration があれば pg upgrade は保留（data-01 「前提」参照）。upgrade window と migration window が重なる場合は upgrade を先行させる。
- **10（tenant onboarding）→ tier2-09（業務マスタ CSV import）**: onboarding 完了後に tier2 担当者が 09 の CSV import を実施する順序。
- **02（preservation class 変更）の AND-gate**: 変更後の restoration class で restore_drill（03）が green になるまで本番適用を保留。
- **04（crypto erase）→ 05（暗号化変更）の順序確認**: erase（04）と rotation（05）を同時実施しない。rotation（05）完了後に erase（04）を実施する。
- **03（restore drill）→ 14（DR 実 failover）の前提**: drill が最後に green でなければ実 failover の RTO 保証は担保されない。drill fail を放置した状態で実 failover に臨まない。
- **04（crypto erase）済みデータは 13（archive 復元）不可**: DEK revoke 後のデータは物理的に復元不能。erase と archive 操作の順序は `data_lifecycle.lock.yaml` で管理する。
- **11（Kafka topic 追加）→ tier2-13（Read model Projector 追加）の順序**: Kafka topic が存在しない状態で Projector を deploy すると Consumer エラーが発生する。topic 作成後に Projector を deploy する。

## 関連参照

- [data 設計方針](../../../03_概要設計/06_data設計方針/README.md) — 5 preservation_class・4 層保全・restore_drill AND-gate・lifecycle 単一経路の設計原則
- [ターゲットと利用シナリオ index](../README.md) — 全担当者シナリオの横断構成と各軸シナリオ index へのリンク
